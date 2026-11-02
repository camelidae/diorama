//! Supervised standalone oci-cri daemon lifecycle adapter.
//!
//! Spawns and supervises `oci-cri serve --socket <path>` in an owned process group
//! managed by `diorama-supervisor`, guarantees parent-death and supervisor shutdown
//! signal propagation, reclaims stale socket inodes safely, and unlinks sockets on clean exit.

use std::fmt;
use std::fs;
use std::os::unix::fs::FileTypeExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use diorama_supervisor::process::{
    OutputLimit, OwnedProcessAdapter, ProcessCleanup, ProcessCommand, ProcessOutput,
};
use diorama_supervisor::{
    Adapter, AdapterContext, AdapterError, AdapterFuture, ComponentKind, ComponentSpec,
    FailurePolicy, StopPhase,
};

use crate::endpoint::{CriEndpoint, RuntimeEndpoints};
use crate::readiness::{DEFAULT_READINESS_TIMEOUT, DEFAULT_RETRY_INTERVAL, probe_cri_readiness};

pub const COMPONENT_STANDALONE_OCI_CRI: &str = "standalone-oci-cri";

const SOCKET_CONNECT_TIMEOUT: Duration = Duration::from_millis(500);

/// Typed errors encountered during standalone oci-cri preparation and lifecycle management.
#[derive(Debug)]
pub enum StandaloneOciCriError {
    /// Target path is not a Unix domain socket (regular file, directory, or symlink).
    NonSocketFile(PathBuf),
    /// A live listener is already accepting connections on the socket.
    SocketAlreadyActive(PathBuf),
    /// Filesystem or I/O error while inspecting or unlinking the socket.
    Io(std::io::Error),
    /// Readiness probing failed within the configured deadline.
    ReadinessFailed,
    /// Failed to start or configure the supervised process.
    ProcessStartFailed(&'static str),
}

impl StandaloneOciCriError {
    #[must_use]
    pub fn adapter_code(&self) -> &'static str {
        match self {
            Self::NonSocketFile(_) => "socket_path_not_a_socket",
            Self::SocketAlreadyActive(_) => "socket_already_active",
            Self::Io(_) => "socket_io_error",
            Self::ReadinessFailed => "standalone_oci_cri_readiness_failed",
            Self::ProcessStartFailed(msg) => msg,
        }
    }
}

impl fmt::Display for StandaloneOciCriError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonSocketFile(path) => {
                write!(
                    f,
                    "refusing to remove non-socket file at '{}'",
                    path.display()
                )
            },
            Self::SocketAlreadyActive(path) => {
                write!(
                    f,
                    "live listener already active on socket '{}'",
                    path.display()
                )
            },
            Self::Io(err) => write!(f, "I/O error preparing socket: {err}"),
            Self::ReadinessFailed => write!(f, "standalone oci-cri readiness probe failed"),
            Self::ProcessStartFailed(msg) => write!(f, "process start failed: {msg}"),
        }
    }
}

impl std::error::Error for StandaloneOciCriError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for StandaloneOciCriError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

/// Prepares the filesystem path for a new standalone `oci-cri` instance.
///
/// Refuses regular files, directories, symlinks, or live listeners without unlinking them.
/// Reclaims and safely unlinks stale Unix domain socket files left behind from crashed runs.
pub async fn prepare_standalone_socket(socket_path: &Path) -> Result<(), StandaloneOciCriError> {
    match fs::symlink_metadata(socket_path) {
        Ok(meta) => {
            // Refuse symlinks, regular files, and directories.
            // Notice: symlink_metadata does not follow symlinks, so a symlink returns
            // is_symlink() == true and is_socket() == false.
            if !meta.file_type().is_socket() {
                return Err(StandaloneOciCriError::NonSocketFile(
                    socket_path.to_path_buf(),
                ));
            }

            // Probe whether a live listener is active on this Unix domain socket.
            let connect_result = tokio::time::timeout(
                SOCKET_CONNECT_TIMEOUT,
                tokio::net::UnixStream::connect(socket_path),
            )
            .await;

            match connect_result {
                Ok(Ok(_stream)) => {
                    // Live listener accepted the connection. Do not overwrite or unlink.
                    Err(StandaloneOciCriError::SocketAlreadyActive(
                        socket_path.to_path_buf(),
                    ))
                },
                Ok(Err(e))
                    if e.kind() == std::io::ErrorKind::ConnectionRefused
                        || e.kind() == std::io::ErrorKind::NotFound =>
                {
                    // Stale socket from a crashed or terminated run; safe to reclaim.
                    fs::remove_file(socket_path)?;
                    Ok(())
                },
                Ok(Err(e)) => Err(StandaloneOciCriError::Io(e)),
                Err(_) => {
                    // Timeout connecting to socket: socket is held open or busy, reject.
                    Err(StandaloneOciCriError::SocketAlreadyActive(
                        socket_path.to_path_buf(),
                    ))
                },
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            if let Some(parent) = socket_path.parent()
                && !parent.as_os_str().is_empty()
                && !parent.exists()
            {
                fs::create_dir_all(parent)?;
            }
            Ok(())
        },
        Err(e) => Err(StandaloneOciCriError::Io(e)),
    }
}

/// Readiness probing mode for standalone oci-cri.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StandaloneReadinessMode {
    /// Full CRI v1 RPC readiness verification (`RuntimeService` and `ImageService`).
    #[default]
    CriRpc,
    /// Unix domain socket connection verification (accepts stream connection).
    SocketConnect,
    /// Immediate readiness upon process spawn (used for lifecycle / shutdown testing).
    Immediate,
}

/// Configuration options for supervising a standalone `oci-cri` daemon.
#[derive(Clone, Debug)]
pub struct StandaloneOciCriOptions {
    pub binary_path: PathBuf,
    pub socket_path: PathBuf,
    pub extra_args: Vec<String>,
    pub envs: Vec<(String, String)>,
    pub readiness_timeout: Duration,
    pub retry_interval: Duration,
    pub startup_timeout: Duration,
    pub readiness_mode: StandaloneReadinessMode,
    pub output_limit: Option<OutputLimit>,
}

impl StandaloneOciCriOptions {
    pub fn new(socket_path: impl Into<PathBuf>) -> Self {
        Self {
            binary_path: PathBuf::from("oci-cri"),
            socket_path: socket_path.into(),
            extra_args: Vec::new(),
            envs: Vec::new(),
            readiness_timeout: DEFAULT_READINESS_TIMEOUT,
            retry_interval: DEFAULT_RETRY_INTERVAL,
            startup_timeout: Duration::from_secs(15),
            readiness_mode: StandaloneReadinessMode::default(),
            output_limit: None,
        }
    }

    #[must_use]
    pub fn with_binary(mut self, binary: impl Into<PathBuf>) -> Self {
        self.binary_path = binary.into();
        self
    }

    #[must_use]
    pub fn with_extra_args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.extra_args = args.into_iter().map(Into::into).collect();
        self
    }

    #[must_use]
    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        self.extra_args.push(arg.into());
        self
    }

    #[must_use]
    pub fn with_env(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.envs.push((key.into(), val.into()));
        self
    }

    #[must_use]
    pub fn with_readiness_timeout(mut self, timeout: Duration) -> Self {
        self.readiness_timeout = timeout;
        self
    }

    #[must_use]
    pub fn with_retry_interval(mut self, interval: Duration) -> Self {
        self.retry_interval = interval;
        self
    }

    #[must_use]
    pub fn with_startup_timeout(mut self, timeout: Duration) -> Self {
        self.startup_timeout = timeout;
        self
    }

    #[must_use]
    pub fn with_readiness_mode(mut self, mode: StandaloneReadinessMode) -> Self {
        self.readiness_mode = mode;
        self
    }

    #[must_use]
    pub fn with_output_limit(mut self, limit: OutputLimit) -> Self {
        self.output_limit = Some(limit);
        self
    }
}

/// Supervised service adapter for a standalone `oci-cri` daemon.
pub struct StandaloneOciCriService {
    options: StandaloneOciCriOptions,
    cleanup: ProcessCleanup,
    adapter: Option<OwnedProcessAdapter>,
    output: Option<ProcessOutput>,
}

impl fmt::Debug for StandaloneOciCriService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StandaloneOciCriService")
            .field("socket_path", &self.options.socket_path)
            .field("cleanup", &self.cleanup)
            .finish_non_exhaustive()
    }
}

impl StandaloneOciCriService {
    pub fn new(options: StandaloneOciCriOptions) -> Self {
        let mut command = ProcessCommand::new(&options.binary_path);
        command = command
            .arg("serve")
            .arg("--socket")
            .arg(&options.socket_path);
        for arg in &options.extra_args {
            command = command.arg(arg);
        }
        for (k, v) in &options.envs {
            command = command.env(k, v);
        }

        let readiness_options = options.clone();
        let readiness = async move { run_readiness_probe(&readiness_options).await };

        if let Some(limit) = options.output_limit {
            let (adapter, cleanup, output) =
                OwnedProcessAdapter::new_with_bounded_output(command, readiness, limit);
            Self {
                options,
                cleanup,
                adapter: Some(adapter),
                output: Some(output),
            }
        } else {
            let (adapter, cleanup) = OwnedProcessAdapter::new(command, readiness);
            Self {
                options,
                cleanup,
                adapter: Some(adapter),
                output: None,
            }
        }
    }

    #[must_use]
    pub fn options(&self) -> &StandaloneOciCriOptions {
        &self.options
    }

    #[must_use]
    pub fn cleanup(&self) -> ProcessCleanup {
        self.cleanup.clone()
    }

    #[must_use]
    pub fn output(&self) -> Option<ProcessOutput> {
        self.output.clone()
    }

    pub fn component_spec(startup_timeout: Duration) -> ComponentSpec {
        ComponentSpec {
            id: COMPONENT_STANDALONE_OCI_CRI.into(),
            prerequisites: Vec::new(),
            kind: ComponentKind::LongRunning,
            failure_policy: FailurePolicy::Fatal,
            startup_timeout,
        }
    }

    #[must_use]
    pub fn spec(&self) -> ComponentSpec {
        Self::component_spec(self.options.startup_timeout)
    }
}

async fn run_readiness_probe(options: &StandaloneOciCriOptions) -> Result<(), AdapterError> {
    match options.readiness_mode {
        StandaloneReadinessMode::Immediate => Ok(()),
        StandaloneReadinessMode::SocketConnect => {
            let deadline = tokio::time::Instant::now() + options.readiness_timeout;
            while tokio::time::Instant::now() < deadline {
                if tokio::net::UnixStream::connect(&options.socket_path)
                    .await
                    .is_ok()
                {
                    return Ok(());
                }
                tokio::time::sleep(options.retry_interval).await;
            }
            Err(AdapterError {
                code: "standalone_oci_cri_socket_timeout",
            })
        },
        StandaloneReadinessMode::CriRpc => {
            let abs_path = if options.socket_path.is_absolute() {
                options.socket_path.clone()
            } else {
                std::env::current_dir()
                    .unwrap_or_default()
                    .join(&options.socket_path)
            };
            let endpoint = CriEndpoint::from_path(&abs_path).map_err(|_| AdapterError {
                code: "standalone_oci_cri_endpoint_invalid",
            })?;
            let endpoints = RuntimeEndpoints::single(endpoint);
            probe_cri_readiness(
                &endpoints,
                options.readiness_timeout,
                options.retry_interval,
            )
            .await
            .map_err(|_| AdapterError {
                code: "standalone_oci_cri_readiness_failed",
            })?;
            Ok(())
        },
    }
}

fn remove_socket_if_exists(path: &Path) {
    if let Ok(meta) = fs::symlink_metadata(path)
        && meta.file_type().is_socket()
    {
        let _ = fs::remove_file(path);
    }
}

impl Adapter for StandaloneOciCriService {
    fn run(mut self: Box<Self>, context: AdapterContext) -> AdapterFuture {
        Box::pin(async move {
            if context.stop_phase() != StopPhase::Running {
                return Ok(());
            }

            // Detect and safely handle stale sockets before starting child process.
            prepare_standalone_socket(&self.options.socket_path)
                .await
                .map_err(|e| AdapterError {
                    code: e.adapter_code(),
                })?;

            let adapter = self.adapter.take().ok_or(AdapterError {
                code: "standalone_oci_cri_already_run",
            })?;

            let result = Box::new(adapter).run(context).await;

            // Wait for process cleanup to finish and reap
            let _ = self.cleanup.wait(Duration::from_millis(500)).await;

            // Clean up socket file on clean exit
            if result.is_ok() {
                remove_socket_if_exists(&self.options.socket_path);
            }

            result
        })
    }
}
