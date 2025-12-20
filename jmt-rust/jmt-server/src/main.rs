//! JMT Server - Standalone server binary

use jmt_server::{service, ServerConfig};
use tokio::sync::broadcast;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create shutdown signal
    let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

    // Handle Ctrl+C
    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        let _ = shutdown_tx_clone.send(());
    });

    // Run server
    let config = ServerConfig::default();
    service::run_server(config, shutdown_rx).await?;

    Ok(())
}
