use std::fmt::Write as _;

const INDEX_TEMPLATE: &str = include_str!("../templates/index.html");
const NODE_TEMPLATE: &str = include_str!("../templates/node.html");
const PODS_TEMPLATE: &str = include_str!("../templates/pods.html");

/// Vendored HTMX 1.9.12 bundle served locally at `GET /htmx.min.js` with zero external CDN dependencies.
pub const HTMX_BUNDLE: &str = include_str!("../templates/htmx-1.9.12.bundle");

/// Aggregated CPU and memory gauge metrics for the local node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeMetrics {
    /// Kubernetes node name.
    pub name: String,
    /// Telemetry source badge label (e.g., `"apiserver live"` or `"synthetic fallback"`).
    pub source: String,
    /// Total requested/used CPU in millicores.
    pub cpu_used_m: u64,
    /// Allocatable node CPU in millicores.
    pub cpu_total_m: u64,
    /// CPU utilization percentage (`0..=100`).
    pub cpu_pct: u64,
    /// Total requested/used memory in MiB.
    pub mem_used_mib: u64,
    /// Allocatable node memory in MiB.
    pub mem_total_mib: u64,
    /// Memory utilization percentage (`0..=100`).
    pub mem_pct: u64,
}

/// Summary row for a single pod displayed in the dashboard table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PodRow {
    /// Pod name.
    pub name: String,
    /// Pod lifecycle phase (`Running`, `Pending`, `Succeeded`, `Failed`, etc.).
    pub phase: String,
    /// Total container restart count across the pod.
    pub restarts: i32,
}

/// Pre-rendered HTML fragments broadcast to connected HTTP/3 and HTTP/1.1 clients.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DashboardSnapshot {
    /// Rendered `<section id="node-panel">` HTML fragment.
    pub node_html: String,
    /// Rendered `<section id="pods-panel">` HTML fragment.
    pub pods_html: String,
}

impl DashboardSnapshot {
    /// Renders a snapshot from the current node metrics and pod rows.
    #[must_use]
    pub fn from_state(node: &NodeMetrics, pods: &[PodRow]) -> Self {
        Self {
            node_html: render_node_fragment(node),
            pods_html: render_pods_fragment(pods),
        }
    }

    /// Renders the complete HTML document shell embedding the current fragments.
    #[must_use]
    pub fn render_index(&self) -> String {
        INDEX_TEMPLATE
            .replace("{{NODE_HTML}}", &self.node_html)
            .replace("{{PODS_HTML}}", &self.pods_html)
    }

    /// Formats the snapshot as an SSE payload frame pair (`event: node` and `event: pods`).
    #[must_use]
    pub fn to_sse_frame(&self) -> String {
        let node_line = self.node_html.replace('\n', " ");
        let pods_line = self.pods_html.replace('\n', " ");
        format!("event: node\ndata: {node_line}\n\nevent: pods\ndata: {pods_line}\n\n")
    }
}

/// Renders the node gauge HTML fragment with escaped text fields.
#[must_use]
pub fn render_node_fragment(node: &NodeMetrics) -> String {
    NODE_TEMPLATE
        .replace("{{NODE_NAME}}", &escape_html(&node.name))
        .replace("{{NODE_SOURCE}}", &escape_html(&node.source))
        .replace("{{CPU_PCT}}", &node.cpu_pct.to_string())
        .replace("{{CPU_USED_M}}", &node.cpu_used_m.to_string())
        .replace("{{CPU_TOTAL_M}}", &node.cpu_total_m.to_string())
        .replace("{{MEM_PCT}}", &node.mem_pct.to_string())
        .replace("{{MEM_USED_MIB}}", &node.mem_used_mib.to_string())
        .replace("{{MEM_TOTAL_MIB}}", &node.mem_total_mib.to_string())
}

/// Renders the pod table HTML fragment with escaped text fields.
#[must_use]
pub fn render_pods_fragment(pods: &[PodRow]) -> String {
    let mut rows = String::new();
    for pod in pods {
        let name = escape_html(&pod.name);
        let phase = escape_html(&pod.phase);
        let phase_lower = phase.to_ascii_lowercase();
        let _ = writeln!(
            rows,
            "      <tr>\n        <td><code>{name}</code></td>\n        <td><span class=\"phase phase-{phase_lower}\">{phase}</span></td>\n        <td>{}</td>\n      </tr>",
            pod.restarts
        );
    }
    PODS_TEMPLATE
        .replace("{{POD_COUNT}}", &pods.len().to_string())
        .replace("{{POD_ROWS}}", &rows)
}

fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(ch),
        }
    }
    out
}
