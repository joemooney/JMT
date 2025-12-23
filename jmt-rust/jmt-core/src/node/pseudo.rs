//! Pseudo-state nodes (Initial, Final, Choice, Fork, Join, Junction)

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::geometry::{Color, Rect};
use super::{NodeId, NodeType};

/// The kind of pseudo-state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PseudoStateKind {
    Initial,
    Final,
    Choice,
    Fork,
    Join,
    Junction,
}

impl From<PseudoStateKind> for NodeType {
    fn from(kind: PseudoStateKind) -> Self {
        match kind {
            PseudoStateKind::Initial => NodeType::Initial,
            PseudoStateKind::Final => NodeType::Final,
            PseudoStateKind::Choice => NodeType::Choice,
            PseudoStateKind::Fork => NodeType::Fork,
            PseudoStateKind::Join => NodeType::Join,
            PseudoStateKind::Junction => NodeType::Junction,
        }
    }
}

impl PseudoStateKind {
    /// Returns the display name for this pseudo-state kind
    pub fn display_name(&self) -> &'static str {
        match self {
            PseudoStateKind::Initial => "Initial",
            PseudoStateKind::Final => "Final",
            PseudoStateKind::Choice => "Choice",
            PseudoStateKind::Fork => "Fork",
            PseudoStateKind::Join => "Join",
            PseudoStateKind::Junction => "Junction",
        }
    }

    /// Returns the default size for this pseudo-state kind
    pub fn default_size(&self) -> (f32, f32) {
        match self {
            PseudoStateKind::Initial | PseudoStateKind::Final | PseudoStateKind::Junction => {
                (20.0, 20.0)
            }
            PseudoStateKind::Choice => (30.0, 30.0),
            PseudoStateKind::Fork | PseudoStateKind::Join => (60.0, 8.0),
        }
    }

    /// Returns true if this pseudo-state should be square
    pub fn should_be_square(&self) -> bool {
        matches!(
            self,
            PseudoStateKind::Initial
                | PseudoStateKind::Final
                | PseudoStateKind::Choice
                | PseudoStateKind::Junction
        )
    }

    /// Returns true if this pseudo-state can be a connection source
    pub fn can_be_source(&self) -> bool {
        !matches!(self, PseudoStateKind::Final | PseudoStateKind::Join)
    }

    /// Returns true if this pseudo-state can be a connection target
    pub fn can_be_target(&self) -> bool {
        !matches!(self, PseudoStateKind::Initial | PseudoStateKind::Fork)
    }
}

/// A pseudo-state in the state machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PseudoState {
    /// Unique identifier
    pub id: NodeId,
    /// Display name (optional for pseudo-states)
    pub name: String,
    /// The kind of pseudo-state
    pub kind: PseudoStateKind,
    /// Bounding rectangle
    pub bounds: Rect,
    /// Fill color
    pub fill_color: Color,
    /// Parent region ID
    pub parent_region_id: Option<Uuid>,
    /// Whether this pseudo-state is currently selected
    #[serde(skip)]
    pub has_focus: bool,
    /// Whether this node has a placement error (partially inside another node)
    #[serde(skip)]
    pub has_error: bool,
    /// Whether this node has been explicitly aligned (for crop grid-snapping)
    #[serde(default)]
    pub aligned: bool,
    /// Sequential ID for this pseudo-state (e.g., "initial0001")
    #[serde(default)]
    pub seq_id: String,
}

impl PseudoState {
    /// Create a new pseudo-state
    pub fn new(kind: PseudoStateKind, x: f32, y: f32) -> Self {
        let (width, height) = kind.default_size();
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            kind,
            bounds: Rect::from_pos_size(x, y, width, height),
            fill_color: Color::BLACK,
            parent_region_id: None,
            has_focus: false,
            has_error: false,
            aligned: false,
            seq_id: String::new(),
        }
    }

    /// Create a new pseudo-state with a specific ID
    pub fn with_id(id: NodeId, kind: PseudoStateKind, x: f32, y: f32) -> Self {
        let (width, height) = kind.default_size();
        Self {
            id,
            name: String::new(),
            kind,
            bounds: Rect::from_pos_size(x, y, width, height),
            fill_color: Color::BLACK,
            parent_region_id: None,
            has_focus: false,
            has_error: false,
            aligned: false,
            seq_id: String::new(),
        }
    }

    /// Enforce square dimensions if required by the pseudo-state kind
    pub fn enforce_square(&mut self) {
        if self.kind.should_be_square() {
            let size = self.bounds.width().max(self.bounds.height());
            self.bounds.x2 = self.bounds.x1 + size;
            self.bounds.y2 = self.bounds.y1 + size;
        }
    }

    /// Get the center point
    pub fn center(&self) -> crate::geometry::Point {
        self.bounds.center()
    }

    /// Get the radius (for circular pseudo-states)
    pub fn radius(&self) -> f32 {
        self.bounds.width().min(self.bounds.height()) / 2.0
    }
}
