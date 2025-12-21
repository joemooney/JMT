//! Node types for state machine diagrams

mod state;
mod region;
mod pseudo;

pub use state::State;
pub use region::Region;
pub use pseudo::{PseudoState, PseudoStateKind};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::geometry::{Color, Point, Rect};

/// Unique identifier for a node
pub type NodeId = Uuid;

/// The type of node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    State,
    Initial,
    Final,
    Choice,
    Fork,
    Join,
    Junction,
}

impl NodeType {
    /// Returns true if this is a pseudo-state type
    pub fn is_pseudo_state(&self) -> bool {
        !matches!(self, NodeType::State)
    }

    /// Returns the display name for this node type
    pub fn display_name(&self) -> &'static str {
        match self {
            NodeType::State => "State",
            NodeType::Initial => "Initial",
            NodeType::Final => "Final",
            NodeType::Choice => "Choice",
            NodeType::Fork => "Fork",
            NodeType::Join => "Join",
            NodeType::Junction => "Junction",
        }
    }
}

/// Which side of a node a connection attaches to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Side {
    #[default]
    None,
    Top,
    Bottom,
    Left,
    Right,
}

impl Side {
    /// Returns the opposite side
    pub fn opposite(&self) -> Side {
        match self {
            Side::None => Side::None,
            Side::Top => Side::Bottom,
            Side::Bottom => Side::Top,
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }

    /// Returns true if this is a vertical side (top or bottom)
    pub fn is_vertical(&self) -> bool {
        matches!(self, Side::Top | Side::Bottom)
    }

    /// Returns true if this is a horizontal side (left or right)
    pub fn is_horizontal(&self) -> bool {
        matches!(self, Side::Left | Side::Right)
    }
}

/// Which corner of a node is being interacted with (for resizing)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Corner {
    #[default]
    None,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Corner {
    /// Check which corner a point is in, given the node bounds and margin
    /// The margin extends BOTH inside and outside the node bounds to allow
    /// starting a resize by dragging outward (to grow) or inward (to shrink)
    pub fn from_point(bounds: &Rect, p: Point, margin: f32) -> Corner {
        // Check if point is within margin distance of each edge (inside OR outside)
        let in_left = p.x >= bounds.x1 - margin && p.x <= bounds.x1 + margin;
        let in_right = p.x >= bounds.x2 - margin && p.x <= bounds.x2 + margin;
        let in_top = p.y >= bounds.y1 - margin && p.y <= bounds.y1 + margin;
        let in_bottom = p.y >= bounds.y2 - margin && p.y <= bounds.y2 + margin;

        match (in_left, in_right, in_top, in_bottom) {
            (true, _, true, _) => Corner::TopLeft,
            (_, true, true, _) => Corner::TopRight,
            (true, _, _, true) => Corner::BottomLeft,
            (_, true, _, true) => Corner::BottomRight,
            _ => Corner::None,
        }
    }
}

/// A node in the state machine diagram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Node {
    State(State),
    Pseudo(PseudoState),
}

impl Node {
    /// Get the unique ID of this node
    pub fn id(&self) -> NodeId {
        match self {
            Node::State(s) => s.id,
            Node::Pseudo(p) => p.id,
        }
    }

    /// Get the type of this node
    pub fn node_type(&self) -> NodeType {
        match self {
            Node::State(_) => NodeType::State,
            Node::Pseudo(p) => p.kind.into(),
        }
    }

    /// Get the bounds of this node
    pub fn bounds(&self) -> &Rect {
        match self {
            Node::State(s) => &s.bounds,
            Node::Pseudo(p) => &p.bounds,
        }
    }

    /// Get mutable bounds of this node
    pub fn bounds_mut(&mut self) -> &mut Rect {
        match self {
            Node::State(s) => &mut s.bounds,
            Node::Pseudo(p) => &mut p.bounds,
        }
    }

    /// Get the name of this node
    pub fn name(&self) -> &str {
        match self {
            Node::State(s) => &s.name,
            Node::Pseudo(p) => &p.name,
        }
    }

    /// Set the name of this node
    pub fn set_name(&mut self, name: String) {
        match self {
            Node::State(s) => s.name = name,
            Node::Pseudo(p) => p.name = name,
        }
    }

    /// Get the fill color of this node
    pub fn fill_color(&self) -> Option<Color> {
        match self {
            Node::State(s) => s.fill_color,
            Node::Pseudo(p) => Some(p.fill_color),
        }
    }

    /// Set the fill color of this node
    pub fn set_fill_color(&mut self, color: Option<Color>) {
        match self {
            Node::State(s) => s.fill_color = color,
            Node::Pseudo(p) => {
                if let Some(c) = color {
                    p.fill_color = c;
                }
            }
        }
    }

    /// Check if this node is selected/has focus
    pub fn has_focus(&self) -> bool {
        match self {
            Node::State(s) => s.has_focus,
            Node::Pseudo(p) => p.has_focus,
        }
    }

    /// Set the focus state of this node
    pub fn set_focus(&mut self, focus: bool) {
        match self {
            Node::State(s) => s.has_focus = focus,
            Node::Pseudo(p) => p.has_focus = focus,
        }
    }

    /// Get the parent region ID of this node
    pub fn parent_region_id(&self) -> Option<Uuid> {
        match self {
            Node::State(s) => s.parent_region_id,
            Node::Pseudo(p) => p.parent_region_id,
        }
    }

    /// Set the parent region ID of this node
    pub fn set_parent_region_id(&mut self, region_id: Option<Uuid>) {
        match self {
            Node::State(s) => s.parent_region_id = region_id,
            Node::Pseudo(p) => p.parent_region_id = region_id,
        }
    }

    /// Check if a point is inside this node
    pub fn contains_point(&self, p: Point) -> bool {
        self.bounds().contains_point(p)
    }

    /// Get the corner at the given point
    pub fn get_corner(&self, p: Point, margin: f32) -> Corner {
        Corner::from_point(self.bounds(), p, margin)
    }

    /// Move the node by an offset
    pub fn translate(&mut self, dx: f32, dy: f32) {
        let bounds = self.bounds_mut();
        *bounds = bounds.translate(dx, dy);
    }

    /// Resize the node by dragging a corner
    pub fn resize_from_corner(&mut self, corner: Corner, dx: f32, dy: f32, min_width: f32, min_height: f32) {
        let bounds = self.bounds_mut();
        *bounds = bounds.resize_corner(corner, dx, dy, min_width, min_height);
    }

    /// Check if this node can be resized (States can, pseudo-states cannot)
    pub fn can_resize(&self) -> bool {
        matches!(self, Node::State(_))
    }

    /// Get the center point of this node
    pub fn center(&self) -> Point {
        self.bounds().center()
    }

    /// Try to get this node as a State
    pub fn as_state(&self) -> Option<&State> {
        match self {
            Node::State(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get this node as a mutable State
    pub fn as_state_mut(&mut self) -> Option<&mut State> {
        match self {
            Node::State(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get this node as a PseudoState
    pub fn as_pseudo(&self) -> Option<&PseudoState> {
        match self {
            Node::Pseudo(p) => Some(p),
            _ => None,
        }
    }

    /// Try to get this node as a mutable PseudoState
    pub fn as_pseudo_mut(&mut self) -> Option<&mut PseudoState> {
        match self {
            Node::Pseudo(p) => Some(p),
            _ => None,
        }
    }
}
