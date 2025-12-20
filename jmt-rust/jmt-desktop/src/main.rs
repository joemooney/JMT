//! JMT Desktop - Desktop launcher that runs both server and client

use std::sync::Arc;
use std::thread;
use tokio::sync::broadcast;
use tracing_subscriber;

fn main() -> eframe::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create shutdown signal for server
    let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(1);
    let shutdown_tx = Arc::new(shutdown_tx);

    // Start server in background thread
    let server_config = jmt_server::ServerConfig::default();
    let server_addr = server_config.addr;

    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            if let Err(e) = jmt_server::service::run_server(server_config, shutdown_rx).await {
                tracing::error!("Server error: {}", e);
            }
        });
    });

    tracing::info!("Server started on {}", server_addr);

    // Run the egui application
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("JMT - State Machine Editor"),
        ..Default::default()
    };

    let result = eframe::run_native(
        "JMT",
        options,
        Box::new(|cc| Ok(Box::new(jmt_client::JmtApp::new(cc)))),
    );

    // Signal server to shutdown
    let _ = shutdown_tx.send(());

    result
}
