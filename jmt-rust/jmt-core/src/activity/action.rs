//! Action - represents an action node in an activity diagram

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::geometry::{Point, Rect, Color};

/// Unique identifier for an action
pub type ActionId = Uuid;

/// The kind of action node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ActionKind {
    /// Regular action (rounded rectangle)
    #[default]
    Action,
    /// Call behavior action (invokes another activity)
    CallBehavior,
    /// Call operation action (invokes an operation)
    CallOperation,
    /// Send signal action (convex pentagon pointing right)
    SendSignal,
    /// Accept event action (concave pentagon)
    AcceptEvent,
    /// Accept time event action (hourglass shape)
    AcceptTimeEvent,
}

impl ActionKind {
    /// Returns the display name
    pub fn display_name(&self) -> &'static str {
        match self {
            ActionKind::Action => "Action",
            ActionKind::CallBehavior => "Call Behavior",
            ActionKind::CallOperation => "Call Operation",
            ActionKind::SendSignal => "Send Signal",
            ActionKind::AcceptEvent => "Accept Event",
            ActionKind::AcceptTimeEvent => "Accept Time Event",
        }
    }
}

/// An action node in an activity diagram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// Unique identifier
    pub id: ActionId,
    /// Name/label of the action
    pub name: String,
    /// Bounding rectangle
    pub bounds: Rect,
    /// The kind of action
    pub kind: ActionKind,
    /// Optional fill color
    pub fill_color: Option<Color>,
    /// Pre-condition
    pub precondition: Option<String>,
    /// Post-condition
    pub postcondition: Option<String>,
    /// Partition/swimlane ID this action belongs to
    pub partition_id: Option<Uuid>,
    /// Whether this action is currently selected
    #[serde(skip)]
    pub has_focus: bool,
}

impl Action {
    /// Create a new action
    pub fn new(name: &str, x: f32, y: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            bounds: Rect::from_pos_size(x, y, 100.0, 50.0),
            kind: ActionKind::Action,
            fill_color: None,
            precondition: None,
            postcondition: None,
            partition_id: None,
            has_focus: false,
        }
    }

    /// Create a send signal action
    pub fn new_send_signal(name: &str, x: f32, y: f32) -> Self {
        let mut action = Self::new(name, x, y);
        action.kind = ActionKind::SendSignal;
        action.bounds = Rect::from_pos_size(x, y, 100.0, 40.0);
        action
    }

    /// Create an accept event action
    pub fn new_accept_event(name: &str, x: f32, y: f32) -> Self {
        let mut action = Self::new(name, x, y);
        action.kind = ActionKind::AcceptEvent;
        action.bounds = Rect::from_pos_size(x, y, 100.0, 40.0);
        action
    }

    /// Create an accept time event action
    pub fn new_time_event(x: f32, y: f32) -> Self {
        let mut action = Self::new("", x, y);
        action.kind = ActionKind::AcceptTimeEvent;
        action.bounds = Rect::from_pos_size(x, y, 30.0, 40.0);
        action
    }

    /// Get the center point
    pub fn center(&self) -> Point {
        self.bounds.center()
    }

    /// Check if a point is inside the action
    pub fn contains_point(&self, p: Point) -> bool {
        self.bounds.contains_point(p)
    }

    /// Translate the action by an offset
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.bounds = self.bounds.translate(dx, dy);
    }

    /// Get the connection point on a specific side
    pub fn connection_point(&self, side: crate::node::Side) -> Point {
        use crate::node::Side;
        match side {
            Side::Left => Point::new(self.bounds.x1, self.bounds.center().y),
            Side::Right => Point::new(self.bounds.x2, self.bounds.center().y),
            Side::Top => Point::new(self.bounds.center().x, self.bounds.y1),
            Side::Bottom => Point::new(self.bounds.center().x, self.bounds.y2),
            Side::None => self.center(),
        }
    }

    /// Calculate the rounding for the action (rounded rectangle corners)
    pub fn corner_rounding(&self) -> f32 {
        match self.kind {
            ActionKind::Action | ActionKind::CallBehavior | ActionKind::CallOperation => {
                (self.bounds.height() / 2.0).min(15.0)
            }
            _ => 0.0, // Other shapes don't use rounding
        }
    }

    /// Get the shape points for non-rectangular actions
    /// Returns points for SendSignal (convex pentagon) and AcceptEvent (concave pentagon)
    pub fn shape_points(&self) -> Option<Vec<Point>> {
        let b = &self.bounds;
        let notch_width = 15.0;

        match self.kind {
            ActionKind::SendSignal => {
                // Convex pentagon pointing right (like an arrow)
                Some(vec![
                    Point::new(b.x1, b.y1),
                    Point::new(b.x2 - notch_width, b.y1),
                    Point::new(b.x2, b.center().y),
                    Point::new(b.x2 - notch_width, b.y2),
                    Point::new(b.x1, b.y2),
                ])
            }
            ActionKind::AcceptEvent => {
                // Concave pentagon (notch on left)
                Some(vec![
                    Point::new(b.x1 + notch_width, b.y1),
                    Point::new(b.x2, b.y1),
                    Point::new(b.x2, b.y2),
                    Point::new(b.x1 + notch_width, b.y2),
                    Point::new(b.x1, b.center().y),
                ])
            }
            _ => None,
        }
    }
}

impl Default for Action {
    fn default() -> Self {
        Self::new("Action", 100.0, 100.0)
    }
}
