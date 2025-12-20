//! UseCase - represents a use case in a use case diagram

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::geometry::{Point, Rect, Color};

/// Unique identifier for a use case
pub type UseCaseId = Uuid;

/// A use case in a use case diagram
/// Represents a specific functionality or behavior of the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseCase {
    /// Unique identifier
    pub id: UseCaseId,
    /// Name of the use case
    pub name: String,
    /// Bounding rectangle (use case is drawn as ellipse within this)
    pub bounds: Rect,
    /// Optional description/notes
    pub description: String,
    /// Optional fill color
    pub fill_color: Option<Color>,
    /// Optional stereotype
    pub stereotype: Option<String>,
    /// Extension points (for extend relationships)
    pub extension_points: Vec<String>,
    /// Whether this use case is currently selected
    #[serde(skip)]
    pub has_focus: bool,
}

impl UseCase {
    /// Create a new use case
    pub fn new(name: &str, x: f32, y: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            bounds: Rect::from_pos_size(x, y, 120.0, 60.0),
            description: String::new(),
            fill_color: None,
            stereotype: None,
            extension_points: Vec::new(),
            has_focus: false,
        }
    }

    /// Create a use case with a specific size
    pub fn with_size(name: &str, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            bounds: Rect::from_pos_size(x, y, width, height),
            description: String::new(),
            fill_color: None,
            stereotype: None,
            extension_points: Vec::new(),
            has_focus: false,
        }
    }

    /// Get the center point of the use case
    pub fn center(&self) -> Point {
        self.bounds.center()
    }

    /// Get the connection point on the ellipse for a given angle (radians)
    pub fn connection_point_at_angle(&self, angle: f32) -> Point {
        let center = self.center();
        let rx = self.bounds.width() / 2.0;
        let ry = self.bounds.height() / 2.0;

        Point::new(
            center.x + rx * angle.cos(),
            center.y + ry * angle.sin(),
        )
    }

    /// Get the connection point on a specific side
    pub fn connection_point(&self, side: crate::node::Side) -> Point {
        use crate::node::Side;
        use std::f32::consts::PI;

        match side {
            Side::Right => self.connection_point_at_angle(0.0),
            Side::Bottom => self.connection_point_at_angle(PI / 2.0),
            Side::Left => self.connection_point_at_angle(PI),
            Side::Top => self.connection_point_at_angle(-PI / 2.0),
            Side::None => self.center(),
        }
    }

    /// Get the closest connection point to a target point
    pub fn closest_connection_point(&self, target: Point) -> Point {
        let center = self.center();
        let dx = target.x - center.x;
        let dy = target.y - center.y;
        let angle = dy.atan2(dx);
        self.connection_point_at_angle(angle)
    }

    /// Check if a point is inside the use case ellipse
    pub fn contains_point(&self, p: Point) -> bool {
        let center = self.center();
        let rx = self.bounds.width() / 2.0;
        let ry = self.bounds.height() / 2.0;

        if rx <= 0.0 || ry <= 0.0 {
            return false;
        }

        let dx = p.x - center.x;
        let dy = p.y - center.y;

        (dx * dx) / (rx * rx) + (dy * dy) / (ry * ry) <= 1.0
    }

    /// Translate the use case by an offset
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.bounds = self.bounds.translate(dx, dy);
    }

    /// Add an extension point
    pub fn add_extension_point(&mut self, name: &str) {
        self.extension_points.push(name.to_string());
    }
}

impl Default for UseCase {
    fn default() -> Self {
        Self::new("Use Case", 100.0, 100.0)
    }
}
