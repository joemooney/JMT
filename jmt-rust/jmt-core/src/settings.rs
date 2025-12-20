//! Diagram settings and global options

use serde::{Deserialize, Serialize};
use crate::geometry::Color;

/// Settings for a diagram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramSettings {
    /// Name of the diagram
    pub name: String,
    /// File path where the diagram is saved
    pub file_path: Option<String>,
    /// Default fill color for states
    pub state_color: Color,
    /// Corner rounding radius for states
    pub corner_rounding: f32,
    /// Stub length for connections
    pub stub_length: f32,
    /// Arrow head dimensions
    pub arrow_width: f32,
    pub arrow_height: f32,
    /// Corner indicator size for selected nodes
    pub corner_size: f32,
    /// Pseudo-state corner size
    pub pseudo_corner_size: f32,
    /// Default state dimensions
    pub default_state_width: f32,
    pub default_state_height: f32,
    /// Default pseudo-state dimensions
    pub default_pseudo_size: f32,
}

impl Default for DiagramSettings {
    fn default() -> Self {
        Self {
            name: String::from("Untitled"),
            file_path: None,
            state_color: Color::STATE_FILL,
            corner_rounding: 12.0,
            stub_length: 10.0,
            arrow_width: 3.0,
            arrow_height: 8.0,
            corner_size: 6.0,
            pseudo_corner_size: 3.0,
            default_state_width: 100.0,
            default_state_height: 60.0,
            default_pseudo_size: 20.0,
        }
    }
}

impl DiagramSettings {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    /// Returns whether this diagram has been saved to a file
    pub fn has_file(&self) -> bool {
        self.file_path.is_some()
    }
}
