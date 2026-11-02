use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

#[cfg(feature = "apiserver")]
use diorama_apiserver::client::KubernetesApiClient;
use serde_json::Value;
use tokio::sync::{broadcast, watch};

use crate::render::{DashboardSnapshot, NodeMetrics, PodRow};

/// Shared reactive state between the collector loop and HTTP/3 + HTTP/1.1 handlers.
#[derive(Clone, Debug)]
pub struct DashboardState {
    snapshot: Arc<RwLock<DashboardSnapshot>>,
    tx: broadcast::Sender<DashboardSnapshot>,
    h3_port: Arc<AtomicU16>,
}

impl DashboardState {
    /// Creates a new shared dashboard state seeded with the initial synthetic snapshot.
    #[must_use]
    pub fn new(h3_port: u16) -> Self {
        let (tx, _) = broadcast::channel(64);
        Self {
            snapshot: Arc::new(RwLock::new(synthetic_snapshot(0))),
            tx,
            h3_port: Arc::new(AtomicU16::new(h3_port)),
        }
    }

    /// Returns the most recently rendered snapshot.
    #[must_use]
    pub fn current_snapshot(&self) -> DashboardSnapshot {
        self.snapshot
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Subscribes to live snapshot updates for SSE streaming.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<DashboardSnapshot> {
        self.tx.subscribe()
    }

    /// Returns the UDP port advertised in `Alt-Svc` headers.
    #[must_use]
    pub fn h3_port(&self) -> u16 {
        self.h3_port.load(Ordering::Relaxed)
    }

    /// Updates the bound UDP port advertised in `Alt-Svc` headers.
    pub fn set_h3_port(&self, port: u16) {
        self.h3_port.store(port, Ordering::Relaxed);
    }

    /// Publishes a newly rendered snapshot to in-memory state and all SSE subscribers.
    pub fn publish(&self, snap: DashboardSnapshot) {
        *self
            .snapshot
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = snap.clone();
        let _ = self.tx.send(snap);
    }
}

/// Runs the background collector loop, watching the in-process `KubernetesApiClient`
/// when available and falling back to synthetic node/pod telemetry otherwise.
#[cfg(feature = "apiserver")]
pub async fn run_collector(
    state: DashboardState,
    client: Option<Arc<KubernetesApiClient>>,
    refresh_interval: Duration,
    mut shutdown: watch::Receiver<bool>,
) {
    if let Some(api) = client {
        if let Some(snap) = snapshot_from_apiserver(&api).await {
            state.publish(snap);
        }
        watch_apiserver_loop(state, api, refresh_interval, &mut shutdown).await;
    } else {
        synthetic_loop(state, refresh_interval, &mut shutdown).await;
    }
}

/// Runs the standalone synthetic/in-cluster telemetry refresh loop.
pub async fn run_synthetic_collector(
    state: DashboardState,
    refresh_interval: Duration,
    mut shutdown: watch::Receiver<bool>,
) {
    synthetic_loop(state, refresh_interval, &mut shutdown).await;
}

#[cfg(feature = "apiserver")]
async fn watch_apiserver_loop(
    state: DashboardState,
    api: Arc<KubernetesApiClient>,
    refresh_interval: Duration,
    shutdown: &mut watch::Receiver<bool>,
) {
    let mut node_watch = api.watch("nodes").await.ok();
    let mut pod_watch = api.watch("pods").await.ok();
    let mut ticker = tokio::time::interval(refresh_interval);
    let mut tick = 1u64;

    loop {
        tokio::select! {
            biased;
            _ = shutdown.changed() => {
                if *shutdown.borrow() {
                    break;
                }
            }
            Some(Ok(_)) = async {
                match node_watch.as_mut() {
                    Some(rx) => Some(rx.recv().await),
                    None => None,
                }
            } => {}
            Some(Ok(_)) = async {
                match pod_watch.as_mut() {
                    Some(rx) => Some(rx.recv().await),
                    None => None,
                }
            } => {}
            _ = ticker.tick() => {
                tick = tick.wrapping_add(1);
            }
        }
        let next = snapshot_from_apiserver(&api)
            .await
            .unwrap_or_else(|| synthetic_snapshot(tick));
        state.publish(next);
    }
}

async fn synthetic_loop(
    state: DashboardState,
    refresh_interval: Duration,
    shutdown: &mut watch::Receiver<bool>,
) {
    let mut ticker = tokio::time::interval(refresh_interval);
    let mut tick = 1u64;
    loop {
        tokio::select! {
            biased;
            _ = shutdown.changed() => {
                if *shutdown.borrow() {
                    break;
                }
            }
            _ = ticker.tick() => {
                state.publish(synthetic_snapshot(tick));
                tick = tick.wrapping_add(1);
            }
        }
    }
}

/// Queries the in-process `KubernetesApiClient` for live nodes and pods.
#[cfg(feature = "apiserver")]
pub async fn snapshot_from_apiserver(api: &KubernetesApiClient) -> Option<DashboardSnapshot> {
    let nodes_doc = api.list_nodes().await.ok()?;
    let pods_doc = api.list_all_pods().await.ok()?;
    let node_items = nodes_doc.get("items")?.as_array()?;
    let pod_items = pods_doc.get("items")?.as_array()?;
    snapshot_from_items(node_items, pod_items)
}

/// Builds a `DashboardSnapshot` from raw Kubernetes Node and Pod JSON objects.
#[must_use]
pub fn snapshot_from_items(node_items: &[Value], pod_items: &[Value]) -> Option<DashboardSnapshot> {
    if node_items.is_empty() && pod_items.is_empty() {
        return None;
    }

    let (node_name, cpu_total_m, mem_total_mib) = node_items
        .first()
        .map_or_else(|| ("diorama-node".to_string(), 4000, 8192), parse_node_item);

    let mut rows = Vec::with_capacity(pod_items.len().min(12));
    let mut cpu_used_m = 0u64;
    let mut mem_used_mib = 0u64;
    for item in pod_items {
        let (row, cpu_req, mem_req) = parse_pod_item(item);
        cpu_used_m = cpu_used_m.saturating_add(cpu_req);
        mem_used_mib = mem_used_mib.saturating_add(mem_req);
        if rows.len() < 12 {
            rows.push(row);
        }
    }

    let node = NodeMetrics {
        name: node_name,
        source: "apiserver live".to_string(),
        cpu_used_m,
        cpu_total_m,
        cpu_pct: ((cpu_used_m * 100) / cpu_total_m.max(1)).min(100),
        mem_used_mib,
        mem_total_mib,
        mem_pct: ((mem_used_mib * 100) / mem_total_mib.max(1)).min(100),
    };
    Some(DashboardSnapshot::from_state(&node, &rows))
}

fn parse_node_item(node: &Value) -> (String, u64, u64) {
    let name = node
        .pointer("/metadata/name")
        .and_then(Value::as_str)
        .unwrap_or("diorama-node")
        .to_string();
    let alloc = node
        .pointer("/status/allocatable")
        .or_else(|| node.pointer("/status/capacity"));
    let cpu_m = alloc
        .and_then(|a| a.get("cpu"))
        .and_then(Value::as_str)
        .map_or(4000, parse_cpu_m)
        .max(100);
    let mem_mib = alloc
        .and_then(|a| a.get("memory"))
        .and_then(Value::as_str)
        .map_or(8192, parse_mem_mib)
        .max(128);
    (name, cpu_m, mem_mib)
}

fn parse_pod_item(pod: &Value) -> (PodRow, u64, u64) {
    let name = pod
        .pointer("/metadata/name")
        .and_then(Value::as_str)
        .unwrap_or("pod")
        .to_string();
    let phase = pod
        .pointer("/status/phase")
        .and_then(Value::as_str)
        .unwrap_or("Pending")
        .to_string();
    let restarts = pod
        .pointer("/status/containerStatuses")
        .and_then(Value::as_array)
        .map_or(0, |statuses| {
            statuses
                .iter()
                .filter_map(|s| s.get("restartCount").and_then(Value::as_i64))
                .filter_map(|v| i32::try_from(v).ok())
                .sum()
        });
    let (cpu_req, mem_req) = pod
        .pointer("/spec/containers")
        .and_then(Value::as_array)
        .map_or((50, 64), |containers| {
            containers.iter().fold((0u64, 0u64), |(c_acc, m_acc), c| {
                let cpu = c
                    .pointer("/resources/requests/cpu")
                    .and_then(Value::as_str)
                    .map_or(50, parse_cpu_m);
                let mem = c
                    .pointer("/resources/requests/memory")
                    .and_then(Value::as_str)
                    .map_or(64, parse_mem_mib);
                (c_acc.saturating_add(cpu), m_acc.saturating_add(mem))
            })
        });
    (
        PodRow {
            name,
            phase,
            restarts,
        },
        cpu_req,
        mem_req,
    )
}

/// Parses a Kubernetes CPU quantity string (e.g. `"250m"`, `"4"`) into millicores.
#[must_use]
pub fn parse_cpu_m(raw: &str) -> u64 {
    if let Some(milli) = raw.strip_suffix('m') {
        milli.parse::<u64>().unwrap_or(1000)
    } else {
        raw.parse::<u64>()
            .map_or(1000, |cores| cores.saturating_mul(1000))
    }
}

/// Parses a Kubernetes memory quantity string (e.g. `"8192Mi"`, `"8Gi"`, `"1048576Ki"`) into MiB.
#[must_use]
pub fn parse_mem_mib(raw: &str) -> u64 {
    if let Some(ki) = raw.strip_suffix("Ki") {
        ki.parse::<u64>().unwrap_or(0) / 1024
    } else if let Some(mi) = raw.strip_suffix("Mi") {
        mi.parse::<u64>().unwrap_or(256)
    } else if let Some(gi) = raw.strip_suffix("Gi") {
        gi.parse::<u64>().unwrap_or(1).saturating_mul(1024)
    } else {
        raw.parse::<u64>().unwrap_or(0) / (1024 * 1024)
    }
}

/// Generates a deterministic synthetic snapshot for fallback mode.
#[must_use]
pub fn synthetic_snapshot(tick: u64) -> DashboardSnapshot {
    let cpu_used_m = 620 + ((tick * 130) % 1800);
    let mem_used_mib = 1480 + ((tick * 190) % 2600);
    let node = NodeMetrics {
        name: "diorama-single-node".to_string(),
        source: "synthetic fallback".to_string(),
        cpu_used_m,
        cpu_total_m: 4000,
        cpu_pct: (cpu_used_m * 100) / 4000,
        mem_used_mib,
        mem_total_mib: 8192,
        mem_pct: (mem_used_mib * 100) / 8192,
    };
    let restarts = i32::try_from((tick / 12) % 100).unwrap_or(0);
    let pods = vec![
        PodRow {
            name: "coredns-7db6d8ff4d-x9k2p".to_string(),
            phase: "Running".to_string(),
            restarts: 0,
        },
        PodRow {
            name: "local-path-provisioner-5b9f".to_string(),
            phase: "Running".to_string(),
            restarts: 0,
        },
        PodRow {
            name: "kube-proxy-diorama-node".to_string(),
            phase: "Running".to_string(),
            restarts: 1,
        },
        PodRow {
            name: "edge-telemetry-job-88a1".to_string(),
            phase: if tick.is_multiple_of(5) {
                "Pending".to_string()
            } else {
                "Running".to_string()
            },
            restarts,
        },
    ];
    DashboardSnapshot::from_state(&node, &pods)
}
