//! Swimlane - represents a swimlane/partition in an activity diagram

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::geometry::{Point, Rect, Color};

/// Unique identifier for a swimlane
pub type SwimlaneId = Uuid;

/// Orientation of the swimlane
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum SwimlaneOrientation {
    /// Vertical swimlane (columns)
    #[default]
    Vertical,
    /// Horizontal swimlane (rows)
    Horizontal,
}

/// A swimlane (activity partition) in an activity diagram
/// Used to organize actions by responsibility (actor, system, department, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Swimlane {
    /// Unique identifier
    pub id: SwimlaneId,
    /// Name of the swimlane (e.g., "Customer", "System", "Database")
    pub name: String,
    /// Bounding rectangle
    pub bounds: Rect,
    /// Orientation (vertical or horizontal)
    pub orientation: SwimlaneOrientation,
    /// Order/position index for sorting
    pub order: u32,
    /// Optional fill color (usually alternating or transparent)
    pub fill_color: Option<Color>,
    /// Whether this swimlane is currently selected
    #[serde(skip)]
    pub has_focus: bool,
}

impl Swimlane {
    /// Create a new vertical swimlane
    pub fn new(name: &str, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            bounds: Rect::from_pos_size(x, y, width, height),
            orientation: SwimlaneOrientation::Vertical,
            order: 0,
            fill_color: None,
            has_focus: false,
        }
    }

    /// Create a new horizontal swimlane
    pub fn new_horizontal(name: &str, x: f32, y: f32, width: f32, height: f32) -> Self {
        let mut swimlane = Self::new(name, x, y, width, height);
        swimlane.orientation = SwimlaneOrientation::Horizontal;
        swimlane
    }

    /// Get the header rectangle (where the name is displayed)
    pub fn header_rect(&self) -> Rect {
        match self.orientation {
            SwimlaneOrientation::Vertical => {
                // Header at top
                Rect::new(
                    self.bounds.x1,
                    self.bounds.y1,
                    self.bounds.x2,
                    self.bounds.y1 + 30.0,
                )
            }
            SwimlaneOrientation::Horizontal => {
                // Header on left
                Rect::new(
                    self.bounds.x1,
                    self.bounds.y1,
                    self.bounds.x1 + 30.0,
                    self.bounds.y2,
                )
            }
        }
    }

    /// Get the content rectangle (where nodes can be placed)
    pub fn content_rect(&self) -> Rect {
        match self.orientation {
            SwimlaneOrientation::Vertical => {
                Rect::new(
                    self.bounds.x1,
                    self.bounds.y1 + 30.0,
                    self.bounds.x2,
                    self.bounds.y2,
                )
            }
            SwimlaneOrientation::Horizontal => {
                Rect::new(
                    self.bounds.x1 + 30.0,
                    self.bounds.y1,
                    self.bounds.x2,
                    self.bounds.y2,
                )
            }
        }
    }

    /// Check if a point is in the header
    pub fn contains_header_point(&self, p: Point) -> bool {
        self.header_rect().contains_point(p)
    }

    /// Check if a point is in the content area
    pub fn contains_content_point(&self, p: Point) -> bool {
        self.content_rect().contains_point(p)
    }

    /// Check if a point is anywhere in the swimlane
    pub fn contains_point(&self, p: Point) -> bool {
        self.bounds.contains_point(p)
    }

    /// Translate the swimlane by an offset
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.bounds = self.bounds.translate(dx, dy);
    }

    /// Resize the swimlane
    pub fn resize(&mut self, new_width: f32, new_height: f32) {
        let min_size = 100.0;
        self.bounds.x2 = self.bounds.x1 + new_width.max(min_size);
        self.bounds.y2 = self.bounds.y1 + new_height.max(min_size);
    }
}

impl Default for Swimlane {
    fn default() -> Self {
        Self::new("Swimlane", 50.0, 50.0, 200.0, 500.0)
    }
}
