//! OCI Distribution registry as a WASI 0.3 **command** component.
//!
//! Speaks HTTP itself over TCP (`wasi:cli/run` + sockets + filesystem).
//! No WIT worlds, no `wasi:http/handler`, no custom capability host.
//! Deploy with `runtimeClassName: wasmtime` like any other wasip3 command.

mod blobstore;
mod http;
mod registry;
mod secrets;

use std::env;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;

use blobstore::Blobstore;
use http::App;
use secrets::Secrets;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Pod default: listen on all interfaces port 5000, persist under /data.
    let mut bind = env::var("OCI_REGISTRY_BIND").unwrap_or_else(|_| "0.0.0.0:5000".into());
    let mut data = env::var_os("OCI_REGISTRY_DATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/data"));

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bind" => {
                bind = args.next().ok_or("--bind needs a value")?;
            },
            "--data" => {
                data = PathBuf::from(args.next().ok_or("--data needs a value")?);
            },
            "-h" | "--help" => {
                print_help();
                return Ok(());
            },
            other => return Err(format!("unknown argument: {other}").into()),
        }
    }

    let app = App {
        store: Arc::new(Blobstore::open(data)?),
        secrets: Arc::new(Secrets::from_env()),
    };
    app.serve(bind)?;
    Ok(())
}

fn print_help() {
    eprintln!(
        "\
oci-registry-wasip3 — OCI Distribution registry (WASI 0.3 command)

Usage:
  oci-registry-wasip3 [--bind HOST:PORT] [--data DIR]

Env:
  OCI_REGISTRY_BIND       default bind (default: 0.0.0.0:5000)
  OCI_REGISTRY_DATA       blob store root (default: /data)
  OCI_REGISTRY_PASSWORD   if set, require HTTP Basic with this password

Build (Wasm):
  rustup target add --toolchain nightly-2026-10-12 wasm32-wasip3
  cargo build --release --target wasm32-wasip3

Native smoke:
  cargo run --release -- --bind 127.0.0.1:5000 --data /tmp/oci-reg
  curl -i http://127.0.0.1:5000/v2/
"
    );
}
