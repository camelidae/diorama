use std::convert::Infallible;
use std::net::SocketAddr;
#[cfg(feature = "h3")]
use std::path::Path;
use std::time::Duration;

use bytes::Bytes;
#[cfg(feature = "h3")]
use futures_util::{SinkExt, StreamExt};
use http::{HeaderValue, Request, Response, StatusCode, header};
use http_body_util::BodyExt;
use http_body_util::combinators::BoxBody;
use http_body_util::{Full, StreamBody};
use hyper::body::{Frame, Incoming};
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
#[cfg(feature = "h3")]
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;
#[cfg(feature = "h3")]
use tokio_quiche::http3::driver::{IncomingH3Headers, OutboundFrame, ServerH3Event};
#[cfg(feature = "h3")]
use tokio_quiche::http3::settings::Http3Settings;
#[cfg(feature = "h3")]
use tokio_quiche::metrics::DefaultMetrics;
#[cfg(feature = "h3")]
use tokio_quiche::quiche::h3::{Header, NameValue};
#[cfg(feature = "h3")]
use tokio_quiche::settings::{CertificateKind, Hooks, QuicSettings, TlsCertificatePaths};
#[cfg(feature = "h3")]
use tokio_quiche::{ConnectionParams, ServerH3Driver, listen};

use crate::collector::DashboardState;
#[cfg(feature = "h3")]
use crate::config::DashboardError;
use crate::config::Result;

type BoxHttpBody = BoxBody<Bytes, Infallible>;

/// Spawns the HTTP/1.1 TCP fallback server advertising `Alt-Svc: h3=":<port>"; ma=86400`.
pub async fn spawn_http1_server(
    listen_addr: SocketAddr,
    state: DashboardState,
    mut shutdown: watch::Receiver<bool>,
) -> Result<(SocketAddr, JoinHandle<()>)> {
    let listener = TcpListener::bind(listen_addr).await?;
    let local_addr = listener.local_addr()?;

    let handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                biased;
                _ = shutdown.changed() => {
                    if *shutdown.borrow() {
                        break;
                    }
                }
                accepted = listener.accept() => {
                    let Ok((stream, _)) = accepted else { continue };
                    let conn_state = state.clone();
                    let conn_shutdown = shutdown.clone();
                    tokio::spawn(async move {
                        let io = TokioIo::new(stream);
                        let svc = service_fn(move |req: Request<Incoming>| {
                            let req_state = conn_state.clone();
                            let req_shutdown = conn_shutdown.clone();
                            async move {
                                Ok::<_, Infallible>(handle_http_request(
                                    req.uri().path(),
                                    &req_state,
                                    req_shutdown,
                                ))
                            }
                        });
                        let _ = hyper::server::conn::http1::Builder::new()
                            .serve_connection(io, svc)
                            .await;
                    });
                }
            }
        }
    });

    Ok((local_addr, handle))
}

/// Spawns the HTTP/3 (QUIC) listener backed by `tokio-quiche` (`ServerH3Driver`).
#[cfg(feature = "h3")]
pub async fn spawn_h3_server(
    listen_addr: SocketAddr,
    cert_file: &Path,
    key_file: &Path,
    state: DashboardState,
    mut shutdown: watch::Receiver<bool>,
) -> Result<(SocketAddr, JoinHandle<()>)> {
    let udp = UdpSocket::bind(listen_addr).await?;
    let local_addr = udp.local_addr()?;

    let mut quic_settings = QuicSettings::default();
    quic_settings.max_idle_timeout = Some(Duration::from_secs(30));
    quic_settings.max_recv_udp_payload_size = 1350;
    quic_settings.max_send_udp_payload_size = 1350;
    quic_settings.disable_client_ip_validation = true;

    let cert_str = cert_file.to_string_lossy().into_owned();
    let key_str = key_file.to_string_lossy().into_owned();
    let tls_cert = TlsCertificatePaths {
        cert: &cert_str,
        private_key: &key_str,
        kind: CertificateKind::X509,
    };
    let params = ConnectionParams::new_server(quic_settings, tls_cert, Hooks::default());
    let mut accept_stream = listen([udp], params, DefaultMetrics)
        .map_err(|e| DashboardError::QuicListen(e.to_string()))?
        .remove(0);

    let handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                biased;
                _ = shutdown.changed() => {
                    if *shutdown.borrow() {
                        break;
                    }
                }
                next_conn = accept_stream.next() => {
                    let Some(Ok(conn)) = next_conn else { break };
                    let (driver, mut controller) = ServerH3Driver::new(Http3Settings::default());
                    let quic_conn = conn.start(driver);
                    let conn_state = state.clone();
                    let conn_shutdown = shutdown.clone();
                    tokio::spawn(async move {
                        let _keep_alive = quic_conn;
                        while let Some(event) = controller.event_receiver_mut().recv().await {
                            if let ServerH3Event::Headers { incoming_headers, .. } = event {
                                tokio::spawn(bridge_h3_request(
                                    conn_state.clone(),
                                    incoming_headers,
                                    conn_shutdown.clone(),
                                ));
                            }
                        }
                    });
                }
            }
        }
    });

    Ok((local_addr, handle))
}

#[cfg(feature = "h3")]
async fn bridge_h3_request(
    state: DashboardState,
    incoming: IncomingH3Headers,
    shutdown: watch::Receiver<bool>,
) {
    let IncomingH3Headers {
        headers, mut send, ..
    } = incoming;
    let path = headers
        .iter()
        .find(|h| h.name() == b":path")
        .and_then(|h| std::str::from_utf8(h.value()).ok())
        .unwrap_or("/");

    let resp = handle_http_request(path, &state, shutdown);
    let mut h3_headers = vec![Header::new(b":status", resp.status().as_str().as_bytes())];
    for (name, value) in resp.headers() {
        h3_headers.push(Header::new(name.as_str().as_bytes(), value.as_bytes()));
    }
    if send
        .send(OutboundFrame::Headers(h3_headers, None))
        .await
        .is_err()
    {
        return;
    }

    let mut body = resp.into_body();
    while let Some(Ok(frame)) = body.frame().await {
        if let Ok(chunk) = frame.into_data()
            && !chunk.is_empty()
            && send
                .send(OutboundFrame::Body(chunk.to_vec().into(), false))
                .await
                .is_err()
        {
            return;
        }
    }
    let _ = send
        .send(OutboundFrame::Body(Vec::new().into(), true))
        .await;
}

/// Dispatches a dashboard HTTP request (`GET /`, `/node`, `/pods`, `/events`).
pub fn handle_http_request(
    path: &str,
    state: &DashboardState,
    shutdown: watch::Receiver<bool>,
) -> Response<BoxHttpBody> {
    match path {
        "/" => html_response(
            StatusCode::OK,
            state.h3_port(),
            state.current_snapshot().render_index(),
        ),
        "/node" => html_response(
            StatusCode::OK,
            state.h3_port(),
            state.current_snapshot().node_html,
        ),
        "/pods" => html_response(
            StatusCode::OK,
            state.h3_port(),
            state.current_snapshot().pods_html,
        ),
        "/htmx.min.js" => js_response(state.h3_port(), crate::render::HTMX_BUNDLE),
        "/events" => sse_response(state, shutdown),
        "/healthz" => html_response(StatusCode::OK, state.h3_port(), "ok\n".into()),
        _ => html_response(StatusCode::NOT_FOUND, state.h3_port(), "not found".into()),
    }
}

fn html_response(status: StatusCode, h3_port: u16, html: String) -> Response<BoxHttpBody> {
    let mut resp = Response::new(Full::new(Bytes::from(html)).boxed());
    *resp.status_mut() = status;
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    if let Ok(alt) = HeaderValue::from_str(&format!("h3=\":{h3_port}\"; ma=86400")) {
        resp.headers_mut().insert(header::ALT_SVC, alt);
    }
    resp
}

fn js_response(h3_port: u16, script: &'static str) -> Response<BoxHttpBody> {
    let mut resp = Response::new(Full::new(Bytes::from_static(script.as_bytes())).boxed());
    *resp.status_mut() = StatusCode::OK;
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/javascript; charset=utf-8"),
    );
    if let Ok(alt) = HeaderValue::from_str(&format!("h3=\":{h3_port}\"; ma=86400")) {
        resp.headers_mut().insert(header::ALT_SVC, alt);
    }
    resp
}

fn sse_response(
    state: &DashboardState,
    mut shutdown: watch::Receiver<bool>,
) -> Response<BoxHttpBody> {
    let (tx, rx) = mpsc::channel::<std::result::Result<Frame<Bytes>, Infallible>>(16);
    let mut sub = state.subscribe();
    let initial = state.current_snapshot().to_sse_frame();

    tokio::spawn(async move {
        if tx
            .send(Ok(Frame::data(Bytes::from(initial))))
            .await
            .is_err()
        {
            return;
        }
        let mut keepalive = tokio::time::interval(Duration::from_secs(10));
        loop {
            tokio::select! {
                biased;
                _ = shutdown.changed() => {
                    if *shutdown.borrow() {
                        break;
                    }
                }
                msg = sub.recv() => {
                    let Ok(snap) = msg else { continue };
                    let payload = Bytes::from(snap.to_sse_frame());
                    if tx.send(Ok(Frame::data(payload))).await.is_err() {
                        break;
                    }
                }
                _ = keepalive.tick() => {
                    if tx
                        .send(Ok(Frame::data(Bytes::from_static(b": keepalive\n\n"))))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            }
        }
    });

    let stream = futures_util::stream::unfold(rx, |mut receiver| async move {
        receiver.recv().await.map(|item| (item, receiver))
    });
    let mut resp = Response::new(BodyExt::boxed(StreamBody::new(stream)));
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/event-stream"),
    );
    resp.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    if let Ok(alt) = HeaderValue::from_str(&format!("h3=\":{}\"; ma=86400", state.h3_port())) {
        resp.headers_mut().insert(header::ALT_SVC, alt);
    }
    resp
}
