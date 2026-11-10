//! Bootstrap the embedded wasip3 OCI registry after wasmtime is advertised.
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use diorama_apiserver::ApiserverError;
use diorama_apiserver::client::KubernetesApiClient;
use diorama_cri::runtime::v1::{
    ImageSpec, PullImageRequest, StatusRequest, image_service_client::ImageServiceClient,
    runtime_service_client::RuntimeServiceClient,
};
use diorama_supervisor::{
    Adapter, AdapterContext, AdapterError, AdapterFuture, ComponentKind, ComponentSpec,
    FailurePolicy, Registration,
};
use flate2::read::GzDecoder;

/// Gzipped OCI image archive packaged into the node binary.
pub(crate) const EMBEDDED_OCI_REGISTRY_OCI_TAR_GZ: &[u8] =
    include_bytes!("../assets/diorama-oci-registry-oci.tar.gz");

/// Image reference matching `container/wasip3/oci-registry/oci-registry.yaml`.
pub(crate) const OCI_REGISTRY_IMAGE: &str = "localhost/diorama-oci-registry:dev";

const COMPONENT_ID: &str = "oci-registry";
const ARCHIVE_FILE: &str = "diorama-oci-registry.tar";
const NAMESPACE: &str = "default";
const NAME: &str = "oci-registry";

struct OciRegistry {
    socket: PathBuf,
    state_dir: PathBuf,
    client: KubernetesApiClient,
}

pub(crate) fn registration(
    socket: PathBuf,
    state_dir: PathBuf,
    client: KubernetesApiClient,
    prerequisites: Vec<String>,
    timeout: Duration,
) -> Registration {
    Registration::new(
        ComponentSpec {
            id: COMPONENT_ID.into(),
            prerequisites,
            kind: ComponentKind::OneShot,
            failure_policy: FailurePolicy::Degrade,
            startup_timeout: timeout,
        },
        OciRegistry {
            socket,
            state_dir,
            client,
        },
    )
}

impl Adapter for OciRegistry {
    fn run(self: Box<Self>, _context: AdapterContext) -> AdapterFuture {
        Box::pin(async move {
            let channel =
                diorama_cri::connect_unix(&self.socket)
                    .await
                    .map_err(|_| AdapterError {
                        code: "oci-registry-connect",
                    })?;
            let status = RuntimeServiceClient::new(channel.clone())
                .status(StatusRequest { verbose: false })
                .await
                .map_err(|_| AdapterError {
                    code: "oci-registry-status",
                })?
                .into_inner();
            if !status.runtime_handlers.iter().any(|h| h.name == "wasmtime") {
                return Ok(());
            }
            let archive = materialize_archive(&self.state_dir).map_err(|_| AdapterError {
                code: "oci-registry-materialize",
            })?;
            pull_archive(channel, &archive, &self.state_dir).await?;
            reconcile_objects(&self.client).await
        })
    }
}

fn host_archive_path(state_dir: &Path) -> PathBuf {
    state_dir
        .join("oci-cri")
        .join("root")
        .join("images")
        .join(ARCHIVE_FILE)
}

/// Path the CRI engine can open for `oci-archive:`.
fn pull_image_ref(state_dir: &Path, host_archive: &Path) -> String {
    #[cfg(target_os = "macos")]
    {
        // Host root is virtiofs at /mnt/.host-root; an ext overlay covers /mnt/root.
        // Prefer the absolute host_share path (state_dir), which the guest remounts.
        let _ = host_archive;
        let abs = if state_dir.is_absolute() {
            state_dir.to_path_buf()
        } else {
            match std::env::current_dir() {
                Ok(cwd) => cwd.join(state_dir),
                Err(_) => state_dir.to_path_buf(),
            }
        };
        format!(
            "oci-archive:{}/oci-cri/root/images/{ARCHIVE_FILE}",
            abs.display()
        )
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = state_dir;
        format!("oci-archive:{}", host_archive.display())
    }
}

fn materialize_archive(state_dir: &Path) -> io::Result<PathBuf> {
    let dest = host_archive_path(state_dir);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut decoded = Vec::new();
    std::io::copy(
        &mut GzDecoder::new(EMBEDDED_OCI_REGISTRY_OCI_TAR_GZ),
        &mut decoded,
    )?;
    let temporary = dest.with_extension("tar.tmp");
    {
        let mut file = std::fs::File::create(&temporary)?;
        file.write_all(&decoded)?;
        file.sync_all()?;
    }
    std::fs::rename(&temporary, &dest)?;
    Ok(dest)
}

async fn pull_archive(
    channel: diorama_cri::tonic::transport::Channel,
    host_archive: &Path,
    state_dir: &Path,
) -> Result<(), AdapterError> {
    let image = pull_image_ref(state_dir, host_archive);
    ImageServiceClient::new(channel)
        .pull_image(PullImageRequest {
            image: Some(ImageSpec {
                image: image.clone(),
                annotations: BTreeMap::default(),
                user_specified_image: OCI_REGISTRY_IMAGE.into(),
                runtime_handler: "wasmtime".into(),
            }),
            auth: None,
            sandbox_config: None,
        })
        .await
        .map_err(|_| AdapterError {
            code: "oci-registry-pull",
        })?;
    Ok(())
}

fn registry_pod() -> serde_json::Value {
    serde_json::json!({
        "apiVersion": "v1",
        "kind": "Pod",
        "metadata": {
            "name": NAME,
            "namespace": NAMESPACE,
            "labels": {"app": NAME}
        },
        "spec": {
            "runtimeClassName": "wasmtime",
            "restartPolicy": "Always",
            "containers": [{
                "name": "registry",
                "image": OCI_REGISTRY_IMAGE,
                "imagePullPolicy": "Never",
                "ports": [{"name": "registry", "containerPort": 5000}],
                "env": [
                    {"name": "OCI_REGISTRY_BIND", "value": "0.0.0.0:5000"},
                    {"name": "OCI_REGISTRY_DATA", "value": "/data"}
                ],
                "volumeMounts": [{"name": "data", "mountPath": "/data"}],
                "resources": {
                    "limits": {"memory": "128Mi", "cpu": "500m"}
                }
            }],
            "volumes": [{"name": "data", "emptyDir": {}}]
        }
    })
}

fn registry_service() -> serde_json::Value {
    serde_json::json!({
        "apiVersion": "v1",
        "kind": "Service",
        "metadata": {
            "name": NAME,
            "namespace": NAMESPACE
        },
        "spec": {
            "selector": {"app": NAME},
            "ports": [{
                "name": "registry",
                "port": 5000,
                "targetPort": "registry"
            }]
        }
    })
}

async fn ensure_namespaced(
    get: impl std::future::Future<Output = Result<serde_json::Value, ApiserverError>>,
    create: impl std::future::Future<Output = Result<serde_json::Value, ApiserverError>>,
    read_code: &'static str,
    create_code: &'static str,
) -> Result<(), AdapterError> {
    match get.await {
        Ok(_) => Ok(()),
        Err(ApiserverError::NotFound { .. }) => match create.await {
            Ok(_) | Err(ApiserverError::Conflict { .. }) => Ok(()),
            Err(_) => Err(AdapterError { code: create_code }),
        },
        Err(_) => Err(AdapterError { code: read_code }),
    }
}

async fn reconcile_objects(client: &KubernetesApiClient) -> Result<(), AdapterError> {
    ensure_namespaced(
        client.get_pod(NAMESPACE, NAME),
        client.create_pod(NAMESPACE, registry_pod()),
        "oci-registry-pod-read",
        "oci-registry-pod-create",
    )
    .await?;
    ensure_namespaced(
        client.get_service(NAMESPACE, NAME),
        client.create_service(NAMESPACE, registry_service()),
        "oci-registry-service-read",
        "oci-registry-service-create",
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use diorama_apiserver::{ApiserverConfig, ApiserverService, KubernetesStorage};
    use diorama_datastore::{DatastoreConfig, DatastoreEngine};
    use flate2::read::GzDecoder;
    use std::io::Read;

    #[test]
    fn embedded_asset_is_gzip_with_registry_ref() {
        assert!(EMBEDDED_OCI_REGISTRY_OCI_TAR_GZ.len() > 1024);
        assert_eq!(&EMBEDDED_OCI_REGISTRY_OCI_TAR_GZ[..2], &[0x1f, 0x8b]);
        let mut decoded = Vec::new();
        GzDecoder::new(EMBEDDED_OCI_REGISTRY_OCI_TAR_GZ)
            .read_to_end(&mut decoded)
            .unwrap();
        let text = String::from_utf8_lossy(&decoded);
        assert!(
            text.contains(OCI_REGISTRY_IMAGE),
            "archive missing image ref annotation"
        );
    }

    #[test]
    fn materialize_writes_uncompressed_tar() {
        let dir = tempfile::tempdir().unwrap();
        let path = materialize_archive(dir.path()).unwrap();
        assert_eq!(path, host_archive_path(dir.path()));
        let bytes = std::fs::read(&path).unwrap();
        assert_ne!(&bytes[..2], &[0x1f, 0x8b]);
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains(OCI_REGISTRY_IMAGE));
        // Second write replaces atomically.
        let again = materialize_archive(dir.path()).unwrap();
        assert_eq!(again, path);
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }

    #[tokio::test]
    async fn creates_pod_and_service_idempotently() {
        let dir = tempfile::tempdir().unwrap();
        let (engine, _) =
            DatastoreEngine::open(DatastoreConfig::new(dir.path().join("db"))).unwrap();
        let service = ApiserverService::new(
            ApiserverConfig::default_for_pki(dir.path().join("pki"), "127.0.0.1".parse().unwrap()),
            KubernetesStorage::new(engine.client(), "/registry"),
        );
        let client = service.admin_client();
        client
            .create_cluster_object(
                "node.k8s.io",
                "RuntimeClass",
                "runtimeclasses",
                serde_json::json!({
                    "apiVersion": "node.k8s.io/v1",
                    "kind": "RuntimeClass",
                    "metadata": {"name": "wasmtime"},
                    "handler": "wasmtime"
                }),
            )
            .await
            .unwrap();
        reconcile_objects(&client).await.unwrap();
        let pod = client.get_pod(NAMESPACE, NAME).await.unwrap();
        assert_eq!(pod["spec"]["runtimeClassName"], "wasmtime");
        assert_eq!(pod["spec"]["containers"][0]["image"], OCI_REGISTRY_IMAGE);
        let service_doc = client.get_service(NAMESPACE, NAME).await.unwrap();
        assert_eq!(service_doc["spec"]["ports"][0]["port"], 5000);
        reconcile_objects(&client).await.unwrap();
        assert_eq!(
            client.get_pod(NAMESPACE, NAME).await.unwrap()["metadata"]["uid"],
            pod["metadata"]["uid"]
        );
    }
}
