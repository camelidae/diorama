use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;

use thiserror::Error;

/// Default TCP HTTP/1.1 fallback port for the node dashboard.
pub const DEFAULT_DASHBOARD_HTTP_PORT: u16 = 8080;

/// Default UDP HTTP/3 (QUIC) port for the node dashboard.
pub const DEFAULT_DASHBOARD_H3_PORT: u16 = 8443;

/// Default interval between synthetic or periodic refresh ticks.
pub const DEFAULT_REFRESH_INTERVAL: Duration = Duration::from_secs(2);

/// Errors emitted by the realtime node dashboard service.
#[derive(Debug, Error)]
pub enum DashboardError {
    /// Failed to bind TCP or UDP listener.
    #[error("dashboard listener bind failed: {0}")]
    Bind(#[from] io::Error),
    /// Failed to generate or write TLS material for HTTP/3 QUIC listener.
    #[error("dashboard TLS material initialization failed: {0}")]
    TlsMaterial(String),
    /// Failed to initialize `tokio-quiche` HTTP/3 listener.
    #[error("dashboard HTTP/3 listener initialization failed: {0}")]
    QuicListen(String),
}

impl DashboardError {
    /// Stable diagnostic code for supervisor lifecycle reporting.
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::Bind(_) => "dashboard-bind-failed",
            Self::TlsMaterial(_) => "dashboard-tls-failed",
            Self::QuicListen(_) => "dashboard-quic-listen-failed",
        }
    }
}

/// Result alias for dashboard operations.
pub type Result<T> = std::result::Result<T, DashboardError>;

/// Configuration for the realtime HTTP/3 + HTMX node dashboard component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DashboardConfig {
    /// Whether the dashboard component is enabled.
    pub enabled: bool,
    /// TCP bind address for the HTTP/1.1 listener (also advertises `Alt-Svc: h3`).
    pub http_addr: SocketAddr,
    /// UDP bind address for the HTTP/3 (QUIC) listener.
    pub h3_addr: SocketAddr,
    /// Optional X.509 PEM certificate path for HTTP/3 QUIC.
    pub tls_cert_path: Option<PathBuf>,
    /// Optional PKCS#8 PEM private key path for HTTP/3 QUIC.
    pub tls_key_path: Option<PathBuf>,
    /// Refresh interval for periodic snapshots and synthetic fallback ticks.
    pub refresh_interval: Duration,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            http_addr: SocketAddr::new(
                IpAddr::V4(Ipv4Addr::LOCALHOST),
                DEFAULT_DASHBOARD_HTTP_PORT,
            ),
            h3_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), DEFAULT_DASHBOARD_H3_PORT),
            tls_cert_path: None,
            tls_key_path: None,
            refresh_interval: DEFAULT_REFRESH_INTERVAL,
        }
    }
}

impl DashboardConfig {
    /// Creates an enabled dashboard configuration bound to the given HTTP/1.1 and HTTP/3 addresses.
    #[must_use]
    pub const fn new(http_addr: SocketAddr, h3_addr: SocketAddr) -> Self {
        Self {
            enabled: true,
            http_addr,
            h3_addr,
            tls_cert_path: None,
            tls_key_path: None,
            refresh_interval: DEFAULT_REFRESH_INTERVAL,
        }
    }

    /// Resolves dashboard configuration from `DIORAMA_DASHBOARD_*` environment variables,
    /// optionally reusing node PKI certificate and key paths when present.
    #[must_use]
    pub fn from_env_with_pki(pki_dir: Option<&std::path::Path>) -> Self {
        let enabled = std::env::var("DIORAMA_DASHBOARD_ENABLED")
            .is_ok_and(|v| matches!(v.trim(), "1" | "true" | "TRUE" | "yes" | "on"));
        let http_addr = std::env::var("DIORAMA_DASHBOARD_HTTP_ADDR")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| {
                SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), DEFAULT_DASHBOARD_HTTP_PORT)
            });
        let h3_addr = std::env::var("DIORAMA_DASHBOARD_H3_ADDR")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| {
                SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), DEFAULT_DASHBOARD_H3_PORT)
            });
        let (tls_cert_path, tls_key_path) = if let Some(dir) = pki_dir {
            let cert = dir.join("apiserver.crt");
            let key = dir.join("apiserver.key");
            if cert.is_file() && key.is_file() {
                (Some(cert), Some(key))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };
        Self {
            enabled,
            http_addr,
            h3_addr,
            tls_cert_path,
            tls_key_path,
            refresh_interval: DEFAULT_REFRESH_INTERVAL,
        }
    }

    /// Sets explicit TLS certificate and private key paths for the HTTP/3 QUIC listener.
    #[must_use]
    pub fn with_tls_paths(mut self, cert_path: PathBuf, key_path: PathBuf) -> Self {
        self.tls_cert_path = Some(cert_path);
        self.tls_key_path = Some(key_path);
        self
    }

    /// Sets the periodic snapshot refresh interval.
    #[must_use]
    pub const fn with_refresh_interval(mut self, interval: Duration) -> Self {
        self.refresh_interval = interval;
        self
    }

    /// Returns whether the dashboard service is enabled.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }
}
