//! Standalone in-cluster dashboard pod binary packaged into a `FROM scratch` OCI image by `oci-builder`.
#![forbid(unsafe_code)]

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::process::ExitCode;

use diorama_dashboard::server::spawn_http1_server;
use diorama_dashboard::{
    DEFAULT_DASHBOARD_H3_PORT, DEFAULT_DASHBOARD_HTTP_PORT, DEFAULT_REFRESH_INTERVAL,
    DashboardState, run_synthetic_collector,
};
use tokio::sync::watch;

fn resolve_listen_addr() -> SocketAddr {
    std::env::var("DIORAMA_DASHBOARD_HTTP_ADDR")
        .ok()
        .and_then(|raw| raw.parse().ok())
        .unwrap_or_else(|| {
            SocketAddr::new(
                IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                DEFAULT_DASHBOARD_HTTP_PORT,
            )
        })
}

async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = resolve_listen_addr();
    let state = DashboardState::new(DEFAULT_DASHBOARD_H3_PORT);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let collector_state = state.clone();
    let collector_shutdown = shutdown_rx.clone();
    tokio::spawn(async move {
        run_synthetic_collector(
            collector_state,
            DEFAULT_REFRESH_INTERVAL,
            collector_shutdown,
        )
        .await;
    });

    let (bound, handle) = spawn_http1_server(addr, state, shutdown_rx).await?;
    println!("diorama-dashboard listening on http://{bound}");

    let _ = tokio::signal::ctrl_c().await;
    let _ = shutdown_tx.send(true);
    let _ = handle.await;
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: diorama-dashboard: {err}");
            ExitCode::FAILURE
        },
    }
}
