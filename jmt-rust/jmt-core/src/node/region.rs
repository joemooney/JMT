//! Region - container for child nodes within a composite state

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::geometry::Rect;
use super::NodeId;

/// A region within a composite state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    /// Unique identifier
    pub id: Uuid,
    /// Display name
    pub name: String,
    /// Bounding rectangle within parent state
    pub bounds: Rect,
    /// IDs of child nodes in this region
    pub children: Vec<NodeId>,
    /// Whether regions are arranged horizontally (true) or vertically (false)
    pub is_horizontal: bool,
    /// Whether this region is currently selected
    #[serde(skip)]
    pub has_focus: bool,
}

impl Region {
    /// Create a new region with default bounds
    pub fn new(name: &str, parent_bounds: &Rect) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            bounds: parent_bounds.clone(),
            children: Vec::new(),
            is_horizontal: false,
            has_focus: false,
        }
    }

    /// Create a new region with specific bounds
    pub fn with_bounds(name: &str, bounds: Rect) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            bounds,
            children: Vec::new(),
            is_horizontal: false,
            has_focus: false,
        }
    }

    /// Add a child node to this region
    pub fn add_child(&mut self, node_id: NodeId) {
        if !self.children.contains(&node_id) {
            self.children.push(node_id);
        }
    }

    /// Remove a child node from this region
    pub fn remove_child(&mut self, node_id: NodeId) {
        self.children.retain(|id| *id != node_id);
    }

    /// Check if a node is a child of this region
    pub fn contains_child(&self, node_id: NodeId) -> bool {
        self.children.contains(&node_id)
    }

    /// Check if a point is inside this region
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        self.bounds.contains_point(crate::geometry::Point::new(x, y))
    }
}

impl Default for Region {
    fn default() -> Self {
        Self::new("Region", &Rect::default())
    }
}
