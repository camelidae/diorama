#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Realtime HTTP/3 (`tokio-quiche`) + HTTP/1.1 + HTMX/SSE Kubernetes node dashboard
//! supervised in-process by `diorama`.
//!
//! Exposes four endpoints with zero client-side JavaScript state:
//! - `GET /` — Full HTML shell (`hx-sse="connect:/events"`)
//! - `GET /node` — CPU and memory `<meter>` gauge fragment
//! - `GET /pods` — Pod status table fragment
//! - `GET /events` — Server-Sent Events (`text/event-stream`) stream of `node` and `pods` HTML fragments

/// Live and synthetic telemetry collector for node and pod snapshots.
pub mod collector;
/// Configuration and error types for the node dashboard component.
pub mod config;
/// HTML fragment and SSE frame rendering.
pub mod render;
/// HTTP/3 (`tokio-quiche`) and HTTP/1.1 (`hyper`) listener implementations.
pub mod server;
/// In-process dashboard service lifecycle.
#[cfg(all(feature = "h3", feature = "apiserver"))]
pub mod service;
/// `diorama-supervisor` adapter (`COMPONENT_DASHBOARD = "dashboard"`).
#[cfg(all(feature = "h3", feature = "apiserver"))]
pub mod supervisor;

#[cfg(feature = "apiserver")]
pub use collector::snapshot_from_apiserver;
pub use collector::{
    DashboardState, parse_cpu_m, parse_mem_mib, run_synthetic_collector, snapshot_from_items,
    synthetic_snapshot,
};
pub use config::{
    DEFAULT_DASHBOARD_H3_PORT, DEFAULT_DASHBOARD_HTTP_PORT, DEFAULT_REFRESH_INTERVAL,
    DashboardConfig, DashboardError, Result,
};
pub use render::{
    DashboardSnapshot, NodeMetrics, PodRow, render_node_fragment, render_pods_fragment,
};
#[cfg(all(feature = "h3", feature = "apiserver"))]
pub use service::DashboardService;
#[cfg(all(feature = "h3", feature = "apiserver"))]
pub use supervisor::{COMPONENT_DASHBOARD, DEFAULT_STARTUP_TIMEOUT, DashboardAdapter};
