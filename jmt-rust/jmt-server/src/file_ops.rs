//! File operations for diagrams

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use jmt_core::Diagram;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("File not found: {0}")]
    NotFound(String),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

pub type Result<T> = std::result::Result<T, FileError>;

/// Create a new diagram
pub fn new_diagram(name: &str) -> Diagram {
    Diagram::new(name)
}

/// Save a diagram to a file
pub fn save_diagram(diagram: &Diagram, path: &Path) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(diagram)?;
    fs::write(path, json)?;
    Ok(())
}

/// Load a diagram from a file
pub fn load_diagram(path: &Path) -> Result<Diagram> {
    if !path.exists() {
        return Err(FileError::NotFound(path.display().to_string()));
    }

    let json = fs::read_to_string(path)?;
    let diagram = serde_json::from_str(&json)?;
    Ok(diagram)
}

/// Information about a file
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub path: PathBuf,
    pub modified_time: u64,
}

/// List files in a directory with optional extension filter
pub fn list_files(directory: &Path, extension: Option<&str>) -> Result<Vec<FileInfo>> {
    if !directory.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            // Check extension filter
            if let Some(ext) = extension {
                if path.extension().map(|e| e.to_str().unwrap_or("")) != Some(ext.trim_start_matches('.')) {
                    continue;
                }
            }

            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let modified_time = path
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            files.push(FileInfo {
                name,
                path,
                modified_time,
            });
        }
    }

    // Sort by modification time, newest first
    files.sort_by(|a, b| b.modified_time.cmp(&a.modified_time));

    Ok(files)
}

/// Ensure project directory exists
pub fn ensure_project_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_load_diagram() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.json");

        let diagram = new_diagram("Test Diagram");
        save_diagram(&diagram, &path).unwrap();

        let loaded = load_diagram(&path).unwrap();
        assert_eq!(loaded.settings.name, "Test Diagram");
    }

    #[test]
    fn test_list_files() {
        let dir = tempdir().unwrap();

        // Create some test files
        fs::write(dir.path().join("test1.json"), "{}").unwrap();
        fs::write(dir.path().join("test2.json"), "{}").unwrap();
        fs::write(dir.path().join("other.txt"), "").unwrap();

        let files = list_files(dir.path(), Some("json")).unwrap();
        assert_eq!(files.len(), 2);
    }
}
