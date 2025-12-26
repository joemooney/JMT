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
    /// Whether to show activities in states by default
    #[serde(default = "default_show_activities")]
    pub show_activities: bool,
    /// Whether to show leader lines connecting labels to their connection midpoints
    #[serde(default = "default_show_leader_lines")]
    pub show_leader_lines: bool,
    /// Minimum spacing between nodes (used when placing new nodes and for connection routing)
    #[serde(default = "default_min_node_spacing")]
    pub min_node_spacing: f32,
    /// Extra hit margin for small nodes (makes them easier to click/select)
    #[serde(default = "default_small_node_hit_margin")]
    pub small_node_hit_margin: f32,
    /// Size threshold below which nodes get the extra hit margin
    #[serde(default = "default_small_node_threshold")]
    pub small_node_threshold: f32,
}

fn default_show_activities() -> bool {
    true
}

fn default_small_node_hit_margin() -> f32 {
    10.0  // 10 pixels extra on each side for small nodes
}

fn default_small_node_threshold() -> f32 {
    30.0  // Nodes smaller than 30x30 are considered "small"
}

fn default_show_leader_lines() -> bool {
    true
}

fn default_min_node_spacing() -> f32 {
    40.0
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
            show_activities: true,
            show_leader_lines: true,
            min_node_spacing: 40.0,
            small_node_hit_margin: 10.0,
            small_node_threshold: 30.0,
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
