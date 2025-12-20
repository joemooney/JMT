//! Message - represents a message between lifelines in a sequence diagram

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::geometry::Point;
use super::lifeline::LifelineId;

/// Unique identifier for a message
pub type MessageId = Uuid;

/// The kind of message in a sequence diagram
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum MessageKind {
    /// Synchronous call (solid line, filled arrowhead)
    #[default]
    Synchronous,
    /// Asynchronous call (solid line, open arrowhead)
    Asynchronous,
    /// Return message (dashed line, open arrowhead)
    Return,
    /// Create message (creates a new object)
    Create,
    /// Destroy message (destroys the target object, shows X)
    Destroy,
    /// Self message (loops back to same lifeline)
    SelfMessage,
    /// Found message (from outside the diagram)
    Found,
    /// Lost message (to outside the diagram)
    Lost,
}

impl MessageKind {
    /// Returns the display name
    pub fn display_name(&self) -> &'static str {
        match self {
            MessageKind::Synchronous => "Synchronous",
            MessageKind::Asynchronous => "Asynchronous",
            MessageKind::Return => "Return",
            MessageKind::Create => "Create",
            MessageKind::Destroy => "Destroy",
            MessageKind::SelfMessage => "Self",
            MessageKind::Found => "Found",
            MessageKind::Lost => "Lost",
        }
    }

    /// Returns true if this message type uses a dashed line
    pub fn is_dashed(&self) -> bool {
        matches!(self, MessageKind::Return)
    }

    /// Returns true if this message type uses a filled arrowhead
    pub fn has_filled_arrow(&self) -> bool {
        matches!(self, MessageKind::Synchronous | MessageKind::Create)
    }
}

/// A message between two lifelines in a sequence diagram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique identifier
    pub id: MessageId,
    /// Source lifeline ID (None for Found messages)
    pub source_id: Option<LifelineId>,
    /// Target lifeline ID (None for Lost messages)
    pub target_id: Option<LifelineId>,
    /// The kind of message
    pub kind: MessageKind,
    /// Y position of the message (same for source and target in simple case)
    pub y: f32,
    /// Message label/name
    pub label: String,
    /// Optional sequence number (e.g., "1", "1.1", "2")
    pub sequence_number: Option<String>,
    /// Optional guard condition (e.g., "[x > 0]")
    pub guard: Option<String>,
    /// Optional arguments
    pub arguments: Option<String>,
    /// Whether this message is currently selected
    #[serde(skip)]
    pub selected: bool,
}

impl Message {
    /// Create a new synchronous message
    pub fn new(source_id: LifelineId, target_id: LifelineId, label: &str, y: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            source_id: Some(source_id),
            target_id: Some(target_id),
            kind: MessageKind::Synchronous,
            y,
            label: label.to_string(),
            sequence_number: None,
            guard: None,
            arguments: None,
            selected: false,
        }
    }

    /// Create a return message
    pub fn new_return(source_id: LifelineId, target_id: LifelineId, label: &str, y: f32) -> Self {
        let mut msg = Self::new(source_id, target_id, label, y);
        msg.kind = MessageKind::Return;
        msg
    }

    /// Create a self message
    pub fn new_self(lifeline_id: LifelineId, label: &str, y: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            source_id: Some(lifeline_id),
            target_id: Some(lifeline_id),
            kind: MessageKind::SelfMessage,
            y,
            label: label.to_string(),
            sequence_number: None,
            guard: None,
            arguments: None,
            selected: false,
        }
    }

    /// Get the full label including sequence number, guard, and arguments
    pub fn full_label(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref seq) = self.sequence_number {
            parts.push(format!("{}:", seq));
        }

        parts.push(self.label.clone());

        if let Some(ref args) = self.arguments {
            parts.push(format!("({})", args));
        }

        if let Some(ref guard) = self.guard {
            parts.push(format!("[{}]", guard));
        }

        parts.join(" ")
    }

    /// Check if a point is near this message line
    pub fn is_near_point(&self, p: Point, source_x: f32, target_x: f32, tolerance: f32) -> bool {
        // Check if Y is close
        if (p.y - self.y).abs() > tolerance {
            return false;
        }

        // Check if X is between source and target
        let min_x = source_x.min(target_x);
        let max_x = source_x.max(target_x);

        p.x >= min_x - tolerance && p.x <= max_x + tolerance
    }
}

impl Default for Message {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            source_id: None,
            target_id: None,
            kind: MessageKind::Synchronous,
            y: 100.0,
            label: "message".to_string(),
            sequence_number: None,
            guard: None,
            arguments: None,
            selected: false,
        }
    }
}
