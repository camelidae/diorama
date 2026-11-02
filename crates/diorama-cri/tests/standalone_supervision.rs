//! Integration tests for standalone oci-cri daemon lifecycle supervision,
//! stale socket detection and cleanup, active listener rejection, and reverse-dependency
//! shutdown ordering.

use diorama_cri::runtime::v1::{ListImagesResponse, VersionResponse};
use diorama_cri::standalone::{
    COMPONENT_STANDALONE_OCI_CRI, StandaloneOciCriError, StandaloneOciCriOptions,
    StandaloneOciCriService, StandaloneReadinessMode, prepare_standalone_socket,
};
use diorama_supervisor::{
    Adapter, AdapterContext, AdapterFuture, ComponentKind, ComponentSpec, FailurePolicy,
    Registration, StopPhase, Supervisor, stop_channel,
};
use h2::server;
use prost::Message;
use prost::bytes::{BufMut, Bytes, BytesMut};
use std::fs;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tempfile::TempDir;
use tokio::net::UnixListener;
use tonic::codegen::http::{HeaderMap, Response, StatusCode};

fn serialize_grpc<M: Message>(msg: &M) -> Bytes {
    let raw = msg.encode_to_vec();
    let mut out = BytesMut::with_capacity(5 + raw.len());
    out.put_u8(0);
    let len = u32::try_from(raw.len()).expect("grpc frame length fits u32");
    out.put_u32(len);
    out.put_slice(&raw);
    out.freeze()
}

fn send_grpc_reply(mut send: server::SendResponse<Bytes>, data: Bytes) {
    let header = Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/grpc")
        .body(())
        .unwrap();
    let Ok(mut stream) = send.send_response(header, false) else {
        return;
    };
    let _ = stream.send_data(data, false);
    let mut trailers = HeaderMap::new();
    trailers.insert("grpc-status", "0".parse().unwrap());
    let _ = stream.send_trailers(trailers);
}

async fn handle_h2_connection(
    stream: tokio::net::UnixStream,
    ver: Bytes,
    img: Bytes,
    stopped: Arc<AtomicBool>,
) {
    let Ok(mut h2_conn) = server::handshake(stream).await else {
        return;
    };
    while let Some(Ok((req, send))) = h2_conn.accept().await {
        if stopped.load(Ordering::Relaxed) {
            break;
        }
        let payload = match req.uri().path() {
            p if p.contains("Version") => ver.clone(),
            p if p.contains("ListImages") => img.clone(),
            _ => continue,
        };
        send_grpc_reply(send, payload);
    }
}

fn spawn_mock_cri_listener(listener: UnixListener, stopped: Arc<AtomicBool>) {
    let ver_reply = serialize_grpc(&VersionResponse {
        version: "0.1.0".into(),
        runtime_name: "oci-cri".into(),
        runtime_version: "1.0.0".into(),
        runtime_api_version: "v1".into(),
    });
    let img_reply = serialize_grpc(&ListImagesResponse { images: vec![] });

    tokio::spawn(async move {
        while !stopped.load(Ordering::Relaxed) {
            let Ok(Ok((stream, _))) =
                tokio::time::timeout(Duration::from_millis(250), listener.accept()).await
            else {
                continue;
            };

            let ver = ver_reply.clone();
            let img = img_reply.clone();
            let stopped_conn = Arc::clone(&stopped);

            tokio::spawn(handle_h2_connection(stream, ver, img, stopped_conn));
        }
    });
}

/// Helper child process invoked as a mock `oci-cri serve` daemon by tests.
#[tokio::test]
#[ignore = "invoked only as a child process by standalone supervision integration tests"]
async fn mock_oci_cri_child() {
    let socket_path =
        std::env::var("MOCK_OCI_CRI_SOCKET").expect("MOCK_OCI_CRI_SOCKET env var required");
    let listener = UnixListener::bind(&socket_path).expect("bind socket in mock child");

    if let Ok(ready_marker) = std::env::var("MOCK_READY_MARKER") {
        fs::write(ready_marker, b"ready").expect("write ready marker");
    }

    let stopped = Arc::new(AtomicBool::new(false));
    let serve_cri = std::env::var("MOCK_SERVE_CRI").as_deref() == Ok("1");

    if serve_cri {
        spawn_mock_cri_listener(listener, Arc::clone(&stopped));
    }

    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("install sigterm handler");
    sigterm.recv().await;

    stopped.store(true, Ordering::Relaxed);

    if let Ok(kubelet_drain_marker) = std::env::var("MOCK_EXPECT_KUBELET_DRAINED") {
        assert!(
            Path::new(&kubelet_drain_marker).exists(),
            "kubelet must have drained before oci-cri receives SIGTERM"
        );
    }

    if let Ok(term_marker) = std::env::var("MOCK_TERM_MARKER") {
        fs::write(term_marker, b"terminated").expect("write term marker");
    }
}

fn create_wrapper_script(dir: &Path) -> PathBuf {
    let script_path = dir.join("mock_oci_cri_wrapper.sh");
    let script_content = r#"#!/bin/sh
export MOCK_OCI_CRI_SOCKET="$3"
sh -c 'trap "" TERM; sleep 300' &
exec "$MOCK_TEST_BINARY" --exact standalone_supervision::mock_oci_cri_child --ignored
"#;
    fs::write(&script_path, script_content).expect("write wrapper script");
    fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755))
        .expect("set wrapper executable");
    script_path
}

// =========================================================================
// Requirement 1: Stale Unix socket detection & cleanup vs active listener
// rejection vs non-socket file/symlink rejection.
// =========================================================================

#[tokio::test]
async fn prepare_socket_rejects_regular_file_without_unlinking() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("regular.sock");
    fs::write(&file_path, b"do-not-delete").unwrap();

    let err = prepare_standalone_socket(&file_path).await.unwrap_err();
    match err {
        StandaloneOciCriError::NonSocketFile(p) => assert_eq!(p, file_path),
        other => panic!("expected NonSocketFile, got {other:?}"),
    }

    assert!(file_path.exists());
    assert_eq!(fs::read(&file_path).unwrap(), b"do-not-delete");
}

#[tokio::test]
async fn prepare_socket_rejects_directory_without_unlinking() {
    let dir = TempDir::new().unwrap();
    let dir_path = dir.path().join("directory.sock");
    fs::create_dir(&dir_path).unwrap();

    let err = prepare_standalone_socket(&dir_path).await.unwrap_err();
    match err {
        StandaloneOciCriError::NonSocketFile(p) => assert_eq!(p, dir_path),
        other => panic!("expected NonSocketFile, got {other:?}"),
    }

    assert!(dir_path.is_dir());
}

#[tokio::test]
async fn prepare_socket_rejects_symlink_without_unlinking() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("target-file");
    fs::write(&target, b"target-content").unwrap();
    let link = dir.path().join("symlink.sock");
    std::os::unix::fs::symlink(&target, &link).unwrap();

    let err = prepare_standalone_socket(&link).await.unwrap_err();
    match err {
        StandaloneOciCriError::NonSocketFile(p) => assert_eq!(p, link),
        other => panic!("expected NonSocketFile, got {other:?}"),
    }

    assert!(link.symlink_metadata().is_ok());
    assert!(target.exists());
}

#[tokio::test]
async fn prepare_socket_rejects_symlink_to_socket_without_unlinking() {
    let dir = TempDir::new().unwrap();
    let real_sock = dir.path().join("real.sock");
    let _listener = UnixListener::bind(&real_sock).unwrap();
    let link = dir.path().join("socket_symlink.sock");
    std::os::unix::fs::symlink(&real_sock, &link).unwrap();

    let err = prepare_standalone_socket(&link).await.unwrap_err();
    match err {
        StandaloneOciCriError::NonSocketFile(p) => assert_eq!(p, link),
        other => panic!("expected NonSocketFile, got {other:?}"),
    }

    assert!(link.symlink_metadata().is_ok());
    assert!(real_sock.exists());
}

#[tokio::test]
async fn prepare_socket_rejects_active_listener_without_unlinking() {
    let dir = TempDir::new().unwrap();
    let sock_path = dir.path().join("active.sock");
    let _listener = UnixListener::bind(&sock_path).unwrap();

    let err = prepare_standalone_socket(&sock_path).await.unwrap_err();
    match err {
        StandaloneOciCriError::SocketAlreadyActive(p) => assert_eq!(p, sock_path),
        other => panic!("expected SocketAlreadyActive, got {other:?}"),
    }

    assert!(sock_path.exists());
}

#[tokio::test]
async fn prepare_socket_reclaims_stale_socket_from_crashed_run() {
    let dir = TempDir::new().unwrap();
    let sock_path = dir.path().join("stale.sock");

    // Create a stale socket inode by binding and immediately dropping the listener
    {
        let _listener = UnixListener::bind(&sock_path).unwrap();
    }
    assert!(sock_path.exists());
    assert!(
        fs::symlink_metadata(&sock_path)
            .unwrap()
            .file_type()
            .is_socket()
    );

    let res = prepare_standalone_socket(&sock_path).await;
    assert!(res.is_ok(), "failed to reclaim stale socket: {res:?}");
    assert!(!sock_path.exists(), "stale socket must be safely removed");
}

#[tokio::test]
async fn prepare_socket_non_existent_succeeds_and_ensures_parent() {
    let dir = TempDir::new().unwrap();
    let nested = dir.path().join("a").join("b").join("socket.sock");
    assert!(!nested.parent().unwrap().exists());

    let res = prepare_standalone_socket(&nested).await;
    assert!(res.is_ok());
    assert!(nested.parent().unwrap().exists());
    assert!(!nested.exists());
}

// =========================================================================
// Requirement 2: Supervised child process termination on parent/supervisor stop
// (ProcessCleanupSnapshot::complete()) and socket file removal on clean exit.
// =========================================================================

#[tokio::test]
async fn supervised_child_process_terminates_and_socket_is_removed_on_clean_exit() {
    let dir = TempDir::new().unwrap();
    let sock_path = dir.path().join("oci-cri.sock");
    let ready_marker = dir.path().join("child_ready");
    let term_marker = dir.path().join("child_term");
    let script_path = create_wrapper_script(dir.path());
    let current_exe = std::env::current_exe().unwrap();

    let options = StandaloneOciCriOptions::new(&sock_path)
        .with_binary(&script_path)
        .with_env("MOCK_TEST_BINARY", current_exe.to_str().unwrap())
        .with_env("MOCK_READY_MARKER", ready_marker.to_str().unwrap())
        .with_env("MOCK_TERM_MARKER", term_marker.to_str().unwrap())
        .with_readiness_mode(StandaloneReadinessMode::SocketConnect)
        .with_readiness_timeout(Duration::from_secs(5))
        .with_retry_interval(Duration::from_millis(20))
        .with_startup_timeout(Duration::from_secs(10));

    let service = StandaloneOciCriService::new(options);
    let cleanup = service.cleanup();

    let supervisor = Supervisor::new(vec![Registration::new(
        StandaloneOciCriService::component_spec(Duration::from_secs(10)),
        service,
    )])
    .expect("supervisor graph registration");

    let (stop_sender, stop_receiver) = stop_channel();
    let sup_task = tokio::spawn(supervisor.run(stop_receiver));

    // Wait until child is running and socket is live
    let mut ready = false;
    for _ in 0..100 {
        if ready_marker.exists() && sock_path.exists() {
            ready = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(ready, "mock child daemon failed to become ready");

    // Initiate graceful supervisor shutdown
    stop_sender.stop();

    let report = tokio::time::timeout(Duration::from_secs(10), sup_task)
        .await
        .expect("supervisor shutdown deadline")
        .expect("join supervisor task");

    assert!(
        report.failures.is_empty(),
        "supervisor failures: {report:?}"
    );

    // Verify ProcessCleanupSnapshot::complete()
    let snapshot = cleanup.wait(Duration::from_secs(3)).await;
    assert!(
        snapshot.complete(),
        "process cleanup incomplete: {snapshot:?}"
    );
    assert!(snapshot.spawned);
    assert!(snapshot.leader_reaped);
    assert!(snapshot.term_attempted);
    assert!(!snapshot.ownership_lost);

    // Verify child observed SIGTERM and executed clean teardown
    assert!(term_marker.exists(), "child must have received SIGTERM");

    // Verify socket file was cleanly unlinked upon service exit
    assert!(
        !sock_path.exists(),
        "socket file must be removed on clean exit"
    );
}

#[tokio::test]
async fn supervised_child_cleans_stale_socket_from_previous_crash_before_spawning() {
    let dir = TempDir::new().unwrap();
    let sock_path = dir.path().join("crashed.sock");

    // Simulate stale socket from crashed run
    {
        let _listener = UnixListener::bind(&sock_path).unwrap();
    }
    assert!(sock_path.exists());

    let ready_marker = dir.path().join("crashed_ready");
    let script_path = create_wrapper_script(dir.path());
    let current_exe = std::env::current_exe().unwrap();

    let options = StandaloneOciCriOptions::new(&sock_path)
        .with_binary(&script_path)
        .with_env("MOCK_TEST_BINARY", current_exe.to_str().unwrap())
        .with_env("MOCK_READY_MARKER", ready_marker.to_str().unwrap())
        .with_readiness_mode(StandaloneReadinessMode::SocketConnect)
        .with_readiness_timeout(Duration::from_secs(5))
        .with_retry_interval(Duration::from_millis(20));

    let service = StandaloneOciCriService::new(options);
    let cleanup = service.cleanup();

    let supervisor = Supervisor::new(vec![Registration::new(service.spec(), service)]).unwrap();
    let (stop_sender, stop_receiver) = stop_channel();
    let sup_task = tokio::spawn(supervisor.run(stop_receiver));

    // Wait until child is running
    for _ in 0..100 {
        if ready_marker.exists() && sock_path.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    stop_sender.stop();
    let report = sup_task.await.unwrap();
    assert!(report.failures.is_empty(), "{report:?}");

    let snapshot = cleanup.wait(Duration::from_secs(3)).await;
    assert!(snapshot.complete(), "{snapshot:?}");
    assert!(!sock_path.exists(), "socket must be unlinked on exit");
}

// =========================================================================
// Requirement 3: Reverse-dependency shutdown ordering with mock kubelet
// depending on COMPONENT_STANDALONE_OCI_CRI.
// =========================================================================

struct MockKubeletAdapter {
    events: Arc<std::sync::Mutex<Vec<String>>>,
    ready_marker: PathBuf,
    drained_marker: PathBuf,
}

impl Adapter for MockKubeletAdapter {
    fn run(self: Box<Self>, mut context: AdapterContext) -> AdapterFuture {
        Box::pin(async move {
            fs::write(&self.ready_marker, b"kubelet_ready").unwrap();
            context.ready();
            while context.changed().await == StopPhase::Running {}

            // Shutdown triggered: drain workloads before exiting
            self.events
                .lock()
                .unwrap()
                .push("kubelet_drain_started".into());
            tokio::time::sleep(Duration::from_millis(100)).await;
            fs::write(&self.drained_marker, b"kubelet_drained").unwrap();
            self.events
                .lock()
                .unwrap()
                .push("kubelet_drain_completed".into());
            Ok(())
        })
    }
}

#[tokio::test]
async fn reverse_dependency_shutdown_ordering_drains_kubelet_before_stopping_oci_cri() {
    let dir = TempDir::new().unwrap();
    let sock_path = dir.path().join("ordered-oci-cri.sock");
    let cri_ready_marker = dir.path().join("cri_ready");
    let kubelet_ready_marker = dir.path().join("kubelet_ready");
    let drained_marker = dir.path().join("kubelet_drained");
    let term_marker = dir.path().join("cri_term");
    let script_path = create_wrapper_script(dir.path());
    let current_exe = std::env::current_exe().unwrap();

    let events = Arc::new(std::sync::Mutex::new(Vec::new()));

    let cri_options = StandaloneOciCriOptions::new(&sock_path)
        .with_binary(&script_path)
        .with_env("MOCK_TEST_BINARY", current_exe.to_str().unwrap())
        .with_env("MOCK_READY_MARKER", cri_ready_marker.to_str().unwrap())
        .with_env("MOCK_TERM_MARKER", term_marker.to_str().unwrap())
        .with_env(
            "MOCK_EXPECT_KUBELET_DRAINED",
            drained_marker.to_str().unwrap(),
        )
        .with_readiness_mode(StandaloneReadinessMode::SocketConnect)
        .with_readiness_timeout(Duration::from_secs(5))
        .with_retry_interval(Duration::from_millis(20));

    let cri_service = StandaloneOciCriService::new(cri_options);
    let cri_cleanup = cri_service.cleanup();

    let kubelet_adapter = MockKubeletAdapter {
        events: Arc::clone(&events),
        ready_marker: kubelet_ready_marker.clone(),
        drained_marker: drained_marker.clone(),
    };

    let kubelet_spec = ComponentSpec {
        id: "kubelet".into(),
        prerequisites: vec![COMPONENT_STANDALONE_OCI_CRI.into()],
        kind: ComponentKind::LongRunning,
        failure_policy: FailurePolicy::Fatal,
        startup_timeout: Duration::from_secs(10),
    };

    let supervisor = Supervisor::new(vec![
        Registration::new(
            StandaloneOciCriService::component_spec(Duration::from_secs(10)),
            cri_service,
        ),
        Registration::new(kubelet_spec, kubelet_adapter),
    ])
    .expect("supervisor graph registration");

    let (stop_sender, stop_receiver) = stop_channel();
    let sup_task = tokio::spawn(supervisor.run(stop_receiver));

    // Wait until both standalone-oci-cri and kubelet are live and ready
    for _ in 0..100 {
        if cri_ready_marker.exists() && kubelet_ready_marker.exists() && sock_path.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(
        kubelet_ready_marker.exists(),
        "kubelet must have started and become ready"
    );

    // Stop supervisor: shutdown occurs in reverse topological order (kubelet first, then oci-cri)
    stop_sender.stop();

    let report = tokio::time::timeout(Duration::from_secs(10), sup_task)
        .await
        .expect("supervisor shutdown deadline")
        .expect("join supervisor task");

    assert!(
        report.failures.is_empty(),
        "supervisor failures: {report:?}"
    );

    // Assert that kubelet drain marker exists
    assert!(
        drained_marker.exists(),
        "kubelet must have drained before termination completed"
    );

    // Assert that mock oci-cri verified kubelet was drained upon SIGTERM receipt
    assert!(
        term_marker.exists(),
        "oci-cri must have observed kubelet drained prior to terminating"
    );

    let recorded = events.lock().unwrap().clone();
    assert_eq!(
        recorded,
        vec!["kubelet_drain_started", "kubelet_drain_completed"],
        "kubelet drain sequence was executed"
    );

    let snapshot = cri_cleanup.wait(Duration::from_secs(3)).await;
    assert!(
        snapshot.complete(),
        "oci-cri cleanup incomplete: {snapshot:?}"
    );
    assert!(!sock_path.exists(), "socket must be unlinked on clean exit");
}

#[tokio::test]
async fn supervised_child_cri_rpc_readiness_probe_succeeds() {
    let dir = TempDir::new().unwrap();
    let sock_path = dir.path().join("grpc-oci-cri.sock");
    let ready_marker = dir.path().join("grpc_ready");
    let script_path = create_wrapper_script(dir.path());
    let current_exe = std::env::current_exe().unwrap();

    let options = StandaloneOciCriOptions::new(&sock_path)
        .with_binary(&script_path)
        .with_env("MOCK_TEST_BINARY", current_exe.to_str().unwrap())
        .with_env("MOCK_READY_MARKER", ready_marker.to_str().unwrap())
        .with_env("MOCK_SERVE_CRI", "1")
        .with_readiness_mode(StandaloneReadinessMode::CriRpc)
        .with_readiness_timeout(Duration::from_secs(5))
        .with_retry_interval(Duration::from_millis(50));

    let service = StandaloneOciCriService::new(options);
    let cleanup = service.cleanup();

    let supervisor = Supervisor::new(vec![Registration::new(service.spec(), service)]).unwrap();
    let (stop_sender, stop_receiver) = stop_channel();
    let sup_task = tokio::spawn(supervisor.run(stop_receiver));

    // Wait until socket is live and probed
    for _ in 0..100 {
        if ready_marker.exists() && sock_path.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    tokio::time::sleep(Duration::from_millis(200)).await;
    stop_sender.stop();

    let report = sup_task.await.unwrap();
    assert!(report.failures.is_empty(), "{report:?}");

    let snapshot = cleanup.wait(Duration::from_secs(3)).await;
    assert!(snapshot.complete(), "{snapshot:?}");
    assert!(!sock_path.exists(), "socket removed on exit");
}
