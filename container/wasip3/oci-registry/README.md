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

Cluster bring-up bootstraps this registry by default when the CRI advertises
`wasmtime`: the node binary embeds `localhost/diorama-oci-registry:dev`, imports
it via `oci-archive`, and creates the Pod + Service. No GHCR pull and no manual
`kubectl apply` are required for a normal start.

`oci-registry.yaml` remains the source-of-truth shape for that bootstrap (and for
standalone/dev rebuilds). Mount a volume at `/data` for persistence.

On the macOS VM, guest image pulls use the Apple NAT DNS (`192.168.64.1`), not CoreDNS. A pull of `oci-registry:5000/...` succeeds when a Ready sandbox is named `oci-registry` (or is the only Ready sandbox labeled `app` / `app.kubernetes.io/name` = `oci-registry`): oci-cri writes that pod IP into the guest `/etc/hosts`. The pod IP is on the guest CNI bridge, so the pull does not need ClusterIP or kube-proxy. Pushing from the Mac still needs an address the host can route, usually the same pod IP (`http://<pod-ip>:5000`), because the Mac is outside cluster DNS.

`dioramactl vm` sets `insecure_registries` so plain HTTP registries under `localhost`, `oci-registry`, and RFC1918 prefixes skip TLS. That flag is required for this registry.

`dioramactl wasm deploy` defaults to a Job (`restartPolicy: Never`). A command that exits 0 finishes Succeeded. Pass `--kind deployment` for a long-running server; that also writes a Service.

## Manual host push (before cluster apply)

With a local native registry (or `kubectl port-forward svc/oci-registry 5000:5000`):

```sh
cargo build -p dioramactl
cargo build --release --manifest-path container/wasip3/oci-registry/Cargo.toml
./container/wasip3/oci-registry/target/release/oci-registry-wasip3 \
  --bind 127.0.0.1:34567 --data /tmp/oci-reg &

./target/debug/dioramactl wasm deploy /tmp/probe.wasm \
  -o /tmp/diorama-wasm-live \
  --name probe \
  --image oci-registry:5000/probe:dev \
  --registry http://127.0.0.1:34567 \
  --push-only

curl -fsS http://127.0.0.1:34567/v2/probe/tags/list
```

Omit `--push-only` once the cluster is up to also `kubectl apply` the generated manifests. Ensure the in-cluster `oci-registry` Service is running so nodes can pull `oci-registry:5000/...`.
