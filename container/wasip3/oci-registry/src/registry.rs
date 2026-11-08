//! OCI Distribution `/v2/` handlers.

use std::collections::BTreeSet;

use crate::http::App;
use crate::secrets::AuthError;

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub target: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Response {
    fn empty(status: u16) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    fn text(status: u16, body: impl Into<String>) -> Self {
        let body = body.into().into_bytes();
        Self {
            status,
            headers: vec![("Content-Type".into(), "text/plain; charset=utf-8".into())],
            body,
        }
    }

    fn bytes(status: u16, content_type: &str, body: Vec<u8>) -> Self {
        Self {
            status,
            headers: vec![("Content-Type".into(), content_type.into())],
            body,
        }
    }

    fn with_header(mut self, name: &str, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }
}

pub fn handle(app: &App, request: Request) -> Response {
    if let Err(err) = app
        .secrets
        .authorize_basic(header(&request, "authorization"))
    {
        let mut res = Response::empty(401).with_header(
            "WWW-Authenticate",
            "Basic realm=\"oci-registry-wasip3\"",
        );
        res.body = match err {
            AuthError::Missing => b"unauthorized".to_vec(),
            AuthError::Invalid | AuthError::Denied => b"denied".to_vec(),
        };
        return res;
    }

    let (path, query) = split_target(&request.target);
    match (request.method.as_str(), path.as_str()) {
        ("GET", "/v2" | "/v2/") => Response::empty(200)
            .with_header("Docker-Distribution-API-Version", "registry/2.0"),

        ("POST", p) if let Some(name) = strip_suffix(p, "/blobs/uploads/") => {
            start_upload(app, name)
        }
        ("PUT", p) if p.contains("/blobs/uploads/") => {
            complete_upload(app, p, &query, &request)
        }
        ("PATCH", p) if p.contains("/blobs/uploads/") => patch_upload(app, p, &request),
        ("HEAD", p) if let Some((name, digest)) = split_blob(p) => head_blob(app, name, digest),
        ("GET", p) if let Some((name, digest)) = split_blob(p) => get_blob(app, name, digest),
        ("PUT", p) if let Some((name, reference)) = split_manifest(p) => {
            put_manifest(app, name, reference, &request)
        }
        ("HEAD", p) if let Some((name, reference)) = split_manifest(p) => {
            head_manifest(app, name, reference)
        }
        ("GET", p) if let Some((name, reference)) = split_manifest(p) => {
            get_manifest(app, name, reference)
        }
        ("GET", p) if let Some(name) = strip_suffix(p, "/tags/list") => list_tags(app, name),
        ("DELETE", p) if let Some((name, reference)) = split_manifest(p) => {
            delete_manifest(app, name, reference)
        }
        _ => Response::text(404, "not found"),
    }
}

fn start_upload(app: &App, name: &str) -> Response {
    let name = normalize_name(name);
    let id = upload_id();
    let key = format!("{name}/uploads/{id}");
    if let Err(err) = app.store.put(&key, b"") {
        return Response::text(500, err.to_string());
    }
    let location = format!("/v2/{name}/blobs/uploads/{id}");
    Response::empty(202)
        .with_header("Location", location)
        .with_header("Docker-Upload-UUID", id)
        .with_header("Range", "0-0")
}

fn patch_upload(app: &App, path: &str, request: &Request) -> Response {
    let Some((name, id)) = split_upload(path) else {
        return Response::text(400, "bad upload path");
    };
    let key = format!("{name}/uploads/{id}");
    let mut existing = match app.store.get(&key) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return Response::text(404, "upload unknown"),
        Err(err) => return Response::text(500, err.to_string()),
    };
    existing.extend_from_slice(&request.body);
    if let Err(err) = app.store.put(&key, &existing) {
        return Response::text(500, err.to_string());
    }
    let end = existing.len().saturating_sub(1);
    let location = format!("/v2/{name}/blobs/uploads/{id}");
    Response::empty(202)
        .with_header("Location", location)
        .with_header("Docker-Upload-UUID", id)
        .with_header("Range", format!("0-{end}"))
}

fn complete_upload(app: &App, path: &str, query: &str, request: &Request) -> Response {
    let Some((name, id)) = split_upload(path) else {
        return Response::text(400, "bad upload path");
    };
    let Some(digest) = query_param(query, "digest") else {
        return Response::text(400, "digest required");
    };
    if !valid_digest(&digest) {
        return Response::text(400, "invalid digest");
    }
    let upload_key = format!("{name}/uploads/{id}");
    let mut data = match app.store.get(&upload_key) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return Response::text(404, "upload unknown"),
        Err(err) => return Response::text(500, err.to_string()),
    };
    data.extend_from_slice(&request.body);
    let actual = sha256_hex(&data);
    let expected = digest.strip_prefix("sha256:").unwrap_or(&digest);
    if actual != expected {
        return Response::text(400, "digest mismatch");
    }
    let blob_key = blob_key(&name, &digest);
    if let Err(err) = app.store.put(&blob_key, &data) {
        return Response::text(500, err.to_string());
    }
    let _ = app.store.delete(&upload_key);
    Response::empty(201)
        .with_header("Location", format!("/v2/{name}/blobs/{digest}"))
        .with_header("Docker-Content-Digest", digest)
}

fn head_blob(app: &App, name: &str, digest: &str) -> Response {
    let key = blob_key(name, digest);
    if !app.store.has(&key) {
        return Response::text(404, "blob unknown");
    }
    match app.store.get(&key) {
        Ok(Some(bytes)) => Response::empty(200)
            .with_header("Content-Length", bytes.len().to_string())
            .with_header("Docker-Content-Digest", digest.to_string()),
        Ok(None) => Response::text(404, "blob unknown"),
        Err(err) => Response::text(500, err.to_string()),
    }
}

fn get_blob(app: &App, name: &str, digest: &str) -> Response {
    let key = blob_key(name, digest);
    match app.store.get(&key) {
        Ok(Some(bytes)) => Response::bytes(200, "application/octet-stream", bytes)
            .with_header("Docker-Content-Digest", digest.to_string()),
        Ok(None) => Response::text(404, "blob unknown"),
        Err(err) => Response::text(500, err.to_string()),
    }
}

fn put_manifest(app: &App, name: &str, reference: &str, request: &Request) -> Response {
    let digest = format!("sha256:{}", sha256_hex(&request.body));
    let media = header(request, "content-type")
        .unwrap_or("application/vnd.oci.image.manifest.v1+json");
    let manifest_key = blob_key(name, &digest);
    if let Err(err) = app.store.put(&manifest_key, &request.body) {
        return Response::text(500, err.to_string());
    }
    // Always store digest-addressable content; tags are pointers.
    if !reference.starts_with("sha256:") {
        let tag_key = format!("{name}/tags/{reference}");
        if let Err(err) = app.store.put(&tag_key, digest.as_bytes()) {
            return Response::text(500, err.to_string());
        }
        let media_key = format!("{name}/tag-media/{reference}");
        if let Err(err) = app.store.put(&media_key, media.as_bytes()) {
            return Response::text(500, err.to_string());
        }
    } else {
        let media_key = format!("{name}/digest-media/{}", digest.replace(':', "_"));
        if let Err(err) = app.store.put(&media_key, media.as_bytes()) {
            return Response::text(500, err.to_string());
        }
    }
    Response::empty(201)
        .with_header("Location", format!("/v2/{name}/manifests/{reference}"))
        .with_header("Docker-Content-Digest", digest)
}

fn resolve_manifest(
    app: &App,
    name: &str,
    reference: &str,
) -> Result<Option<(String, Vec<u8>, String)>, String> {
    let digest = if reference.starts_with("sha256:") {
        reference.to_string()
    } else {
        let tag_key = format!("{name}/tags/{reference}");
        match app.store.get(&tag_key).map_err(|e| e.to_string())? {
            Some(bytes) => String::from_utf8(bytes).map_err(|e| e.to_string())?,
            None => return Ok(None),
        }
    };
    let key = blob_key(name, &digest);
    let body = match app.store.get(&key).map_err(|e| e.to_string())? {
        Some(bytes) => bytes,
        None => return Ok(None),
    };
    let media_key = if reference.starts_with("sha256:") {
        format!("{name}/digest-media/{}", digest.replace(':', "_"))
    } else {
        format!("{name}/tag-media/{reference}")
    };
    let media = app
        .store
        .get(&media_key)
        .map_err(|e| e.to_string())?
        .and_then(|b| String::from_utf8(b).ok())
        .unwrap_or_else(|| "application/vnd.oci.image.manifest.v1+json".into());
    Ok(Some((digest, body, media)))
}

fn head_manifest(app: &App, name: &str, reference: &str) -> Response {
    match resolve_manifest(app, name, reference) {
        Ok(Some((digest, body, media))) => Response::empty(200)
            .with_header("Content-Type", media)
            .with_header("Content-Length", body.len().to_string())
            .with_header("Docker-Content-Digest", digest),
        Ok(None) => Response::text(404, "manifest unknown"),
        Err(err) => Response::text(500, err),
    }
}

fn get_manifest(app: &App, name: &str, reference: &str) -> Response {
    match resolve_manifest(app, name, reference) {
        Ok(Some((digest, body, media))) => Response::bytes(200, &media, body)
            .with_header("Docker-Content-Digest", digest),
        Ok(None) => Response::text(404, "manifest unknown"),
        Err(err) => Response::text(500, err),
    }
}

fn delete_manifest(app: &App, name: &str, reference: &str) -> Response {
    if reference.starts_with("sha256:") {
        let key = blob_key(name, reference);
        if let Err(err) = app.store.delete(&key) {
            return Response::text(500, err.to_string());
        }
    } else {
        let tag_key = format!("{name}/tags/{reference}");
        if let Err(err) = app.store.delete(&tag_key) {
            return Response::text(500, err.to_string());
        }
        let media_key = format!("{name}/tag-media/{reference}");
        let _ = app.store.delete(&media_key);
    }
    Response::empty(202)
}

fn list_tags(app: &App, name: &str) -> Response {
    let name = normalize_name(name);
    let prefix = format!("{name}/tags/");
    let keys = match app.store.list_prefix(&prefix) {
        Ok(keys) => keys,
        Err(err) => return Response::text(500, err.to_string()),
    };
    let mut tags = BTreeSet::new();
    for key in keys {
        if let Some(tag) = key.strip_prefix(&prefix) {
            tags.insert(tag.to_string());
        }
    }
    let body = format!(
        "{{\"name\":\"{}\",\"tags\":[{}]}}",
        name,
        tags.iter()
            .map(|t| format!("\"{t}\""))
            .collect::<Vec<_>>()
            .join(",")
    );
    Response::bytes(200, "application/json", body.into_bytes())
}

fn header<'a>(request: &'a Request, name: &str) -> Option<&'a str> {
    request
        .headers
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.as_str())
}

fn split_target(target: &str) -> (String, String) {
    match target.split_once('?') {
        Some((path, query)) => (path.to_string(), query.to_string()),
        None => (target.to_string(), String::new()),
    }
}

fn query_param<'a>(query: &'a str, key: &str) -> Option<String> {
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == key {
                return Some(percent_decode(v));
            }
        }
    }
    None
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (from_hex(bytes[i + 1]), from_hex(bytes[i + 2])) {
                out.push((hi << 4) | lo);
                i += 3;
                continue;
            }
        }
        out.push(if bytes[i] == b'+' { b' ' } else { bytes[i] });
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn strip_suffix<'a>(path: &'a str, suffix: &str) -> Option<&'a str> {
    path.strip_prefix("/v2/")?.strip_suffix(suffix)
}

fn split_blob(path: &str) -> Option<(&str, &str)> {
    let rest = path.strip_prefix("/v2/")?;
    let (name, digest) = rest.split_once("/blobs/")?;
    Some((name, digest))
}

fn split_manifest(path: &str) -> Option<(&str, &str)> {
    let rest = path.strip_prefix("/v2/")?;
    let (name, reference) = rest.split_once("/manifests/")?;
    Some((name, reference))
}

fn split_upload(path: &str) -> Option<(String, String)> {
    let rest = path.strip_prefix("/v2/")?;
    let (name, id) = rest.split_once("/blobs/uploads/")?;
    Some((normalize_name(name), id.to_string()))
}

fn normalize_name(name: &str) -> String {
    name.trim_matches('/').to_string()
}

fn blob_key(name: &str, digest: &str) -> String {
    format!("{}/blobs/{}", normalize_name(name), digest.replace(':', "_"))
}

fn valid_digest(digest: &str) -> bool {
    let Some(hex) = digest.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn upload_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{nanos:x}")
}

/// Tiny SHA-256 (std only). Enough for digest checks in this prototype.
fn sha256_hex(data: &[u8]) -> String {
    let hash = sha256(data);
    let mut out = String::with_capacity(64);
    for byte in hash {
        out.push(hex_digit(byte >> 4));
        out.push(hex_digit(byte & 0xf));
    }
    out
}

fn hex_digit(n: u8) -> char {
    char::from(if n < 10 { b'0' + n } else { b'a' + (n - 10) })
}

fn sha256(data: &[u8]) -> [u8; 32] {
    // FIPS 180-4 SHA-256, compact implementation for the prototype.
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let bit_len = (data.len() as u64) * 8;
    let mut padded = data.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = [0u8; 32];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..(i + 1) * 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_empty() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn sha256_abc() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
