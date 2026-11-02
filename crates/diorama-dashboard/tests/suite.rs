use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use diorama_apiserver::ApiserverConfig;
use diorama_apiserver::service::ApiserverService;
use diorama_apiserver::storage::KubernetesStorage;
use diorama_dashboard::{
    COMPONENT_DASHBOARD, DashboardAdapter, DashboardConfig, DashboardService,
    snapshot_from_apiserver,
};
use diorama_datastore::config::DatastoreConfig;
use diorama_datastore::engine::DatastoreEngine;
use diorama_supervisor::{Supervisor, stop_channel};
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

fn ephemeral_config() -> DashboardConfig {
    let loopback = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
    DashboardConfig::new(loopback, loopback).with_refresh_interval(Duration::from_millis(100))
}

#[tokio::test]
async fn dashboard_serves_html_fragments_sse_and_binds_h3() {
    let service = DashboardService::new(ephemeral_config(), None);
    service.start().await.expect("dashboard start");

    let http_addr = service.bound_http_addr().expect("bound HTTP/1.1 addr");
    let h3_addr = service.bound_h3_addr().expect("bound HTTP/3 addr");
    assert_ne!(http_addr.port(), 0);
    assert_ne!(h3_addr.port(), 0);

    for (path, expected) in [
        ("/", "hx-sse=\"connect:/events\""),
        ("/htmx.min.js", "htmx"),
        ("/node", "diorama-single-node"),
        ("/pods", "coredns-7db6d8ff4d-x9k2p"),
    ] {
        let mut stream = TcpStream::connect(http_addr).await.expect("connect http1");
        let req = format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
        stream.write_all(req.as_bytes()).await.expect("write req");
        let mut buf = String::new();
        stream.read_to_string(&mut buf).await.expect("read resp");
        assert!(
            buf.starts_with("HTTP/1.1 200 OK"),
            "unexpected status: {buf}"
        );
        assert!(
            buf.contains("alt-svc: h3="),
            "missing Alt-Svc header: {buf}"
        );
        assert!(buf.contains(expected), "missing {expected} in {buf}");
    }

    let mut sse_stream = TcpStream::connect(http_addr).await.expect("connect sse");
    sse_stream
        .write_all(b"GET /events HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .await
        .expect("write sse req");
    let mut sse_text = String::new();
    let mut chunk = [0u8; 1024];
    tokio::time::timeout(Duration::from_secs(2), async {
        while !sse_text.contains("event: pods") {
            let n = sse_stream.read(&mut chunk).await.expect("sse read bytes");
            assert!(n > 0, "sse stream closed early: {sse_text}");
            sse_text.push_str(&String::from_utf8_lossy(&chunk[..n]));
        }
    })
    .await
    .expect("sse read timeout");
    assert!(
        sse_text.contains("text/event-stream"),
        "sse headers: {sse_text}"
    );
    assert!(
        sse_text.contains(&format!("alt-svc: h3=\":{}\"", h3_addr.port())),
        "sse alt-svc: {sse_text}"
    );
    assert!(sse_text.contains("event: node"), "sse payload: {sse_text}");

    service.stop();
}

#[tokio::test]
async fn dashboard_collects_live_node_and_pod_from_apiserver() {
    let temp = tempfile::tempdir().expect("tempdir");
    let ds_cfg = DatastoreConfig::new(temp.path().join("ds"));
    let (engine, _) = DatastoreEngine::open(ds_cfg).expect("open datastore");
    let storage = KubernetesStorage::new(engine.client(), "/registry");
    let api_cfg = ApiserverConfig::default_for_pki(temp.path(), IpAddr::V4(Ipv4Addr::LOCALHOST));
    let apiserver = ApiserverService::new(api_cfg, storage);
    let client = apiserver.admin_client();

    client
        .create_node(json!({
            "apiVersion": "v1",
            "kind": "Node",
            "metadata": { "name": "edge-node-01" },
            "status": {
                "allocatable": { "cpu": "4000m", "memory": "8192Mi" }
            }
        }))
        .await
        .expect("create node");

    client
        .create_pod(
            "default",
            json!({
                "apiVersion": "v1",
                "kind": "Pod",
                "metadata": { "name": "realtime-probe-pod", "namespace": "default" },
                "spec": {
                    "nodeName": "edge-node-01",
                    "containers": [{
                        "name": "app",
                        "image": "busybox:1.36",
                        "resources": {
                            "requests": { "cpu": "500m", "memory": "1024Mi" }
                        }
                    }]
                },
                "status": {
                    "phase": "Running",
                    "containerStatuses": [{ "name": "app", "restartCount": 2 }]
                }
            }),
        )
        .await
        .expect("create pod");

    let snap = snapshot_from_apiserver(&client)
        .await
        .expect("live snapshot");
    assert!(snap.node_html.contains("edge-node-01"));
    assert!(snap.node_html.contains("apiserver live"));
    assert!(snap.node_html.contains("500m / 4000m"));
    assert!(snap.pods_html.contains("realtime-probe-pod"));
    assert!(snap.pods_html.contains("Running"));
}

#[tokio::test]
async fn dashboard_adapter_runs_under_supervisor_and_stops_cleanly() {
    let service = DashboardService::new(ephemeral_config(), None);
    let reg = DashboardAdapter::registration(
        COMPONENT_DASHBOARD,
        service.clone(),
        Vec::new(),
        Duration::from_secs(5),
    );
    let supervisor = Supervisor::new(vec![reg]).expect("build supervisor");
    let (stop_handle, stop_rx) = stop_channel();

    let sup_task = tokio::spawn(async move { supervisor.run(stop_rx).await });

    for _ in 0..50 {
        if service.bound_http_addr().is_some() && service.bound_h3_addr().is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(service.bound_http_addr().is_some());
    assert!(service.bound_h3_addr().is_some());

    stop_handle.stop();
    let report = sup_task.await.expect("supervisor join");
    assert_eq!(report.failures.len(), 0);
}
