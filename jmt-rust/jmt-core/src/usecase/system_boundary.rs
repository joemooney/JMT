//! SystemBoundary - represents the system boundary in a use case diagram

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::geometry::{Point, Rect, Color};

/// Unique identifier for a system boundary
pub type SystemBoundaryId = Uuid;

/// A system boundary in a use case diagram
/// Represents the boundary of the system being modeled (contains use cases)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemBoundary {
    /// Unique identifier
    pub id: SystemBoundaryId,
    /// Name of the system
    pub name: String,
    /// Bounding rectangle
    pub bounds: Rect,
    /// Optional fill color (usually light gray or transparent)
    pub fill_color: Option<Color>,
    /// Whether this boundary is currently selected
    #[serde(skip)]
    pub has_focus: bool,
}

impl SystemBoundary {
    /// Create a new system boundary
    pub fn new(name: &str, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            bounds: Rect::from_pos_size(x, y, width, height),
            fill_color: Some(Color::with_alpha(240, 240, 240, 128)),
            has_focus: false,
        }
    }

    /// Get the center point
    pub fn center(&self) -> Point {
        self.bounds.center()
    }

    /// Check if a point is inside the boundary
    pub fn contains_point(&self, p: Point) -> bool {
        self.bounds.contains_point(p)
    }

    /// Check if another rectangle is fully contained
    pub fn contains_rect(&self, rect: &Rect) -> bool {
        self.bounds.contains_rect(rect)
    }

    /// Translate the boundary by an offset
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.bounds = self.bounds.translate(dx, dy);
    }

    /// Resize the boundary
    pub fn resize(&mut self, new_width: f32, new_height: f32) {
        self.bounds.x2 = self.bounds.x1 + new_width.max(100.0);
        self.bounds.y2 = self.bounds.y1 + new_height.max(100.0);
    }

    /// Get the header height (where the name is displayed)
    pub fn header_height(&self) -> f32 {
        25.0
    }
}

impl Default for SystemBoundary {
    fn default() -> Self {
        Self::new("System", 100.0, 50.0, 400.0, 300.0)
    }
}
