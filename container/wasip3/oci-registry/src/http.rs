//! HTTP/1.1 over TCP. Serial accept loop (no threads) so the same binary
//! works on wasip3, which has no thread support in this runtime path.

use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::Arc;

use crate::blobstore::Blobstore;
use crate::registry::{self, Request, Response};
use crate::secrets::Secrets;

pub struct App {
    pub store: Arc<Blobstore>,
    pub secrets: Arc<Secrets>,
}

impl App {
    pub fn serve<A: ToSocketAddrs>(self, addr: A) -> io::Result<()> {
        let listener = TcpListener::bind(addr)?;
        let local = listener.local_addr()?;
        eprintln!("oci-registry listening on http://{local}");
        eprintln!("data: {}", self.store.root().display());
        if self.secrets.password.is_some() {
            eprintln!("auth: Basic password required (OCI_REGISTRY_PASSWORD)");
        } else {
            eprintln!("auth: open (set OCI_REGISTRY_PASSWORD to require Basic)");
        }
        let app = Arc::new(self);
        for stream in listener.incoming() {
            let stream = stream?;
            if let Err(err) = handle_connection(&app, stream) {
                eprintln!("connection error: {err}");
            }
        }
        Ok(())
    }
}

fn handle_connection(app: &App, mut stream: TcpStream) -> io::Result<()> {
    let request = read_request(&mut stream)?;
    let response = registry::handle(app, request);
    write_response(&mut stream, &response)
}

fn read_request(stream: &mut TcpStream) -> io::Result<Request> {
    let mut buf = Vec::with_capacity(4096);
    let mut chunk = [0u8; 1024];
    let header_end = loop {
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "empty request"));
        }
        buf.extend_from_slice(&chunk[..n]);
        if let Some(pos) = find_header_end(&buf) {
            break pos;
        }
        if buf.len() > 64 * 1024 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "headers too large"));
        }
    };

    let header_bytes = &buf[..header_end];
    let rest = &buf[header_end + 4..];
    let header_text = std::str::from_utf8(header_bytes)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let mut lines = header_text.split("\r\n");
    let request_line = lines
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing request line"))?;
    let mut parts = request_line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing method"))?
        .to_string();
    let target = parts
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing path"))?
        .to_string();

    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        headers.push((name.trim().to_ascii_lowercase(), value.trim().to_string()));
    }

    let content_length = headers
        .iter()
        .find(|(k, _)| k == "content-length")
        .and_then(|(_, v)| v.parse::<usize>().ok())
        .unwrap_or(0);

    let mut body = rest.to_vec();
    while body.len() < content_length {
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..n]);
    }
    body.truncate(content_length);

    Ok(Request {
        method,
        target,
        headers,
        body,
    })
}

fn find_header_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n")
}

fn write_response(stream: &mut TcpStream, response: &Response) -> io::Result<()> {
    let reason = status_reason(response.status);
    write!(
        stream,
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nConnection: close\r\n",
        response.status,
        reason,
        response.body.len()
    )?;
    for (name, value) in &response.headers {
        write!(stream, "{name}: {value}\r\n")?;
    }
    write!(stream, "\r\n")?;
    stream.write_all(&response.body)?;
    Ok(())
}

fn status_reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        405 => "Method Not Allowed",
        416 => "Range Not Satisfiable",
        500 => "Internal Server Error",
        _ => "Unknown",
    }
}
