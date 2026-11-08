# oci-registry (WASI 0.3 command)

OCI Distribution `/v2/` server as an ordinary **`wasi:cli/run`** component.

- TCP sockets + filesystem only — **no WIT**, no `wasi:http/handler`, no custom host
- Same RuntimeClass as other wasip3 workloads: `runtimeClassName: wasmtime`
- Same source also builds as a native binary for smoke tests

## Native smoke

```sh
cd container/wasip3/oci-registry
cargo test
cargo run --release -- --bind 127.0.0.1:5000 --data /tmp/oci-reg
curl -i http://127.0.0.1:5000/v2/
python3 ../registry.py probe.tar native.tar http://127.0.0.1:5000
```

## Wasm build

```sh
rustup target add --toolchain nightly-2026-10-12 wasm32-wasip3
cargo build --release --target wasm32-wasip3
# artifact: target/wasm32-wasip3/release/oci-registry-wasip3.wasm
python3 ../image.py target/wasm32-wasip3/release/oci-registry-wasip3.wasm oci-registry.tar
```

## Deploy

See `oci-registry.yaml`. Mount a volume at `/data` for persistence.
