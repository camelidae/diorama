//! macOS Linux VM runtime commands.

#[cfg(target_os = "macos")]
use std::io;
use std::path::{Path, PathBuf};

use crate::DEFAULT_DATA_PATH;

/// Help text for the `vm` management command.
pub const HELP: &str = "Manage the macOS Linux VM runtime.\n\nUsage:\n  dioramactl vm serve [--path DIR] [--socket PATH] [--pod-cidr CIDR]\n  dioramactl vm diagnose [--path DIR]\n";

/// A parsed VM management operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VmCommand {
    /// Boot and serve the embedded Linux CRI runtime.
    Serve {
        /// Diorama state directory shared with the Linux guest.
        path: PathBuf,
        /// Host-side CRI socket path, or `None` for the default under `path`.
        socket: Option<PathBuf>,
        /// Pod CIDR reported to the CRI runtime.
        pod_cidr: String,
    },
    /// Check whether this Mac can boot the embedded Linux guest.
    Diagnose {
        /// Diorama state directory used to locate guest state.
        path: PathBuf,
    },
}

/// Parse arguments following `dioramactl vm` without performing host I/O.
pub fn parse(args: &[String], default_path: &Path) -> Result<VmCommand, &'static str> {
    let Some(operation) = args.first().map(String::as_str) else {
        return Err(HELP);
    };
    let mut path = default_path.to_path_buf();
    let mut socket = None;
    let mut pod_cidr = String::new();
    let mut seen_path = false;
    let mut seen_socket = false;
    let mut seen_pod_cidr = false;
    let mut index = 1;

    while index < args.len() {
        let flag = args[index].as_str();
        let Some(value) = args.get(index + 1) else {
            return Err(HELP);
        };
        if value.starts_with("--") || value.is_empty() {
            return Err(HELP);
        }
        match (operation, flag) {
            ("serve" | "diagnose", "--path") if !seen_path => {
                path = PathBuf::from(value);
                seen_path = true;
            },
            ("serve", "--socket") if !seen_socket => {
                socket = Some(PathBuf::from(value));
                seen_socket = true;
            },
            ("serve", "--pod-cidr") if !seen_pod_cidr => {
                pod_cidr.clone_from(value);
                seen_pod_cidr = true;
            },
            _ => return Err(HELP),
        }
        index += 2;
    }

    if path.as_os_str().is_empty() {
        return Err(HELP);
    }
    match operation {
        "serve" => Ok(VmCommand::Serve {
            path,
            socket,
            pod_cidr,
        }),
        "diagnose" => Ok(VmCommand::Diagnose { path }),
        _ => Err(HELP),
    }
}

/// Execute arguments following `dioramactl vm`.
pub fn run(args: &[String]) -> Result<(), String> {
    let command = parse(args, &default_data_path()).map_err(str::to_owned)?;
    #[cfg(target_os = "macos")]
    {
        run_macos(&command)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = command;
        Err("the managed VM runtime is supported only on macOS".to_owned())
    }
}

fn default_data_path() -> PathBuf {
    if let Some(path) = std::env::var_os("DIORAMA_PATH") {
        return PathBuf::from(path);
    }
    #[cfg(target_os = "macos")]
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join("Library/Application Support/diorama");
    }
    PathBuf::from(DEFAULT_DATA_PATH)
}

#[cfg(target_os = "macos")]
fn build_config(command: &VmCommand) -> io::Result<oci_cri::Config> {
    let (path, socket, pod_cidr) = match command {
        VmCommand::Serve {
            path,
            socket,
            pod_cidr,
        } => (path, socket.as_deref(), pod_cidr.as_str()),
        VmCommand::Diagnose { path } => (path, None, ""),
    };
    let path = absolute_path(path)?;
    let base = path.join("oci-cri");
    let socket = socket.map_or_else(|| base.join("cri.sock"), Path::to_path_buf);
    let socket = absolute_path(&socket)?;
    Ok(oci_cri::Config {
        socket,
        root: base.join("root"),
        state_root: base.join("state"),
        host_shares: vec![path],
        pod_cidr: pod_cidr.to_owned(),
        // In-cluster oci-registry and host-forwarded registries speak plain HTTP.
        insecure_registries: true,
        ..oci_cri::Config::default()
    })
}

#[cfg(target_os = "macos")]
fn absolute_path(path: &Path) -> io::Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

#[cfg(target_os = "macos")]
fn run_macos(command: &VmCommand) -> Result<(), String> {
    let config = build_config(command).map_err(|error| error.to_string())?;
    oci_cri::startup().map_err(|error| error.to_string())?;
    match command {
        VmCommand::Serve { .. } => {
            oci_cri::server::serve(config).map_err(|error| error.to_string())
        },
        VmCommand::Diagnose { .. } => match oci_cri::diagnose(&config) {
            Ok(report) => {
                print!("{report}");
                Ok(())
            },
            Err(error) => {
                if !error.detail().is_empty() {
                    eprint!("{}", error.detail());
                }
                Err(error.message().to_owned())
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{VmCommand, parse};
    use std::path::Path;

    #[test]
    fn serve_options_preserve_spaces_and_use_the_configured_defaults() {
        let default_path = Path::new("/tmp/Diorama Data");
        let parsed = parse(
            &[
                "serve".to_owned(),
                "--path".to_owned(),
                "/tmp/State Directory".to_owned(),
                "--socket".to_owned(),
                "/tmp/State Directory/cri.sock".to_owned(),
                "--pod-cidr".to_owned(),
                "10.42.0.0/16".to_owned(),
            ],
            default_path,
        )
        .unwrap();
        assert_eq!(
            parsed,
            VmCommand::Serve {
                path: Path::new("/tmp/State Directory").to_path_buf(),
                socket: Some(Path::new("/tmp/State Directory/cri.sock").to_path_buf()),
                pod_cidr: "10.42.0.0/16".to_owned(),
            }
        );

        assert_eq!(
            parse(&["serve".to_owned()], default_path).unwrap(),
            VmCommand::Serve {
                path: default_path.to_path_buf(),
                socket: None,
                pod_cidr: String::new(),
            }
        );
    }

    #[test]
    fn diagnose_accepts_only_path_and_unknown_options_fail() {
        let default_path = Path::new("/tmp/default");
        assert_eq!(
            parse(&["diagnose".to_owned()], default_path).unwrap(),
            VmCommand::Diagnose {
                path: default_path.to_path_buf(),
            }
        );
        assert!(
            parse(
                &[
                    "diagnose".to_owned(),
                    "--socket".to_owned(),
                    "/tmp/cri.sock".to_owned()
                ],
                default_path
            )
            .is_err()
        );
        assert!(
            parse(
                &[
                    "serve".to_owned(),
                    "--path".to_owned(),
                    "/tmp/a".to_owned(),
                    "--path".to_owned(),
                    "/tmp/b".to_owned()
                ],
                default_path
            )
            .is_err()
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn config_uses_the_state_dir_for_vm_storage_and_host_share() {
        let config = super::build_config(&VmCommand::Serve {
            path: Path::new("/tmp/Diorama Data").to_path_buf(),
            socket: None,
            pod_cidr: "10.42.0.0/16".to_owned(),
        })
        .unwrap();
        assert_eq!(
            config.socket,
            Path::new("/tmp/Diorama Data/oci-cri/cri.sock")
        );
        assert_eq!(config.root, Path::new("/tmp/Diorama Data/oci-cri/root"));
        assert_eq!(
            config.state_root,
            Path::new("/tmp/Diorama Data/oci-cri/state")
        );
        assert_eq!(
            config.host_shares,
            [Path::new("/tmp/Diorama Data").to_path_buf()]
        );
        assert_eq!(config.pod_cidr, "10.42.0.0/16");
    }
}
