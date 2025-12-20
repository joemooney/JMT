//! ControlFlow - represents a control flow edge in an activity diagram

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::geometry::Point;

/// Unique identifier for a control flow
pub type ControlFlowId = Uuid;

/// The kind of flow in an activity diagram
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum FlowKind {
    /// Regular control flow (solid arrow)
    #[default]
    Control,
    /// Object flow (solid arrow, may show object token)
    Object,
    /// Exception handler flow
    Exception,
    /// Interruptible edge (zigzag arrow)
    Interrupt,
}

impl FlowKind {
    /// Returns the display name
    pub fn display_name(&self) -> &'static str {
        match self {
            FlowKind::Control => "Control Flow",
            FlowKind::Object => "Object Flow",
            FlowKind::Exception => "Exception",
            FlowKind::Interrupt => "Interrupt",
        }
    }

    /// Returns true if this flow uses a special line style
    pub fn is_special_style(&self) -> bool {
        matches!(self, FlowKind::Exception | FlowKind::Interrupt)
    }
}

/// A control flow edge in an activity diagram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlFlow {
    /// Unique identifier
    pub id: ControlFlowId,
    /// Source node ID
    pub source_id: Uuid,
    /// Target node ID
    pub target_id: Uuid,
    /// The kind of flow
    pub kind: FlowKind,
    /// Optional guard condition (e.g., "[x > 0]")
    pub guard: Option<String>,
    /// Optional weight (for object flow)
    pub weight: Option<String>,
    /// Waypoints for routing (empty for direct connection)
    pub waypoints: Vec<Point>,
    /// Whether this flow is currently selected
    #[serde(skip)]
    pub selected: bool,
}

impl ControlFlow {
    /// Create a new control flow
    pub fn new(source_id: Uuid, target_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            source_id,
            target_id,
            kind: FlowKind::Control,
            guard: None,
            weight: None,
            waypoints: Vec::new(),
            selected: false,
        }
    }

    /// Create a new control flow with a guard
    pub fn with_guard(source_id: Uuid, target_id: Uuid, guard: &str) -> Self {
        let mut flow = Self::new(source_id, target_id);
        flow.guard = Some(guard.to_string());
        flow
    }

    /// Get the label to display on the edge
    pub fn label(&self) -> Option<String> {
        let mut parts = Vec::new();

        if let Some(ref guard) = self.guard {
            parts.push(format!("[{}]", guard));
        }

        if let Some(ref weight) = self.weight {
            parts.push(format!("{{{}}}", weight));
        }

        if parts.is_empty() {
            None
        } else {
            Some(parts.join(" "))
        }
    }

    /// Add a waypoint to the flow
    pub fn add_waypoint(&mut self, point: Point) {
        self.waypoints.push(point);
    }

    /// Clear all waypoints
    pub fn clear_waypoints(&mut self) {
        self.waypoints.clear();
    }

    /// Check if a point is near any segment of this flow
    pub fn is_near_point(&self, p: Point, source_point: Point, target_point: Point, tolerance: f32) -> bool {
        // Build list of all points (source, waypoints, target)
        let mut points = vec![source_point];
        points.extend(self.waypoints.iter().cloned());
        points.push(target_point);

        // Check each segment
        for i in 0..points.len() - 1 {
            if Self::point_near_segment(p, points[i], points[i + 1], tolerance) {
                return true;
            }
        }

        false
    }

    /// Check if a point is near a line segment
    fn point_near_segment(p: Point, a: Point, b: Point, tolerance: f32) -> bool {
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let length_sq = dx * dx + dy * dy;

        if length_sq == 0.0 {
            return p.distance_to(a) <= tolerance;
        }

        let t = ((p.x - a.x) * dx + (p.y - a.y) * dy) / length_sq;
        let t = t.clamp(0.0, 1.0);

        let proj = Point::new(a.x + t * dx, a.y + t * dy);
        p.distance_to(proj) <= tolerance
    }
}

impl Default for ControlFlow {
    fn default() -> Self {
        Self::new(Uuid::nil(), Uuid::nil())
    }
}
