//! Server client abstraction for file operations

use jmt_core::Diagram;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Server error: {0}")]
    Server(String),
    #[error("Timeout")]
    Timeout,
}

pub type Result<T> = std::result::Result<T, ServerError>;

/// File information returned from server
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub modified_time: u64,
}

/// Trait for server communication (allows different implementations for native vs WASM)
pub trait ServerClient: Send + Sync {
    /// Create a new diagram
    fn new_diagram(&self, name: &str) -> Result<Diagram>;

    /// Open a diagram from a path
    fn open_diagram(&self, path: &str) -> Result<Diagram>;

    /// Save a diagram to a path
    fn save_diagram(&self, diagram: &Diagram, path: Option<&str>) -> Result<String>;

    /// List files in a directory
    fn list_files(&self, directory: &str, extension: &str) -> Result<Vec<FileInfo>>;

    /// Get the default project path
    fn get_project_path(&self) -> Result<String>;
}

/// Offline client that works without a server (for initial development)
pub struct OfflineClient;

impl ServerClient for OfflineClient {
    fn new_diagram(&self, name: &str) -> Result<Diagram> {
        Ok(Diagram::new(name))
    }

    fn open_diagram(&self, _path: &str) -> Result<Diagram> {
        Err(ServerError::Server("Offline mode - cannot open files".to_string()))
    }

    fn save_diagram(&self, _diagram: &Diagram, _path: Option<&str>) -> Result<String> {
        Err(ServerError::Server("Offline mode - cannot save files".to_string()))
    }

    fn list_files(&self, _directory: &str, _extension: &str) -> Result<Vec<FileInfo>> {
        Ok(Vec::new())
    }

    fn get_project_path(&self) -> Result<String> {
        Ok(".".to_string())
    }
}

impl Default for OfflineClient {
    fn default() -> Self {
        Self
    }
}

// Native WebSocket client implementation would go here
// WASM WebSocket client implementation would go here
