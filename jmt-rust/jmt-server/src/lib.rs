//! JMT Server - Backend server for file operations

pub mod file_ops;
pub mod service;

use std::net::SocketAddr;
use tokio::sync::broadcast;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub addr: SocketAddr,
    pub project_path: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Self {
            addr: "127.0.0.1:9876".parse().unwrap(),
            project_path: format!("{}/jmt-projects", home),
        }
    }
}

/// Shutdown signal for graceful termination
pub type ShutdownSignal = broadcast::Receiver<()>;
