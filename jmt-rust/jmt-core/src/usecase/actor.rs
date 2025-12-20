//! Actor - represents an actor in a use case diagram

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::geometry::{Point, Rect};

/// Unique identifier for an actor
pub type ActorId = Uuid;

/// An actor in a use case diagram
/// Represents a role that interacts with the system (person, organization, external system)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    /// Unique identifier
    pub id: ActorId,
    /// Name of the actor
    pub name: String,
    /// X position (center of the stick figure)
    pub x: f32,
    /// Y position (top of the head)
    pub y: f32,
    /// Width of the actor (for hit testing)
    pub width: f32,
    /// Height of the actor (including name label)
    pub height: f32,
    /// Optional stereotype (e.g., "external system", "device")
    pub stereotype: Option<String>,
    /// Whether this is displayed as a stick figure (true) or rectangle (false)
    pub use_stick_figure: bool,
    /// Whether this actor is currently selected
    #[serde(skip)]
    pub has_focus: bool,
}

impl Actor {
    /// Create a new actor
    pub fn new(name: &str, x: f32, y: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            x,
            y,
            width: 40.0,
            height: 70.0, // Head + body + legs + label
            stereotype: None,
            use_stick_figure: true,
            has_focus: false,
        }
    }

    /// Create an actor represented as a rectangle (for non-human actors)
    pub fn new_system_actor(name: &str, x: f32, y: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            x,
            y,
            width: 80.0,
            height: 50.0,
            stereotype: Some("system".to_string()),
            use_stick_figure: false,
            has_focus: false,
        }
    }

    /// Get the bounds of the actor
    pub fn bounds(&self) -> Rect {
        Rect::from_pos_size(
            self.x - self.width / 2.0,
            self.y,
            self.width,
            self.height,
        )
    }

    /// Get the center point of the actor (for connection purposes)
    pub fn center(&self) -> Point {
        Point::new(self.x, self.y + self.height / 2.0)
    }

    /// Get the connection point on a specific side
    pub fn connection_point(&self, side: crate::node::Side) -> Point {
        use crate::node::Side;
        let bounds = self.bounds();
        match side {
            Side::Left => Point::new(bounds.x1, bounds.center().y),
            Side::Right => Point::new(bounds.x2, bounds.center().y),
            Side::Top => Point::new(bounds.center().x, bounds.y1),
            Side::Bottom => Point::new(bounds.center().x, bounds.y2),
            Side::None => self.center(),
        }
    }

    /// Check if a point is inside the actor
    pub fn contains_point(&self, p: Point) -> bool {
        self.bounds().contains_point(p)
    }

    /// Translate the actor by an offset
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.x += dx;
        self.y += dy;
    }

    /// Get stick figure drawing dimensions
    /// Returns (head_center_y, head_radius, body_top_y, body_bottom_y, arm_y, leg_spread)
    pub fn stick_figure_dimensions(&self) -> (f32, f32, f32, f32, f32, f32) {
        let head_radius = 8.0;
        let head_center_y = self.y + head_radius;
        let body_top_y = head_center_y + head_radius;
        let body_bottom_y = body_top_y + 20.0;
        let arm_y = body_top_y + 8.0;
        let leg_spread = 12.0;

        (head_center_y, head_radius, body_top_y, body_bottom_y, arm_y, leg_spread)
    }
}

impl Default for Actor {
    fn default() -> Self {
        Self::new("Actor", 50.0, 50.0)
    }
}
