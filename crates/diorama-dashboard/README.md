# `diorama-dashboard`

In-process realtime Kubernetes node dashboard supervised by `diorama` (`NodeRuntime` component `dashboard`).

## Architecture

- **Transport**: Dual-stack HTTP/3 over QUIC (`tokio-quiche` `ServerH3Driver`) and HTTP/1.1 TCP fallback (`hyper` + `hyper-util`), advertising `Alt-Svc: h3=":<port>"; ma=86400`.
- **Frontend**: Pure HTMX (`hx-sse="connect:/events"`, `hx-sse="swap:node"`, `hx-sse="swap:pods"`, and `hx-trigger="every 2s"` fallback) with compiled-in HTML templates (`templates/{index.html,node.html,pods.html}`). Zero application state in the browser.
- **Control-Plane Integration**: Watches the in-process `KubernetesApiClient` (`list_nodes`, `list_all_pods`, `watch("nodes")`, `watch("pods")`) and falls back to deterministic synthetic node/pod metrics when no cluster objects are present.
- **Supervision**: Registered as `COMPONENT_DASHBOARD` (`"dashboard"`) with `FailurePolicy::Degrade` after `apiserver`, reusing `/var/lib/diorama/pki/apiserver.{crt,key}` when present or generating an ephemeral self-signed certificate via `rcgen`.

## Environment Configuration

- `DIORAMA_DASHBOARD_ENABLED=1` — Enables the supervised dashboard component in `NodeRuntime`.
- `DIORAMA_DASHBOARD_HTTP_ADDR=127.0.0.1:8080` — TCP HTTP/1.1 bind address.
- `DIORAMA_DASHBOARD_H3_ADDR=127.0.0.1:8443` — UDP HTTP/3 (QUIC) bind address.
