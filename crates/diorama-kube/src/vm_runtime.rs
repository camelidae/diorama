//! Supervisor adapter for the macOS `dioramactl vm serve` child process.

use std::io;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command as StdCommand, Stdio};
use std::time::Duration;

use diorama_cri::{CriEndpoint, RuntimeEndpoints, probe_cri_readiness};
use diorama_supervisor::{
    Adapter, AdapterContext, AdapterError, AdapterFuture, ComponentKind, ComponentSpec,
    FailurePolicy, Registration, StopPhase,
};
use tokio::process::{Child, Command};

const COMPONENT_OCI_CRI_VM: &str = "oci-cri-vm";
const READINESS_INTERVAL: Duration = Duration::from_millis(50);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub(crate) struct VmRuntimeService {
    state_dir: PathBuf,
    socket_path: PathBuf,
    pod_cidr: String,
    startup_timeout: Duration,
}

impl VmRuntimeService {
    pub(crate) fn registration(
        state_dir: PathBuf,
        socket_path: PathBuf,
        pod_cidr: String,
        startup_timeout: Duration,
    ) -> Registration {
        let spec = ComponentSpec {
            id: COMPONENT_OCI_CRI_VM.to_owned(),
            prerequisites: Vec::new(),
            kind: ComponentKind::LongRunning,
            failure_policy: FailurePolicy::Fatal,
            startup_timeout,
        };
        Registration::new(
            spec,
            Self {
                state_dir,
                socket_path,
                pod_cidr,
                startup_timeout,
            },
        )
    }
}

impl Adapter for VmRuntimeService {
    fn run(self: Box<Self>, mut context: AdapterContext) -> AdapterFuture {
        Box::pin(async move {
            if context.stop_phase() != StopPhase::Running {
                return Ok(());
            }
            let executable = locate_dioramactl().map_err(|error| {
                eprintln!("could not locate dioramactl: {error}");
                AdapterError {
                    code: "oci_cri_vm_helper_missing",
                }
            })?;
            let mut command = Command::new(&executable);
            command
                .args(["vm", "serve", "--path"])
                .arg(&self.state_dir)
                .args(["--socket"])
                .arg(&self.socket_path)
                .args(["--pod-cidr", &self.pod_cidr])
                .stdin(Stdio::null())
                .kill_on_drop(true);
            command.as_std_mut().process_group(0);
            let mut child = command.spawn().map_err(|error| {
                eprintln!("failed to start {} vm serve: {error}", executable.display());
                AdapterError {
                    code: "oci_cri_vm_helper_start_failed",
                }
            })?;

            let endpoint = CriEndpoint::from_path(&self.socket_path).map_err(|_| AdapterError {
                code: "oci_cri_vm_invalid_socket",
            })?;
            let endpoints = RuntimeEndpoints::single(endpoint);
            let readiness =
                probe_cri_readiness(&endpoints, self.startup_timeout, READINESS_INTERVAL);
            tokio::pin!(readiness);
            let mut ready = false;
            loop {
                tokio::select! {
                    status = child.wait() => {
                        if let Ok(status) = status {
                            eprintln!("dioramactl vm serve exited: {status}");
                        }
                        return Err(AdapterError { code: "oci_cri_vm_helper_exited" });
                    }
                    result = &mut readiness, if !ready => {
                        if result.is_err() {
                            stop_child(&mut child).await;
                            return Err(AdapterError { code: "oci_cri_vm_readiness_failed" });
                        }
                        ready = true;
                        context.ready();
                    }
                    phase = context.changed() => {
                        if phase != StopPhase::Running {
                            stop_child(&mut child).await;
                            return Ok(());
                        }
                    }
                }
            }
        })
    }
}

fn locate_dioramactl() -> io::Result<PathBuf> {
    let current_exe = std::env::current_exe()?;
    locate_dioramactl_from(&current_exe, std::env::var_os("PATH").as_deref())
}

fn locate_dioramactl_from(
    current_exe: &Path,
    path: Option<&std::ffi::OsStr>,
) -> io::Result<PathBuf> {
    let sibling = current_exe.parent().map(|parent| parent.join("dioramactl"));
    if let Some(sibling) = &sibling
        && sibling.is_file()
    {
        return Ok(sibling.clone());
    }
    if let Some(path) = path {
        for directory in std::env::split_paths(path) {
            let candidate = directory.join("dioramactl");
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }
    let sibling = sibling.map_or_else(
        || "<no executable directory>".into(),
        |path| path.display().to_string(),
    );
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!(
            "expected sibling {sibling} or dioramactl on PATH; build/install both macOS release binaries together"
        ),
    ))
}

async fn stop_child(child: &mut Child) {
    if let Some(pid) = child.id() {
        let _ = StdCommand::new("/bin/kill")
            .args(["-TERM", &format!("-{pid}")])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    if tokio::time::timeout(SHUTDOWN_TIMEOUT, child.wait())
        .await
        .is_err()
    {
        let _ = child.start_kill();
        let _ = child.wait().await;
    }
}

#[cfg(test)]
mod tests {
    use super::locate_dioramactl_from;
    use std::path::Path;

    #[test]
    fn locates_sibling_before_searching_path() {
        let directory = tempfile::tempdir().unwrap();
        let sibling = directory.path().join("dioramactl");
        std::fs::write(&sibling, []).unwrap();
        let path_directory = tempfile::tempdir().unwrap();
        let path_helper = path_directory.path().join("dioramactl");
        std::fs::write(&path_helper, []).unwrap();
        let current_exe = directory.path().join("diorama");
        assert_eq!(
            locate_dioramactl_from(&current_exe, Some(path_directory.path().as_os_str())).unwrap(),
            sibling
        );
    }

    #[test]
    fn falls_back_to_path_when_no_sibling_exists() {
        let current_directory = tempfile::tempdir().unwrap();
        let path_directory = tempfile::tempdir().unwrap();
        let helper = path_directory.path().join("dioramactl");
        std::fs::write(&helper, []).unwrap();
        assert_eq!(
            locate_dioramactl_from(
                &current_directory.path().join("diorama"),
                Some(path_directory.path().as_os_str()),
            )
            .unwrap(),
            helper
        );
    }

    #[test]
    fn missing_helper_error_describes_both_search_locations() {
        let directory = tempfile::tempdir().unwrap();
        let error = locate_dioramactl_from(
            &directory.path().join("diorama"),
            Some(Path::new("/no/such/path").as_os_str()),
        )
        .unwrap_err();
        assert!(error.to_string().contains("sibling"));
        assert!(error.to_string().contains("PATH"));
    }
}
