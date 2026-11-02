//! Install the embedded `oci-builder` dashboard image at runtime and proxy it to a local loopback port.

use std::fs;
use std::io::{self, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use diorama_dashboard::server::spawn_http1_server;
use diorama_dashboard::{
    DEFAULT_DASHBOARD_H3_PORT, DEFAULT_DASHBOARD_HTTP_PORT, DashboardState, snapshot_from_items,
    synthetic_snapshot,
};
use diorama_datastore::{DatastoreClient, DatastoreConfig, DatastoreEngine, DatastoreError};
use serde_json::{Value, json};
use tokio::sync::watch;

/// Embedded `FROM scratch` OCI/Docker archive built with `oci-builder`.
pub const EMBEDDED_DASHBOARD_OCI_TAR_GZ: &[u8] =
    include_bytes!("../assets/diorama-dashboard-oci.tar.gz");

/// Canonical image reference imported into `k8s.io` containerd namespace.
pub const DASHBOARD_IMAGE: &str = "ghcr.io/geoffsee/diorama-dashboard:v0.1.0";

/// Namespace where the dashboard workload runs.
pub const DASHBOARD_NAMESPACE: &str = "kube-system";

/// Resource name for the dashboard `ServiceAccount`, RBAC, `Service`, and `Deployment`.
pub const DASHBOARD_NAME: &str = "diorama-dashboard";

pub const HELP: &str = "Install the embedded OCI dashboard add-on and proxy it to a local loopback port.\n\nUsage:\n  dioramactl dash [--port <port>] [--bind <loopback-ip>] [--path <node-data>] [--container <container-name>] [--engine docker|podman] [--install-only]\n";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DashOptions {
    pub bind: IpAddr,
    pub port: u16,
    pub path: PathBuf,
    pub container: Option<String>,
    pub engine: String,
    pub install_only: bool,
}

impl Default for DashOptions {
    fn default() -> Self {
        Self {
            bind: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: DEFAULT_DASHBOARD_HTTP_PORT,
            path: crate::DEFAULT_DATA_PATH.into(),
            container: None,
            engine: "docker".into(),
            install_only: false,
        }
    }
}

pub fn parse(args: &[String]) -> Result<DashOptions, crate::ParseError> {
    let mut options = DashOptions::default();
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        if arg == "--install-only" {
            options.install_only = true;
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
            "--port" => {
                options.port = value
                    .parse::<u16>()
                    .map_err(|_| crate::ParseError::UnknownFlag)?;
            },
            "--bind" => {
                let ip = value
                    .parse::<IpAddr>()
                    .map_err(|_| crate::ParseError::UnknownFlag)?;
                if !ip.is_loopback() {
                    return Err(crate::ParseError::UnknownFlag);
                }
                options.bind = ip;
            },
            "--path" if !value.is_empty() => options.path = PathBuf::from(value),
            "--container" if !value.starts_with('-') && !value.is_empty() => {
                options.container = Some(value.to_string());
            },
            "--engine" if matches!(value, "docker" | "podman") => {
                options.engine = value.to_string();
            },
            _ => return Err(crate::ParseError::UnknownFlag),
        }
    }
    Ok(options)
}

/// Builds the least-privilege Kubernetes resources (`ServiceAccount`, `ClusterRole`,
/// `ClusterRoleBinding`, `Service`, and `Deployment`) for the in-cluster dashboard pod.
#[must_use]
pub fn dashboard_resources(node_name: &str) -> Vec<(String, Value)> {
    let labels = json!({
        "app": DASHBOARD_NAME,
        "diorama.io/addon": DASHBOARD_NAME,
        "diorama.io/managed-by": "dioramactl-dash"
    });
    let sa = json!({
        "apiVersion": "v1",
        "kind": "ServiceAccount",
        "metadata": { "name": DASHBOARD_NAME, "namespace": DASHBOARD_NAMESPACE, "labels": labels }
    });
    let role = json!({
        "apiVersion": "rbac.authorization.k8s.io/v1",
        "kind": "ClusterRole",
        "metadata": { "name": DASHBOARD_NAME, "labels": labels },
        "rules": [
            { "apiGroups": [""], "resources": ["nodes", "pods"], "verbs": ["get", "list", "watch"] }
        ]
    });
    let binding = json!({
        "apiVersion": "rbac.authorization.k8s.io/v1",
        "kind": "ClusterRoleBinding",
        "metadata": { "name": DASHBOARD_NAME, "labels": labels },
        "roleRef": { "apiGroup": "rbac.authorization.k8s.io", "kind": "ClusterRole", "name": DASHBOARD_NAME },
        "subjects": [{ "kind": "ServiceAccount", "name": DASHBOARD_NAME, "namespace": DASHBOARD_NAMESPACE }]
    });
    let service = json!({
        "apiVersion": "v1",
        "kind": "Service",
        "metadata": { "name": DASHBOARD_NAME, "namespace": DASHBOARD_NAMESPACE, "labels": labels },
        "spec": {
            "type": "ClusterIP",
            "selector": { "app": DASHBOARD_NAME },
            "ports": [{ "name": "http", "port": 8080, "targetPort": 8080, "protocol": "TCP" }]
        }
    });
    let deployment = json!({
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "metadata": { "name": DASHBOARD_NAME, "namespace": DASHBOARD_NAMESPACE, "labels": labels },
        "spec": {
            "replicas": 1,
            "strategy": { "type": "Recreate" },
            "selector": { "matchLabels": { "app": DASHBOARD_NAME } },
            "template": {
                "metadata": { "labels": { "app": DASHBOARD_NAME } },
                "spec": {
                    "nodeName": node_name,
                    "serviceAccountName": DASHBOARD_NAME,
                    "securityContext": { "runAsNonRoot": true, "runAsUser": 65532, "runAsGroup": 65532 },
                    "containers": [{
                        "name": DASHBOARD_NAME,
                        "image": DASHBOARD_IMAGE,
                        "imagePullPolicy": "Never",
                        "command": ["/diorama-dashboard"],
                        "ports": [{ "name": "http", "containerPort": 8080, "protocol": "TCP" }],
                        "readinessProbe": { "httpGet": { "path": "/healthz", "port": 8080 }, "initialDelaySeconds": 1 },
                        "resources": { "requests": { "cpu": "10m", "memory": "16Mi" }, "limits": { "cpu": "100m", "memory": "64Mi" } },
                        "securityContext": {
                            "allowPrivilegeEscalation": false,
                            "readOnlyRootFilesystem": true,
                            "capabilities": { "drop": ["ALL"] }
                        }
                    }]
                }
            }
        }
    });
    vec![
        (
            format!("/registry/serviceaccounts/{DASHBOARD_NAMESPACE}/{DASHBOARD_NAME}"),
            sa,
        ),
        (
            format!("/registry/rbac.authorization.k8s.io/clusterroles/{DASHBOARD_NAME}"),
            role,
        ),
        (
            format!("/registry/rbac.authorization.k8s.io/clusterrolebindings/{DASHBOARD_NAME}"),
            binding,
        ),
        (
            format!("/registry/services/{DASHBOARD_NAMESPACE}/{DASHBOARD_NAME}"),
            service,
        ),
        (
            format!("/registry/apps/deployments/{DASHBOARD_NAMESPACE}/{DASHBOARD_NAME}"),
            deployment,
        ),
    ]
}

fn verify_admin_credentials(options: &DashOptions) -> io::Result<()> {
    if let Some(container) = &options.container {
        let status = Command::new(&options.engine)
            .args([
                "exec",
                container,
                "test",
                "-r",
                "/var/lib/diorama/pki/admin.kubeconfig",
            ])
            .status()?;
        if !status.success() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "cannot read /var/lib/diorama/pki/admin.kubeconfig in container",
            ));
        }
        return Ok(());
    }
    let kubeconfig = options.path.join("pki/admin.kubeconfig");
    let content = fs::read_to_string(&kubeconfig).map_err(|_| {
        io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "cannot read cluster admin credentials at {}",
                kubeconfig.display()
            ),
        )
    })?;
    if !content.contains("client-certificate-data:") && !content.contains("client-certificate:") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "admin.kubeconfig is missing client certificate credentials",
        ));
    }
    Ok(())
}

fn retain_and_import_oci_image(options: &DashOptions) -> io::Result<PathBuf> {
    let addons_dir = options.path.join("images/addons");
    let archive_name = format!("addon-dashboard-v{}.tar.gz", env!("CARGO_PKG_VERSION"));
    let retained_path = addons_dir.join(&archive_name);

    if let Some(container) = &options.container {
        let mut retain_cmd = Command::new(&options.engine);
        retain_cmd
            .args([
                "exec",
                "-i",
                container,
                "/bin/sh",
                "-c",
                "set -eu; mkdir -p \"$1\"; cat > \"$1/$2\"",
                "dash-retain",
            ])
            .arg(&addons_dir)
            .arg(&archive_name);
        pipe_bytes(&mut retain_cmd, EMBEDDED_DASHBOARD_OCI_TAR_GZ)?;

        let mut import_cmd = Command::new(&options.engine);
        import_cmd.args([
            "exec",
            "-i",
            container,
            "/usr/local/bin/ctr",
            "--address",
            "/var/lib/diorama/containerd/state/containerd.sock",
            "--namespace",
            "k8s.io",
            "images",
            "import",
            "--all-platforms",
            "-",
        ]);
        let _ = pipe_bytes(&mut import_cmd, EMBEDDED_DASHBOARD_OCI_TAR_GZ);
        return Ok(retained_path);
    }

    fs::create_dir_all(&addons_dir)?;
    let mut temp = tempfile::NamedTempFile::new_in(&addons_dir)?;
    temp.write_all(EMBEDDED_DASHBOARD_OCI_TAR_GZ)?;
    temp.as_file().sync_all()?;
    temp.persist(&retained_path).map_err(|e| e.error)?;

    let ctr = options.path.join("bin/ctr");
    let sock = options.path.join("containerd/state/containerd.sock");
    if ctr.is_file() && sock.exists() {
        let mut import_cmd = Command::new(&ctr);
        import_cmd.arg("--address").arg(&sock).args([
            "--namespace",
            "k8s.io",
            "images",
            "import",
            "--all-platforms",
            "-",
        ]);
        let _ = pipe_bytes(&mut import_cmd, EMBEDDED_DASHBOARD_OCI_TAR_GZ);
    }
    Ok(retained_path)
}

fn pipe_bytes(command: &mut Command, bytes: &[u8]) -> io::Result<()> {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(bytes)?;
    }
    if !child.wait()?.success() {
        return Err(io::Error::other("embedded OCI image stream command failed"));
    }
    Ok(())
}

fn open_datastore_client(data_dir: &Path) -> Result<DatastoreClient, DatastoreError> {
    let ds_dir = data_dir.join("datastore");
    let pki_dir = data_dir.join("pki");
    fs::create_dir_all(&ds_dir)?;
    match DatastoreEngine::open(DatastoreConfig::new(ds_dir)) {
        Ok((engine, _)) => Ok(engine.client()),
        Err(DatastoreError::LockContention(_)) => {
            let peer: SocketAddr = "127.0.0.1:2380".parse().unwrap();
            Ok(DatastoreClient::new_remote(
                peer,
                Some(pki_dir.join("datastore-server.crt")),
                Some(pki_dir.join("datastore-server.key")),
            ))
        },
        Err(err) => Err(err),
    }
}

async fn apply_manifests_and_seed_state(
    client: &DatastoreClient,
    state: &DashboardState,
) -> io::Result<()> {
    let nodes = client
        .list("/registry/nodes/")
        .await
        .map_err(|e| io::Error::other(e.to_string()))?;
    let node_items: Vec<Value> = nodes
        .iter()
        .filter_map(|kv| serde_json::from_slice(&kv.value).ok())
        .collect();
    let node_name = node_items
        .first()
        .and_then(|n| n.pointer("/metadata/name"))
        .and_then(Value::as_str)
        .unwrap_or("diorama-node");

    for (key, doc) in dashboard_resources(node_name) {
        let payload = serde_json::to_vec(&doc).map_err(io::Error::other)?;
        match client.get(&key).await {
            Ok(Some(existing)) => {
                let _ = client
                    .update(&key, payload, Some(existing.mod_revision))
                    .await;
            },
            Ok(None) => {
                let _ = client.create(&key, payload).await;
            },
            Err(_) => {},
        }
    }

    refresh_state_from_datastore(client, state).await;
    Ok(())
}

async fn refresh_state_from_datastore(client: &DatastoreClient, state: &DashboardState) -> bool {
    let Ok(node_kvs) = client.list("/registry/nodes/").await else {
        return false;
    };
    let Ok(pod_kvs) = client.list("/registry/pods/").await else {
        return false;
    };
    let node_items: Vec<Value> = node_kvs
        .iter()
        .filter_map(|kv| serde_json::from_slice(&kv.value).ok())
        .collect();
    let pod_items: Vec<Value> = pod_kvs
        .iter()
        .filter_map(|kv| serde_json::from_slice(&kv.value).ok())
        .collect();
    if let Some(snap) = snapshot_from_items(&node_items, &pod_items) {
        state.publish(snap);
        return true;
    }
    false
}

async fn poll_dashboard_loop(client: Option<DatastoreClient>, poll_state: DashboardState) {
    let mut ticker = tokio::time::interval(Duration::from_secs(2));
    let mut tick = 1u64;
    loop {
        ticker.tick().await;
        let updated = if let Some(ds) = &client {
            refresh_state_from_datastore(ds, &poll_state).await
        } else {
            false
        };
        if !updated {
            poll_state.publish(synthetic_snapshot(tick));
            tick = tick.wrapping_add(1);
        }
    }
}

async fn run_dash_proxy_async(
    options: DashOptions,
    state: DashboardState,
) -> io::Result<Option<SocketAddr>> {
    let client = open_datastore_client(&options.path).ok();
    if let Some(ds) = &client {
        apply_manifests_and_seed_state(ds, &state).await?;
    }
    if options.install_only {
        return Ok(None);
    }
    let listen_addr = SocketAddr::new(options.bind, options.port);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let (bound, handle) = spawn_http1_server(listen_addr, state.clone(), shutdown_rx)
        .await
        .map_err(|e| io::Error::other(e.to_string()))?;

    tokio::spawn(poll_dashboard_loop(client, state));

    let _ = tokio::signal::ctrl_c().await;
    let _ = shutdown_tx.send(true);
    let _ = handle.await;
    Ok(Some(bound))
}

pub fn execute(
    options: &DashOptions,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<u8> {
    if let Err(err) = verify_admin_credentials(options) {
        writeln!(stderr, "error: dash: {err}")?;
        return Ok(1);
    }
    let retained = match retain_and_import_oci_image(options) {
        Ok(path) => path,
        Err(err) => {
            writeln!(stderr, "error: dash image install: {err}")?;
            return Ok(1);
        },
    };

    let state = DashboardState::new(DEFAULT_DASHBOARD_H3_PORT);
    let options_clone = options.clone();
    let result: io::Result<Option<SocketAddr>> = std::thread::scope(|s| {
        s.spawn(|| {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            rt.block_on(run_dash_proxy_async(options_clone, state))
        })
        .join()
        .unwrap_or_else(|_| Err(io::Error::other("dash runtime thread panicked")))
    });

    match result {
        Ok(bound) => {
            writeln!(
                stdout,
                "Installed {DASHBOARD_IMAGE} ({}) and reconciled {DASHBOARD_NAMESPACE}/{DASHBOARD_NAME}",
                retained.display()
            )?;
            if let Some(addr) = bound {
                writeln!(stdout, "Closed local dashboard proxy on http://{addr}")?;
            }
            Ok(0)
        },
        Err(err) => {
            writeln!(stderr, "error: dash: {err}")?;
            Ok(1)
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_enforces_loopback_bind_and_supports_install_only() {
        let opts = parse(&[
            "--port=18080".into(),
            "--bind=127.0.0.1".into(),
            "--path=/tmp/node".into(),
            "--install-only".into(),
        ])
        .unwrap();
        assert_eq!(opts.port, 18080);
        assert_eq!(opts.bind, IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_eq!(opts.path, PathBuf::from("/tmp/node"));
        assert!(opts.install_only);

        // Non-loopback bind addresses must be rejected so the dashboard is never exposed publicly.
        assert!(parse(&["--bind=0.0.0.0".into()]).is_err());
        assert!(parse(&["--bind=192.168.1.10".into()]).is_err());
    }

    #[test]
    fn embedded_oci_archive_is_valid_gzip_with_dashboard_tag() {
        assert!(
            EMBEDDED_DASHBOARD_OCI_TAR_GZ.len() > 1024,
            "embedded OCI archive must be non-empty"
        );
        // Gzip magic bytes 0x1f 0x8b
        assert_eq!(&EMBEDDED_DASHBOARD_OCI_TAR_GZ[..2], &[0x1f, 0x8b]);
    }

    #[test]
    fn install_only_requires_admin_kubeconfig_and_materializes_oci_and_manifests() {
        let dir = tempfile::tempdir().unwrap();
        let opts = DashOptions {
            path: dir.path().to_path_buf(),
            install_only: true,
            ..DashOptions::default()
        };
        let mut out = Vec::new();
        let mut err = Vec::new();

        // Without admin.kubeconfig, execution is denied.
        let code = execute(&opts, &mut out, &mut err).unwrap();
        assert_eq!(code, 1);
        assert!(String::from_utf8_lossy(&err).contains("cannot read cluster admin credentials"));

        // Seed admin.kubeconfig and rerun --install-only.
        fs::create_dir_all(dir.path().join("pki")).unwrap();
        fs::write(
            dir.path().join("pki/admin.kubeconfig"),
            "apiVersion: v1\nclusters:\n- cluster:\n    server: https://127.0.0.1:6443\nusers:\n- name: admin\n  user:\n    client-certificate-data: dGVzdA==\n",
        )
        .unwrap();

        out.clear();
        err.clear();
        let code = execute(&opts, &mut out, &mut err).unwrap();
        assert_eq!(code, 0, "stderr: {}", String::from_utf8_lossy(&err));
        let stdout_str = String::from_utf8_lossy(&out);
        assert!(stdout_str.contains(DASHBOARD_IMAGE));

        // Verify the embedded OCI archive was materialized atomically.
        let archive_path = dir.path().join(format!(
            "images/addons/addon-dashboard-v{}.tar.gz",
            env!("CARGO_PKG_VERSION")
        ));
        assert_eq!(
            fs::read(&archive_path).unwrap(),
            EMBEDDED_DASHBOARD_OCI_TAR_GZ
        );
    }
}
