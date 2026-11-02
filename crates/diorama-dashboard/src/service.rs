use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use diorama_apiserver::client::KubernetesApiClient;
use tokio::sync::watch;

use crate::collector::{DashboardState, run_collector};
use crate::config::{DashboardConfig, DashboardError, Result};
use crate::server::{spawn_h3_server, spawn_http1_server};

/// Supervised realtime HTTP/3 + HTTP/1.1 + HTMX node dashboard service.
#[derive(Clone, Debug)]
pub struct DashboardService {
    config: DashboardConfig,
    client: Option<Arc<KubernetesApiClient>>,
    state: DashboardState,
    bound_http_addr: Arc<Mutex<Option<SocketAddr>>>,
    bound_h3_addr: Arc<Mutex<Option<SocketAddr>>>,
    shutdown_tx: Arc<watch::Sender<bool>>,
}

impl DashboardService {
    /// Constructs a new `DashboardService` with optional in-process `KubernetesApiClient`.
    #[must_use]
    pub fn new(config: DashboardConfig, client: Option<Arc<KubernetesApiClient>>) -> Self {
        let state = DashboardState::new(config.h3_addr.port());
        let (shutdown_tx, _) = watch::channel(false);
        Self {
            config,
            client,
            state,
            bound_http_addr: Arc::new(Mutex::new(None)),
            bound_h3_addr: Arc::new(Mutex::new(None)),
            shutdown_tx: Arc::new(shutdown_tx),
        }
    }

    /// Returns the service configuration.
    #[must_use]
    pub const fn config(&self) -> &DashboardConfig {
        &self.config
    }

    /// Returns the shared dashboard snapshot state.
    #[must_use]
    pub const fn state(&self) -> &DashboardState {
        &self.state
    }

    /// Returns the bound TCP HTTP/1.1 address once started.
    #[must_use]
    pub fn bound_http_addr(&self) -> Option<SocketAddr> {
        *self
            .bound_http_addr
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Returns the bound UDP HTTP/3 (QUIC) address once started.
    #[must_use]
    pub fn bound_h3_addr(&self) -> Option<SocketAddr> {
        *self
            .bound_h3_addr
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Starts the HTTP/3 QUIC listener, HTTP/1.1 TCP fallback listener, and live collector task.
    pub async fn start(&self) -> Result<()> {
        if !self.config.is_enabled() {
            return Ok(());
        }
        let _ = self.shutdown_tx.send(false);
        let shutdown_rx = self.shutdown_tx.subscribe();

        let (cert_path, key_path) = self.ensure_tls_files()?;
        let (h3_addr, _h3_handle) = spawn_h3_server(
            self.config.h3_addr,
            &cert_path,
            &key_path,
            self.state.clone(),
            shutdown_rx.clone(),
        )
        .await?;
        *self
            .bound_h3_addr
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(h3_addr);
        self.state.set_h3_port(h3_addr.port());

        let (http_addr, _http_handle) = spawn_http1_server(
            self.config.http_addr,
            self.state.clone(),
            shutdown_rx.clone(),
        )
        .await?;
        *self
            .bound_http_addr
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(http_addr);

        let collector_state = self.state.clone();
        let collector_client = self.client.clone();
        let interval = self.config.refresh_interval;
        tokio::spawn(async move {
            run_collector(collector_state, collector_client, interval, shutdown_rx).await;
        });

        Ok(())
    }

    /// Signals all dashboard background tasks and listeners to shut down cleanly.
    pub fn stop(&self) {
        let _ = self.shutdown_tx.send(true);
    }

    fn ensure_tls_files(&self) -> Result<(PathBuf, PathBuf)> {
        if let (Some(cert), Some(key)) = (&self.config.tls_cert_path, &self.config.tls_key_path)
            && cert.is_file()
            && key.is_file()
        {
            return Ok((cert.clone(), key.clone()));
        }

        let dir =
            std::env::temp_dir().join(format!("diorama-dashboard-tls-{}", std::process::id()));
        std::fs::create_dir_all(&dir).map_err(|e| DashboardError::TlsMaterial(e.to_string()))?;
        let cert_path = dir.join("cert.pem");
        let key_path = dir.join("key.pem");
        if !cert_path.is_file() || !key_path.is_file() {
            let certified = rcgen::generate_simple_self_signed(vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
            ])
            .map_err(|e| DashboardError::TlsMaterial(e.to_string()))?;
            std::fs::write(&cert_path, certified.cert.pem())
                .map_err(|e| DashboardError::TlsMaterial(e.to_string()))?;
            std::fs::write(&key_path, certified.signing_key.serialize_pem())
                .map_err(|e| DashboardError::TlsMaterial(e.to_string()))?;
        }
        Ok((cert_path, key_path))
    }
}
