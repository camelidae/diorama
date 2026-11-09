//! Package an existing WASI 0.3 command into deployable artifacts, optionally
//! pushing the OCI image and applying the generated manifests.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

pub const HELP: &str = "Package a WASI 0.3 command for cluster deploy.\n\nUsage:\n  dioramactl wasm deploy <app.wasm> -o <outdir> [--dry-run] [--push-only] [--name <app>] [--port <n>] [--image <ref>] [--registry <url>]\n\nAlways writes app.wasm, app.image.tar, and Kubernetes manifests under <outdir>.\nWithout --dry-run, also pushes the image to --registry (or http://<image-host>) and runs kubectl apply.\nUse --push-only to push without applying manifests.\n";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WasmOptions {
    pub wasm: PathBuf,
    pub dry_run: bool,
    pub push_only: bool,
    pub output: Option<PathBuf>,
    pub name: Option<String>,
    pub port: u16,
    pub image: Option<String>,
    pub registry: Option<String>,
}

pub fn parse(args: &[String]) -> Result<WasmOptions, crate::ParseError> {
    if args.first().map(String::as_str) != Some("deploy") {
        return Err(crate::ParseError::UnknownCommand);
    }
    let mut rest = args[1..].iter();
    let mut wasm = None;
    let mut options = WasmOptions {
        wasm: PathBuf::new(),
        dry_run: false,
        push_only: false,
        output: None,
        name: None,
        port: 8080,
        image: None,
        registry: None,
    };
    while let Some(arg) = rest.next() {
        if arg == "--dry-run" {
            options.dry_run = true;
            continue;
        }
        if arg == "--push-only" {
            options.push_only = true;
            continue;
        }
        if !arg.starts_with('-') {
            if wasm.is_some() {
                return Err(crate::ParseError::UnknownCommand);
            }
            wasm = Some(PathBuf::from(arg));
            continue;
        }
        let (key, value) = match arg.split_once('=') {
            Some(pair) => pair,
            None => (
                arg.as_str(),
                rest.next().ok_or(crate::ParseError::UnknownFlag)?.as_str(),
            ),
        };
        match key {
            "-o" | "--output" if !value.is_empty() => options.output = Some(PathBuf::from(value)),
            "--name" if !value.is_empty() && !value.starts_with('-') => {
                options.name = Some(value.to_string());
            },
            "--port" => {
                options.port = value
                    .parse::<u16>()
                    .map_err(|_| crate::ParseError::UnknownFlag)?;
                if options.port == 0 {
                    return Err(crate::ParseError::UnknownFlag);
                }
            },
            "--image" if !value.is_empty() && !value.starts_with('-') => {
                options.image = Some(value.to_string());
            },
            "--registry" if !value.is_empty() && !value.starts_with('-') => {
                options.registry = Some(value.to_string());
            },
            _ => return Err(crate::ParseError::UnknownFlag),
        }
    }
    let Some(wasm) = wasm else {
        return Err(crate::ParseError::UnknownCommand);
    };
    options.wasm = wasm;
    Ok(options)
}

pub fn execute(
    options: &WasmOptions,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<u8> {
    let Some(output) = options.output.as_ref() else {
        writeln!(stderr, "error: -o/--output <outdir> is required")?;
        return Ok(1);
    };
    let resolved = match resolve(options) {
        Ok(resolved) => resolved,
        Err(err) => {
            writeln!(stderr, "error: {err}")?;
            return Ok(1);
        },
    };
    let packaged = match package(&resolved, output) {
        Ok(packaged) => packaged,
        Err(err) => {
            writeln!(stderr, "error: {err}")?;
            return Ok(1);
        },
    };
    if options.dry_run {
        writeln!(
            stdout,
            "wrote wasm deploy dry-run artifacts for {} to {}",
            resolved.name,
            output.display()
        )?;
        return Ok(0);
    }

    let registry = match registry_base(options.registry.as_deref(), &resolved.image_ref) {
        Ok(registry) => registry,
        Err(err) => {
            writeln!(stderr, "error: {err}")?;
            return Ok(1);
        },
    };
    if let Err(err) = push_image(&registry, &resolved.image_ref, &packaged) {
        writeln!(stderr, "error: push failed: {err}")?;
        return Ok(1);
    }
    writeln!(
        stdout,
        "pushed {} to {}/v2/{}/manifests/{}",
        resolved.image, registry, resolved.image_ref.name, resolved.image_ref.tag
    )?;

    if options.push_only {
        writeln!(
            stdout,
            "wrote wasm deploy artifacts for {} to {} (push-only; skipped apply)",
            resolved.name,
            output.display()
        )?;
        return Ok(0);
    }

    let manifests = [
        output.join("wasm-app-storage.yml"),
        output.join("wasm-app-deployment.yml"),
        output.join("wasm-app-service.yml"),
    ];
    if let Err(err) = kubectl_apply(&manifests) {
        writeln!(stderr, "error: kubectl apply failed: {err}")?;
        return Ok(1);
    }
    writeln!(
        stdout,
        "applied wasm deploy manifests for {} from {}",
        resolved.name,
        output.display()
    )?;
    Ok(0)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ImageRef {
    host: String,
    name: String,
    tag: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Resolved {
    wasm: PathBuf,
    name: String,
    port: u16,
    image: String,
    image_ref: ImageRef,
}

struct Packaged {
    /// Archive members: `blobs/sha256/...`, `index.json`, `oci-layout`.
    archive_files: Vec<(String, Vec<u8>)>,
    /// Config + layer blobs to upload (digest, bytes).
    push_blobs: Vec<(String, Vec<u8>)>,
    manifest: Vec<u8>,
    manifest_media_type: String,
}

fn resolve(options: &WasmOptions) -> io::Result<Resolved> {
    if !options.wasm.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("wasm file not found: {}", options.wasm.display()),
        ));
    }
    let name = options
        .name
        .clone()
        .or_else(|| {
            options
                .wasm
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
        })
        .filter(|name| !name.is_empty())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "unable to derive app name"))?;
    if !is_dns_label(&name) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "app name must be a DNS label (lowercase alphanumeric and '-')",
        ));
    }
    let image = options
        .image
        .clone()
        .unwrap_or_else(|| format!("oci-registry:5000/{name}:dev"));
    let image_ref = parse_image_ref(&image)?;
    Ok(Resolved {
        wasm: options.wasm.clone(),
        name,
        port: options.port,
        image,
        image_ref,
    })
}

fn is_dns_label(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.is_empty() || bytes.len() > 63 {
        return false;
    }
    let (Some(first), Some(last)) = (bytes.first(), bytes.last()) else {
        return false;
    };
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return false;
    }
    if !last.is_ascii_lowercase() && !last.is_ascii_digit() {
        return false;
    }
    bytes
        .iter()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
}

fn parse_image_ref(image: &str) -> io::Result<ImageRef> {
    let Some((host, remainder)) = image.split_once('/') else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("image ref must be host[:port]/name[:tag], got {image}"),
        ));
    };
    if host.is_empty() || remainder.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid image ref: {image}"),
        ));
    }
    let (name, tag) = match remainder.rsplit_once(':') {
        Some((name, tag)) if !name.is_empty() && !tag.is_empty() && !tag.contains('/') => {
            (name, tag)
        },
        _ => (remainder, "latest"),
    };
    if name.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid image ref: {image}"),
        ));
    }
    Ok(ImageRef {
        host: host.to_string(),
        name: name.to_string(),
        tag: tag.to_string(),
    })
}

fn registry_base(explicit: Option<&str>, image: &ImageRef) -> io::Result<String> {
    let raw = explicit
        .map(str::to_string)
        .unwrap_or_else(|| format!("http://{}", image.host));
    let trimmed = raw.trim_end_matches('/');
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "registry URL must start with http:// or https://",
        ));
    }
    if trimmed.starts_with("https://") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "https registries are not supported yet; use http://",
        ));
    }
    Ok(trimmed.to_string())
}

fn package(resolved: &Resolved, output: &Path) -> io::Result<Packaged> {
    fs::create_dir_all(output)?;
    let wasm_bytes = fs::read(&resolved.wasm)?;
    fs::write(output.join("app.wasm"), &wasm_bytes)?;
    let packaged = build_oci(&wasm_bytes, &resolved.image)?;
    write_oci_archive(output.join("app.image.tar"), &packaged)?;
    fs::write(
        output.join("wasm-app-deployment.yml"),
        deployment_yaml(resolved),
    )?;
    fs::write(output.join("wasm-app-service.yml"), service_yaml(resolved))?;
    fs::write(output.join("wasm-app-storage.yml"), storage_yaml(resolved))?;
    Ok(packaged)
}

fn deployment_yaml(resolved: &Resolved) -> String {
    format!(
        r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: {name}
  labels:
    app: {name}
spec:
  replicas: 1
  selector:
    matchLabels:
      app: {name}
  template:
    metadata:
      labels:
        app: {name}
    spec:
      runtimeClassName: wasmtime
      containers:
        - name: {name}
          image: {image}
          imagePullPolicy: IfNotPresent
          ports:
            - name: http
              containerPort: {port}
          volumeMounts:
            - name: data
              mountPath: /data
          resources:
            limits:
              memory: 128Mi
              cpu: "500m"
      volumes:
        - name: data
          persistentVolumeClaim:
            claimName: {name}-data
"#,
        name = resolved.name,
        image = resolved.image,
        port = resolved.port,
    )
}

fn service_yaml(resolved: &Resolved) -> String {
    format!(
        r#"apiVersion: v1
kind: Service
metadata:
  name: {name}
  labels:
    app: {name}
spec:
  selector:
    app: {name}
  ports:
    - name: http
      port: {port}
      targetPort: http
"#,
        name = resolved.name,
        port = resolved.port,
    )
}

fn storage_yaml(resolved: &Resolved) -> String {
    format!(
        r#"apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: {name}-data
  labels:
    app: {name}
spec:
  accessModes:
    - ReadWriteOnce
  storageClassName: local-path
  resources:
    requests:
      storage: 1Gi
"#,
        name = resolved.name,
    )
}

fn build_oci(wasm: &[u8], image: &str) -> io::Result<Packaged> {
    let layer = layer_tar(wasm)?;
    let mut blobs = BTreeMap::new();

    let layer_desc = store_blob(&mut blobs, &layer, "application/vnd.oci.image.layer.v1.tar");
    let config = serde_json::json!({
        "architecture": "wasm32",
        "os": "wasip3",
        "config": {
            "Entrypoint": ["/app.wasm"],
            "WorkingDir": "/"
        },
        "rootfs": {
            "type": "layers",
            "diff_ids": [layer_desc["digest"]]
        }
    });
    let config_bytes = compact_json(&config)?;
    let config_desc = store_blob(
        &mut blobs,
        &config_bytes,
        "application/vnd.oci.image.config.v1+json",
    );
    let manifest = serde_json::json!({
        "schemaVersion": 2,
        "mediaType": "application/vnd.oci.image.manifest.v1+json",
        "config": config_desc,
        "layers": [layer_desc]
    });
    let manifest_bytes = compact_json(&manifest)?;
    let mut manifest_desc = store_blob(
        &mut blobs,
        &manifest_bytes,
        "application/vnd.oci.image.manifest.v1+json",
    );
    manifest_desc["annotations"] = serde_json::json!({
        "org.opencontainers.image.ref.name": image,
        "io.containerd.image.name": image
    });
    manifest_desc["platform"] = serde_json::json!({
        "os": "wasip3",
        "architecture": "wasm32"
    });

    let index = serde_json::json!({
        "schemaVersion": 2,
        "manifests": [manifest_desc]
    });
    let layout = serde_json::json!({ "imageLayoutVersion": "1.0.0" });

    let mut push_blobs = Vec::new();
    for desc in [&config_desc, &layer_desc] {
        let digest = desc["digest"]
            .as_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing digest"))?
            .to_string();
        let path = format!("blobs/sha256/{}", digest.trim_start_matches("sha256:"));
        let data = blobs.get(&path).cloned().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, format!("missing blob {digest}"))
        })?;
        push_blobs.push((digest, data));
    }

    let mut archive_files: Vec<(String, Vec<u8>)> = blobs.into_iter().collect();
    archive_files.push(("index.json".into(), compact_json(&index)?));
    archive_files.push(("oci-layout".into(), compact_json(&layout)?));
    archive_files.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(Packaged {
        archive_files,
        push_blobs,
        manifest: manifest_bytes,
        manifest_media_type: "application/vnd.oci.image.manifest.v1+json".into(),
    })
}

fn write_oci_archive(path: impl AsRef<Path>, packaged: &Packaged) -> io::Result<()> {
    let mut archive = Vec::new();
    for (name, data) in &packaged.archive_files {
        append_ustar_file(&mut archive, name, data)?;
    }
    append_ustar_end(&mut archive);
    fs::write(path, archive)
}

fn push_image(registry: &str, image: &ImageRef, packaged: &Packaged) -> io::Result<()> {
    let endpoint = HttpEndpoint::parse(registry)?;
    let probe = endpoint.request("GET", "/v2/", &[], &[])?;
    if probe.status != 200 {
        return Err(io::Error::other(format!(
            "registry /v2/ returned HTTP {}",
            probe.status
        )));
    }

    for (digest, data) in &packaged.push_blobs {
        upload_blob(&endpoint, &image.name, digest, data)?;
    }

    put_manifest(
        &endpoint,
        &image.name,
        &image.tag,
        &packaged.manifest,
        &packaged.manifest_media_type,
    )?;
    Ok(())
}

fn upload_blob(endpoint: &HttpEndpoint, name: &str, digest: &str, data: &[u8]) -> io::Result<()> {
    let head = endpoint.request("HEAD", &format!("/v2/{name}/blobs/{digest}"), &[], &[])?;
    if head.status == 200 {
        return Ok(());
    }

    let start = endpoint.request(
        "POST",
        &format!("/v2/{name}/blobs/uploads/"),
        &[("Content-Length", "0")],
        &[],
    )?;
    if start.status != 202 {
        return Err(io::Error::other(format!(
            "upload start HTTP {}",
            start.status
        )));
    }
    let location = start
        .header("location")
        .ok_or_else(|| io::Error::other("upload missing Location"))?;
    let put_path = resolve_upload_path(&endpoint.base_path, location, digest)?;
    let put = endpoint.request(
        "PUT",
        &put_path,
        &[("Content-Type", "application/octet-stream")],
        data,
    )?;
    if put.status != 201 && put.status != 202 {
        return Err(io::Error::other(format!(
            "upload complete HTTP {}",
            put.status
        )));
    }
    Ok(())
}

fn put_manifest(
    endpoint: &HttpEndpoint,
    name: &str,
    reference: &str,
    manifest: &[u8],
    media_type: &str,
) -> io::Result<()> {
    let response = endpoint.request(
        "PUT",
        &format!("/v2/{name}/manifests/{reference}"),
        &[("Content-Type", media_type)],
        manifest,
    )?;
    if response.status != 201 && response.status != 200 {
        return Err(io::Error::other(format!(
            "manifest put HTTP {}",
            response.status
        )));
    }
    Ok(())
}

fn resolve_upload_path(base_path: &str, location: &str, digest: &str) -> io::Result<String> {
    let path = if let Some(rest) = location.strip_prefix("http://") {
        let path = rest
            .split_once('/')
            .map(|(_, path)| format!("/{path}"))
            .unwrap_or_else(|| "/".into());
        path
    } else if location.starts_with('/') {
        location.to_string()
    } else {
        format!(
            "{}/{}",
            base_path.trim_end_matches('/'),
            location.trim_start_matches('/')
        )
    };
    let sep = if path.contains('?') { '&' } else { '?' };
    Ok(format!("{path}{sep}digest={digest}"))
}

fn kubectl_apply(paths: &[PathBuf]) -> io::Result<()> {
    for path in paths {
        let output = Command::new("kubectl")
            .args(["apply", "-f"])
            .arg(path)
            .output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let detail = if !stderr.trim().is_empty() {
                stderr.trim().to_string()
            } else {
                stdout.trim().to_string()
            };
            return Err(io::Error::other(format!("{}: {detail}", path.display())));
        }
    }
    Ok(())
}

struct HttpEndpoint {
    host: String,
    port: u16,
    base_path: String,
}

struct HttpResponse {
    status: u16,
    headers: Vec<(String, String)>,
    #[allow(dead_code)]
    body: Vec<u8>,
}

impl HttpEndpoint {
    fn parse(url: &str) -> io::Result<Self> {
        let rest = url
            .strip_prefix("http://")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "expected http:// URL"))?;
        let (authority, path) = match rest.split_once('/') {
            Some((authority, path)) => (authority, format!("/{path}")),
            None => (rest, String::new()),
        };
        let (host, port) = match authority.rsplit_once(':') {
            Some((host, port)) if !host.is_empty() => {
                let port = port.parse::<u16>().map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidInput, format!("bad port in {url}"))
                })?;
                (host.to_string(), port)
            },
            _ => (authority.to_string(), 80),
        };
        Ok(Self {
            host,
            port,
            base_path: path.trim_end_matches('/').to_string(),
        })
    }

    fn request(
        &self,
        method: &str,
        path: &str,
        headers: &[(&str, &str)],
        body: &[u8],
    ) -> io::Result<HttpResponse> {
        let mut stream = TcpStream::connect((self.host.as_str(), self.port))?;
        stream.set_read_timeout(Some(Duration::from_secs(30)))?;
        stream.set_write_timeout(Some(Duration::from_secs(30)))?;
        let path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{path}")
        };
        write!(
            stream,
            "{method} {path} HTTP/1.1\r\nHost: {}:{}\r\nConnection: close\r\nContent-Length: {}\r\n",
            self.host,
            self.port,
            body.len()
        )?;
        for (name, value) in headers {
            write!(stream, "{name}: {value}\r\n")?;
        }
        write!(stream, "\r\n")?;
        stream.write_all(body)?;

        let mut raw = Vec::new();
        stream.read_to_end(&mut raw)?;
        parse_http_response(&raw)
    }
}

impl HttpResponse {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

fn parse_http_response(raw: &[u8]) -> io::Result<HttpResponse> {
    let header_end = raw
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "truncated HTTP response"))?;
    let head = std::str::from_utf8(&raw[..header_end])
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    let mut lines = head.split("\r\n");
    let status_line = lines
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing status line"))?;
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse::<u16>().ok())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "bad status line"))?;
    let mut headers = Vec::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            headers.push((name.trim().to_string(), value.trim().to_string()));
        }
    }
    Ok(HttpResponse {
        status,
        headers,
        body: raw[header_end + 4..].to_vec(),
    })
}

fn layer_tar(wasm: &[u8]) -> io::Result<Vec<u8>> {
    let mut out = Vec::new();
    append_ustar_file(&mut out, "app.wasm", wasm)?;
    append_ustar_end(&mut out);
    Ok(out)
}

fn store_blob(
    blobs: &mut BTreeMap<String, Vec<u8>>,
    data: &[u8],
    media_type: &str,
) -> serde_json::Value {
    let digest = format!("sha256:{}", hex_sha256(data));
    let path = format!("blobs/sha256/{}", digest.trim_start_matches("sha256:"));
    blobs.insert(path, data.to_vec());
    serde_json::json!({
        "mediaType": media_type,
        "digest": digest,
        "size": data.len()
    })
}

fn compact_json(value: &serde_json::Value) -> io::Result<Vec<u8>> {
    serde_json::to_vec(value).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

fn hex_sha256(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push(hex_digit(byte >> 4));
        out.push(hex_digit(byte & 0xf));
    }
    out
}

fn hex_digit(nibble: u8) -> char {
    char::from(if nibble < 10 {
        b'0' + nibble
    } else {
        b'a' + (nibble - 10)
    })
}

fn append_ustar_file(out: &mut Vec<u8>, path: &str, data: &[u8]) -> io::Result<()> {
    if path.len() > 100 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "tar path exceeds 100 bytes",
        ));
    }
    let mut header = [0u8; 512];
    header[..path.len()].copy_from_slice(path.as_bytes());
    write_octal(&mut header[100..108], 0o644);
    write_octal(&mut header[108..116], 0);
    write_octal(&mut header[116..124], 0);
    write_octal(&mut header[124..136], data.len() as u64);
    write_octal(&mut header[136..148], 0);
    header[156] = b'0';
    header[257..262].copy_from_slice(b"ustar");
    header[263] = b'0';
    header[264] = b'0';
    header[148..156].fill(b' ');
    let checksum = header.iter().map(|b| u32::from(*b)).sum::<u32>();
    write_octal(&mut header[148..156], u64::from(checksum));
    header[155] = 0;
    out.extend_from_slice(&header);
    out.extend_from_slice(data);
    let pad = (512 - (data.len() % 512)) % 512;
    out.extend(std::iter::repeat_n(0u8, pad));
    Ok(())
}

fn append_ustar_end(out: &mut Vec<u8>) {
    out.extend(std::iter::repeat_n(0u8, 1024));
}

fn write_octal(dst: &mut [u8], value: u64) {
    let width = dst.len().saturating_sub(1);
    let text = format!("{value:0width$o}");
    let bytes = text.as_bytes();
    let copy = bytes.len().min(width);
    dst[..copy].copy_from_slice(&bytes[..copy]);
    if copy < dst.len() {
        dst[copy] = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmpdir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("dioramactl-wasm-{nanos}"))
    }

    #[test]
    fn parse_dry_run_happy_path() {
        let args = [
            "deploy".into(),
            "app.wasm".into(),
            "--dry-run".into(),
            "-o".into(),
            "./dist/app".into(),
            "--name".into(),
            "demo".into(),
            "--port".into(),
            "9090".into(),
            "--image".into(),
            "oci-registry:5000/demo:v1".into(),
            "--registry".into(),
            "http://127.0.0.1:34521".into(),
        ];
        let options = parse(&args).unwrap();
        assert!(options.dry_run);
        assert_eq!(options.wasm, PathBuf::from("app.wasm"));
        assert_eq!(options.output.as_deref(), Some(Path::new("./dist/app")));
        assert_eq!(options.name.as_deref(), Some("demo"));
        assert_eq!(options.port, 9090);
        assert_eq!(options.image.as_deref(), Some("oci-registry:5000/demo:v1"));
        assert_eq!(options.registry.as_deref(), Some("http://127.0.0.1:34521"));
    }

    #[test]
    fn parse_rejects_unknown_flag() {
        let args = ["deploy".into(), "app.wasm".into(), "--nope".into()];
        assert_eq!(parse(&args), Err(crate::ParseError::UnknownFlag));
    }

    #[test]
    fn parse_requires_deploy_and_wasm() {
        assert_eq!(parse(&[]), Err(crate::ParseError::UnknownCommand));
        assert_eq!(
            parse(&["build".into()]),
            Err(crate::ParseError::UnknownCommand)
        );
        assert_eq!(
            parse(&["deploy".into()]),
            Err(crate::ParseError::UnknownCommand)
        );
    }

    #[test]
    fn parse_image_ref_with_port() {
        let parsed = parse_image_ref("oci-registry:5000/demo:v1").unwrap();
        assert_eq!(parsed.host, "oci-registry:5000");
        assert_eq!(parsed.name, "demo");
        assert_eq!(parsed.tag, "v1");
    }

    #[test]
    fn dry_run_writes_five_artifacts() {
        let dir = tmpdir();
        fs::create_dir_all(&dir).unwrap();
        let wasm = dir.join("input.wasm");
        fs::write(&wasm, b"\0asm\x01\x00\x00\x00").unwrap();
        let out = dir.join("dist");
        let options = WasmOptions {
            wasm,
            dry_run: true,
            push_only: false,
            output: Some(out.clone()),
            name: Some("hello".into()),
            port: 8080,
            image: None,
            registry: None,
        };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        assert_eq!(execute(&options, &mut stdout, &mut stderr).unwrap(), 0);
        for name in [
            "app.wasm",
            "app.image.tar",
            "wasm-app-deployment.yml",
            "wasm-app-service.yml",
            "wasm-app-storage.yml",
        ] {
            assert!(out.join(name).is_file(), "missing {name}");
        }
        let deployment = fs::read_to_string(out.join("wasm-app-deployment.yml")).unwrap();
        assert!(deployment.contains("runtimeClassName: wasmtime"));
        assert!(deployment.contains("image: oci-registry:5000/hello:dev"));
        let tar = fs::read(out.join("app.image.tar")).unwrap();
        let tar_text = String::from_utf8_lossy(&tar);
        assert!(tar_text.contains("oci-layout"));
        assert!(tar_text.contains("index.json"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn without_output_errors() {
        let options = WasmOptions {
            wasm: "app.wasm".into(),
            dry_run: false,
            push_only: false,
            output: None,
            name: None,
            port: 8080,
            image: None,
            registry: None,
        };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        assert_eq!(execute(&options, &mut stdout, &mut stderr).unwrap(), 1);
        assert!(String::from_utf8_lossy(&stderr).contains("--output"));
    }
}
