//! Register the WASI class only after the connected runtime advertises its handler.
use std::path::PathBuf;
use std::time::Duration;

use diorama_apiserver::ApiserverError;
use diorama_apiserver::client::KubernetesApiClient;
use diorama_cri::runtime::v1::{StatusRequest, runtime_service_client::RuntimeServiceClient};
use diorama_supervisor::{
    Adapter, AdapterContext, AdapterError, AdapterFuture, ComponentKind, ComponentSpec,
    FailurePolicy, Registration,
};

struct RuntimeClass {
    socket: PathBuf,
    client: KubernetesApiClient,
}

pub(crate) fn registration(
    socket: PathBuf,
    client: KubernetesApiClient,
    prerequisites: Vec<String>,
    timeout: Duration,
) -> Registration {
    Registration::new(
        ComponentSpec {
            id: "runtime-classes".into(),
            prerequisites,
            kind: ComponentKind::OneShot,
            failure_policy: FailurePolicy::Degrade,
            startup_timeout: timeout,
        },
        RuntimeClass { socket, client },
    )
}

impl Adapter for RuntimeClass {
    fn run(self: Box<Self>, _context: AdapterContext) -> AdapterFuture {
        Box::pin(async move {
            let channel =
                diorama_cri::connect_unix(&self.socket)
                    .await
                    .map_err(|_| AdapterError {
                        code: "runtime-class-connect",
                    })?;
            let status = RuntimeServiceClient::new(channel)
                .status(StatusRequest { verbose: false })
                .await
                .map_err(|_| AdapterError {
                    code: "runtime-class-status",
                })?
                .into_inner();
            reconcile(&self.client, &status).await
        })
    }
}

async fn reconcile(
    client: &KubernetesApiClient,
    status: &diorama_cri::runtime::v1::StatusResponse,
) -> Result<(), AdapterError> {
    if !status.runtime_handlers.iter().any(|h| h.name == "wasmtime") {
        return Ok(());
    }
    match client
        .get_cluster_object("node.k8s.io", "runtimeclasses", "wasmtime")
        .await
    {
        Ok(_) => Ok(()),
        Err(ApiserverError::NotFound { .. }) => {
            match client
                .create_cluster_object(
                    "node.k8s.io",
                    "RuntimeClass",
                    "runtimeclasses",
                    serde_json::json!({
                        "apiVersion": "node.k8s.io/v1", "kind": "RuntimeClass",
                        "metadata": {"name": "wasmtime"}, "handler": "wasmtime"
                    }),
                )
                .await
            {
                Ok(_) | Err(ApiserverError::Conflict { .. }) => Ok(()),
                Err(_) => Err(AdapterError {
                    code: "runtime-class-create",
                }),
            }
        },
        Err(_) => Err(AdapterError {
            code: "runtime-class-read",
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diorama_apiserver::{ApiserverConfig, ApiserverService, KubernetesStorage};
    use diorama_cri::runtime::v1::{RuntimeHandler, StatusResponse};
    use diorama_datastore::{DatastoreConfig, DatastoreEngine};

    #[tokio::test]
    async fn creates_class_only_for_an_advertised_handler_and_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let (engine, _) =
            DatastoreEngine::open(DatastoreConfig::new(dir.path().join("db"))).unwrap();
        let service = ApiserverService::new(
            ApiserverConfig::default_for_pki(dir.path().join("pki"), "127.0.0.1".parse().unwrap()),
            KubernetesStorage::new(engine.client(), "/registry"),
        );
        let client = service.admin_client();
        let mut status = StatusResponse::default();
        reconcile(&client, &status).await.unwrap();
        assert!(matches!(
            client
                .get_cluster_object("node.k8s.io", "runtimeclasses", "wasmtime")
                .await,
            Err(ApiserverError::NotFound { .. })
        ));
        status.runtime_handlers.push(RuntimeHandler {
            name: "wasmtime".into(),
            ..Default::default()
        });
        reconcile(&client, &status).await.unwrap();
        let first = client
            .get_cluster_object("node.k8s.io", "runtimeclasses", "wasmtime")
            .await
            .unwrap();
        reconcile(&client, &status).await.unwrap();
        let second = client
            .get_cluster_object("node.k8s.io", "runtimeclasses", "wasmtime")
            .await
            .unwrap();
        assert_eq!(first, second);
    }
}
