//! Lifeline - represents an object or actor in a sequence diagram

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::geometry::{Color, Rect, Point};

/// Unique identifier for a lifeline
pub type LifelineId = Uuid;

/// A lifeline in a sequence diagram
/// Represents an object or actor that participates in the interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lifeline {
    /// Unique identifier
    pub id: LifelineId,
    /// Name of the object/actor (displayed in the head box)
    pub name: String,
    /// Optional stereotype (e.g., "actor", "boundary", "control", "entity")
    pub stereotype: Option<String>,
    /// X position of the lifeline (center of the head and dashed line)
    pub x: f32,
    /// Y position of the head box top
    pub y: f32,
    /// Width of the head box
    pub head_width: f32,
    /// Height of the head box
    pub head_height: f32,
    /// Length of the dashed lifeline below the head
    pub line_length: f32,
    /// Optional fill color for the head box
    pub fill_color: Option<Color>,
    /// Whether this lifeline is currently selected
    #[serde(skip)]
    pub has_focus: bool,
    /// Whether this lifeline is destroyed (shows X at end)
    pub is_destroyed: bool,
    /// Y position where destruction occurs (if is_destroyed is true)
    pub destruction_y: Option<f32>,
}

impl Lifeline {
    /// Create a new lifeline
    pub fn new(name: &str, x: f32, y: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            stereotype: None,
            x,
            y,
            head_width: 80.0,
            head_height: 40.0,
            line_length: 300.0,
            fill_color: None,
            has_focus: false,
            is_destroyed: false,
            destruction_y: None,
        }
    }

    /// Create a new actor lifeline (with stick figure icon)
    pub fn new_actor(name: &str, x: f32, y: f32) -> Self {
        let mut lifeline = Self::new(name, x, y);
        lifeline.stereotype = Some("actor".to_string());
        lifeline.head_width = 40.0;
        lifeline.head_height = 50.0;
        lifeline
    }

    /// Get the bounds of the head box
    pub fn head_bounds(&self) -> Rect {
        Rect::from_pos_size(
            self.x - self.head_width / 2.0,
            self.y,
            self.head_width,
            self.head_height,
        )
    }

    /// Get the center point of the lifeline at a given Y position
    pub fn center_at_y(&self, y: f32) -> Point {
        Point::new(self.x, y)
    }

    /// Get the full bounds including the dashed line
    pub fn full_bounds(&self) -> Rect {
        let end_y = if let Some(dy) = self.destruction_y {
            dy
        } else {
            self.y + self.head_height + self.line_length
        };

        Rect::new(
            self.x - self.head_width / 2.0,
            self.y,
            self.x + self.head_width / 2.0,
            end_y,
        )
    }

    /// Check if a point is within the head box
    pub fn contains_point(&self, p: Point) -> bool {
        self.head_bounds().contains_point(p)
    }

    /// Check if a point is near the lifeline (for message connection)
    pub fn is_near_line(&self, p: Point, tolerance: f32) -> bool {
        let line_start_y = self.y + self.head_height;
        let line_end_y = self.destruction_y.unwrap_or(self.y + self.head_height + self.line_length);

        (self.x - p.x).abs() <= tolerance && p.y >= line_start_y && p.y <= line_end_y
    }

    /// Translate the lifeline by an offset
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.x += dx;
        self.y += dy;
        if let Some(ref mut dy_val) = self.destruction_y {
            *dy_val += dy;
        }
    }
}

impl Default for Lifeline {
    fn default() -> Self {
        Self::new("Object", 100.0, 50.0)
    }
}
