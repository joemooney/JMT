//! Connections (transitions) between nodes

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::geometry::{Point, Rect};
use crate::node::{NodeId, Side};

/// Unique identifier for a connection
pub type ConnectionId = Uuid;

/// A line segment that makes up part of a connection
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LineSegment {
    pub start: Point,
    pub end: Point,
}

impl LineSegment {
    pub fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }

    /// Calculate the length of this segment
    pub fn length(&self) -> f32 {
        self.start.distance_to(self.end)
    }

    /// Check if a point is close to this line segment
    pub fn is_near_point(&self, p: Point, tolerance: f32) -> bool {
        // Quick bounding box check first
        let min_x = self.start.x.min(self.end.x) - tolerance;
        let max_x = self.start.x.max(self.end.x) + tolerance;
        let min_y = self.start.y.min(self.end.y) - tolerance;
        let max_y = self.start.y.max(self.end.y) + tolerance;

        if p.x < min_x || p.x > max_x || p.y < min_y || p.y > max_y {
            return false;
        }

        // Calculate perpendicular distance to line
        let dx = self.end.x - self.start.x;
        let dy = self.end.y - self.start.y;

        if dx.abs() < 0.001 && dy.abs() < 0.001 {
            // Point segment - just check distance to start
            return self.start.distance_to(p) <= tolerance;
        }

        let numerator = ((dy * self.start.y) - (dx * self.start.x)
            + (dx * p.y)
            - (dy * p.x))
            .abs();
        let denominator = (dx * dx + dy * dy).sqrt();
        let distance = numerator / denominator;

        distance <= tolerance
    }
}

/// A connection (transition) between two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    /// Unique identifier
    pub id: ConnectionId,
    /// Display name / label
    pub name: String,
    /// Source node ID
    pub source_id: NodeId,
    /// Target node ID
    pub target_id: NodeId,
    /// Which side of source node the connection starts from
    pub source_side: Side,
    /// Which side of target node the connection ends at
    pub target_side: Side,
    /// Offset along the source side
    pub source_offset: f32,
    /// Offset along the target side
    pub target_offset: f32,
    /// Event that triggers this transition
    pub event: String,
    /// Guard condition for this transition
    pub guard: String,
    /// Action to execute on transition
    pub action: String,
    /// Line segments making up the connection path
    #[serde(skip)]
    pub segments: Vec<LineSegment>,
    /// Whether this connection is selected
    #[serde(skip)]
    pub selected: bool,
}

impl Connection {
    /// Create a new connection between two nodes
    pub fn new(source_id: NodeId, target_id: NodeId) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            source_id,
            target_id,
            source_side: Side::None,
            target_side: Side::None,
            source_offset: 0.0,
            target_offset: 0.0,
            event: String::new(),
            guard: String::new(),
            action: String::new(),
            segments: Vec::new(),
            selected: false,
        }
    }

    /// Create a new connection with a specific ID
    pub fn with_id(id: ConnectionId, source_id: NodeId, target_id: NodeId) -> Self {
        Self {
            id,
            name: String::new(),
            source_id,
            target_id,
            source_side: Side::None,
            target_side: Side::None,
            source_offset: 0.0,
            target_offset: 0.0,
            event: String::new(),
            guard: String::new(),
            action: String::new(),
            segments: Vec::new(),
            selected: false,
        }
    }

    /// Get the label for this connection (event[guard]/action format)
    pub fn label(&self) -> String {
        let mut label = self.event.clone();

        if !self.guard.is_empty() {
            label.push_str(&format!("[{}]", self.guard));
        }

        if !self.action.is_empty() {
            label.push_str(&format!("/{}", self.action));
        }

        label
    }

    /// Check if a point is near this connection
    pub fn is_near_point(&self, p: Point, tolerance: f32) -> bool {
        self.segments.iter().any(|seg| seg.is_near_point(p, tolerance))
    }

    /// Determine the best sides for source and target based on node positions
    pub fn calculate_sides(source_bounds: &Rect, target_bounds: &Rect, stub_len: f32) -> (Side, Side) {
        let source_center = source_bounds.center();
        let target_center = target_bounds.center();

        // Check if target is clearly below source (with enough space for stubs)
        if source_bounds.y2 + stub_len * 2.0 <= target_bounds.y1 {
            return (Side::Bottom, Side::Top);
        }

        // Check if target is clearly above source
        if target_bounds.y2 + stub_len * 2.0 <= source_bounds.y1 {
            return (Side::Top, Side::Bottom);
        }

        // Check if target is to the right of source
        if source_bounds.x2 + stub_len * 2.0 <= target_bounds.x1 {
            return (Side::Right, Side::Left);
        }

        // Check if target is to the left of source
        if target_bounds.x2 + stub_len * 2.0 <= source_bounds.x1 {
            return (Side::Left, Side::Right);
        }

        // Fallback: use the direction with the most separation
        let dx = target_center.x - source_center.x;
        let dy = target_center.y - source_center.y;

        if dx.abs() > dy.abs() {
            if dx > 0.0 {
                (Side::Right, Side::Left)
            } else {
                (Side::Left, Side::Right)
            }
        } else {
            if dy > 0.0 {
                (Side::Bottom, Side::Top)
            } else {
                (Side::Top, Side::Bottom)
            }
        }
    }

    /// Calculate the line segments for this connection based on node positions
    pub fn calculate_segments(
        &mut self,
        source_bounds: &Rect,
        target_bounds: &Rect,
        stub_len: f32,
    ) {
        self.segments.clear();

        // Get connection points on the sides
        let source_point = self.get_side_point(source_bounds, self.source_side, self.source_offset);
        let target_point = self.get_side_point(target_bounds, self.target_side, self.target_offset);

        // Calculate stub points
        let source_stub = self.get_stub_point(source_point, self.source_side, stub_len);
        let target_stub = self.get_stub_point(target_point, self.target_side, stub_len);

        // Create segments: source -> source_stub -> target_stub -> target
        self.segments.push(LineSegment::new(source_point, source_stub));
        self.segments.push(LineSegment::new(source_stub, target_stub));
        self.segments.push(LineSegment::new(target_stub, target_point));
    }

    /// Get the point on a side of a rectangle for connection attachment
    fn get_side_point(&self, bounds: &Rect, side: Side, offset: f32) -> Point {
        match side {
            Side::Top => Point::new(bounds.x1 + bounds.width() / 2.0 + offset, bounds.y1),
            Side::Bottom => Point::new(bounds.x1 + bounds.width() / 2.0 + offset, bounds.y2),
            Side::Left => Point::new(bounds.x1, bounds.y1 + bounds.height() / 2.0 + offset),
            Side::Right => Point::new(bounds.x2, bounds.y1 + bounds.height() / 2.0 + offset),
            Side::None => bounds.center(),
        }
    }

    /// Get the stub point extending from a side point
    fn get_stub_point(&self, point: Point, side: Side, stub_len: f32) -> Point {
        match side {
            Side::Top => Point::new(point.x, point.y - stub_len),
            Side::Bottom => Point::new(point.x, point.y + stub_len),
            Side::Left => Point::new(point.x - stub_len, point.y),
            Side::Right => Point::new(point.x + stub_len, point.y),
            Side::None => point,
        }
    }

    /// Get the end point (for arrow drawing)
    pub fn end_point(&self) -> Option<Point> {
        self.segments.last().map(|s| s.end)
    }

    /// Get the start point
    pub fn start_point(&self) -> Option<Point> {
        self.segments.first().map(|s| s.start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_segment_near_point() {
        let seg = LineSegment::new(Point::new(0.0, 0.0), Point::new(100.0, 0.0));

        // Point on the line
        assert!(seg.is_near_point(Point::new(50.0, 0.0), 5.0));

        // Point near the line
        assert!(seg.is_near_point(Point::new(50.0, 3.0), 5.0));

        // Point far from the line
        assert!(!seg.is_near_point(Point::new(50.0, 10.0), 5.0));

        // Point outside bounding box
        assert!(!seg.is_near_point(Point::new(150.0, 0.0), 5.0));
    }

    #[test]
    fn test_calculate_sides() {
        let source = Rect::new(0.0, 0.0, 100.0, 50.0);
        let target = Rect::new(0.0, 100.0, 100.0, 150.0);

        let (source_side, target_side) = Connection::calculate_sides(&source, &target, 10.0);
        assert_eq!(source_side, Side::Bottom);
        assert_eq!(target_side, Side::Top);
    }
}
