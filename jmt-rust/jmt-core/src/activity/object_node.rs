//! ObjectNode - represents an object node in an activity diagram

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::geometry::{Point, Rect, Color};

/// Unique identifier for an object node
pub type ObjectNodeId = Uuid;

/// The kind of object node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ObjectNodeKind {
    /// Central buffer node (rectangle)
    #[default]
    CentralBuffer,
    /// Data store node (rectangle with lines on sides)
    DataStore,
    /// Input pin (small square on action boundary)
    InputPin,
    /// Output pin (small square on action boundary)
    OutputPin,
    /// Activity parameter node (on activity boundary)
    ActivityParameter,
    /// Expansion node (small square, part of expansion region)
    ExpansionNode,
}

impl ObjectNodeKind {
    /// Returns the display name
    pub fn display_name(&self) -> &'static str {
        match self {
            ObjectNodeKind::CentralBuffer => "Central Buffer",
            ObjectNodeKind::DataStore => "Data Store",
            ObjectNodeKind::InputPin => "Input Pin",
            ObjectNodeKind::OutputPin => "Output Pin",
            ObjectNodeKind::ActivityParameter => "Activity Parameter",
            ObjectNodeKind::ExpansionNode => "Expansion Node",
        }
    }

    /// Returns true if this is a pin type (attached to an action)
    pub fn is_pin(&self) -> bool {
        matches!(self, ObjectNodeKind::InputPin | ObjectNodeKind::OutputPin)
    }
}

/// An object node in an activity diagram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectNode {
    /// Unique identifier
    pub id: ObjectNodeId,
    /// Name/label
    pub name: String,
    /// Type of the object (e.g., "Order", "String")
    pub object_type: Option<String>,
    /// Bounding rectangle
    pub bounds: Rect,
    /// The kind of object node
    pub kind: ObjectNodeKind,
    /// Optional fill color
    pub fill_color: Option<Color>,
    /// Parent action ID (for pins)
    pub parent_action_id: Option<Uuid>,
    /// Optional state (e.g., "[submitted]")
    pub state: Option<String>,
    /// Upper bound for tokens (e.g., "*" for unlimited, "1" for single)
    pub upper_bound: Option<String>,
    /// Whether this is ordering="ordered"
    pub is_ordered: bool,
    /// Whether this is a stream
    pub is_stream: bool,
    /// Whether this node is currently selected
    #[serde(skip)]
    pub has_focus: bool,
}

impl ObjectNode {
    /// Create a new object node
    pub fn new(name: &str, kind: ObjectNodeKind, x: f32, y: f32) -> Self {
        let (width, height) = match kind {
            ObjectNodeKind::InputPin | ObjectNodeKind::OutputPin | ObjectNodeKind::ExpansionNode => (14.0, 14.0),
            ObjectNodeKind::CentralBuffer | ObjectNodeKind::DataStore => (80.0, 40.0),
            ObjectNodeKind::ActivityParameter => (80.0, 30.0),
        };

        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            object_type: None,
            bounds: Rect::from_pos_size(x, y, width, height),
            kind,
            fill_color: None,
            parent_action_id: None,
            state: None,
            upper_bound: None,
            is_ordered: false,
            is_stream: false,
            has_focus: false,
        }
    }

    /// Create an input pin attached to an action
    pub fn new_input_pin(name: &str, action_id: Uuid, x: f32, y: f32) -> Self {
        let mut node = Self::new(name, ObjectNodeKind::InputPin, x, y);
        node.parent_action_id = Some(action_id);
        node
    }

    /// Create an output pin attached to an action
    pub fn new_output_pin(name: &str, action_id: Uuid, x: f32, y: f32) -> Self {
        let mut node = Self::new(name, ObjectNodeKind::OutputPin, x, y);
        node.parent_action_id = Some(action_id);
        node
    }

    /// Create a data store node
    pub fn new_data_store(name: &str, x: f32, y: f32) -> Self {
        let mut node = Self::new(name, ObjectNodeKind::DataStore, x, y);
        node.bounds = Rect::from_pos_size(x, y, 100.0, 50.0);
        node
    }

    /// Get the display label including type and state
    pub fn display_label(&self) -> String {
        let mut parts = Vec::new();

        parts.push(self.name.clone());

        if let Some(ref obj_type) = self.object_type {
            parts.push(format!(": {}", obj_type));
        }

        if let Some(ref state) = self.state {
            parts.push(format!(" [{}]", state));
        }

        parts.join("")
    }

    /// Get the center point
    pub fn center(&self) -> Point {
        self.bounds.center()
    }

    /// Check if a point is inside
    pub fn contains_point(&self, p: Point) -> bool {
        self.bounds.contains_point(p)
    }

    /// Translate by an offset
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.bounds = self.bounds.translate(dx, dy);
    }
}

impl Default for ObjectNode {
    fn default() -> Self {
        Self::new("object", ObjectNodeKind::CentralBuffer, 100.0, 100.0)
    }
}
