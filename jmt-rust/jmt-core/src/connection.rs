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

        // Perpendicular distance formula: |dy * (px - x1) - dx * (py - y1)| / sqrt(dx² + dy²)
        // Expanded: |dy * px - dy * x1 - dx * py + dx * y1| / sqrt(dx² + dy²)
        let numerator = (dy * p.x - dx * p.y + dx * self.start.y - dy * self.start.x).abs();
        let denominator = (dx * dx + dy * dy).sqrt();
        let distance = numerator / denominator;

        distance <= tolerance
    }
}

/// Default value for text_adjoined field
fn default_text_adjoined() -> bool {
    true
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
    /// Custom label position offset from default (connection midpoint)
    /// If None, label is positioned at default location (slightly above midpoint)
    #[serde(default)]
    pub label_offset: Option<(f32, f32)>,
    /// Whether the label is adjoined to the connection (vs floating with leader line)
    /// When true, label stays at default position and target node may be shifted if overlapping
    #[serde(default = "default_text_adjoined")]
    pub text_adjoined: bool,
    /// Line segments making up the connection path
    #[serde(skip)]
    pub segments: Vec<LineSegment>,
    /// Whether this connection is selected
    #[serde(skip)]
    pub selected: bool,
    /// Whether the label is being hovered/selected (for interaction)
    #[serde(skip)]
    pub label_selected: bool,
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
            label_offset: None,
            text_adjoined: true,
            segments: Vec::new(),
            selected: false,
            label_selected: false,
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
            label_offset: None,
            text_adjoined: true,
            segments: Vec::new(),
            selected: false,
            label_selected: false,
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

    /// Get the midpoint of the connection (center of all segments)
    pub fn midpoint(&self) -> Option<Point> {
        if self.segments.is_empty() {
            return None;
        }

        // Calculate total length and find the point at half that length
        let total_length: f32 = self.segments.iter().map(|s| s.length()).sum();
        let half_length = total_length / 2.0;

        let mut accumulated = 0.0;
        for seg in &self.segments {
            let seg_len = seg.length();
            if accumulated + seg_len >= half_length {
                // The midpoint is on this segment
                let remaining = half_length - accumulated;
                let t = if seg_len > 0.0 { remaining / seg_len } else { 0.0 };
                return Some(Point::new(
                    seg.start.x + t * (seg.end.x - seg.start.x),
                    seg.start.y + t * (seg.end.y - seg.start.y),
                ));
            }
            accumulated += seg_len;
        }

        // Fallback: return end of last segment
        self.segments.last().map(|s| s.end)
    }

    /// Get the label position (midpoint + offset if set, or default position above midpoint)
    /// Returns (label_position, midpoint) for drawing leader line
    pub fn label_position(&self) -> Option<(Point, Point)> {
        let midpoint = self.midpoint()?;

        let label_pos = if let Some((dx, dy)) = self.label_offset {
            Point::new(midpoint.x + dx, midpoint.y + dy)
        } else {
            // Default: 15 pixels above the midpoint
            Point::new(midpoint.x, midpoint.y - 15.0)
        };

        Some((label_pos, midpoint))
    }

    /// Set the label offset from the connection midpoint
    pub fn set_label_offset(&mut self, offset: Option<(f32, f32)>) {
        self.label_offset = offset;
    }

    /// Calculate estimated label dimensions based on label text
    /// Returns (width, height) in pixels
    pub fn label_dimensions(&self) -> (f32, f32) {
        let label = self.label();
        // Estimate ~6px per character at 10pt font, 12px height
        let width = (label.len() as f32 * 6.0).max(10.0);
        let height = 12.0;
        (width, height)
    }

    /// Calculate the bounding rectangle of the label
    /// Returns None if no label or no segments (no midpoint)
    pub fn label_bounds(&self) -> Option<Rect> {
        let (label_pos, _) = self.label_position()?;
        let (width, height) = self.label_dimensions();
        // Label is centered horizontally, aligned at bottom of text
        Some(Rect::new(
            label_pos.x - width / 2.0,
            label_pos.y - height,
            label_pos.x + width / 2.0,
            label_pos.y
        ))
    }

    /// Set text_adjoined property
    /// When re-adjoining (setting to true), also resets label_offset
    pub fn set_text_adjoined(&mut self, adjoined: bool) {
        self.text_adjoined = adjoined;
        if adjoined {
            self.label_offset = None;
        }
    }

    /// Check if a point is near the label (for hit testing)
    /// Returns true if the point is within the label bounds
    pub fn is_near_label(&self, p: Point, label_width: f32, label_height: f32) -> bool {
        if let Some((label_pos, _)) = self.label_position() {
            // Label is centered horizontally, aligned at bottom
            let half_width = label_width / 2.0;
            p.x >= label_pos.x - half_width && p.x <= label_pos.x + half_width &&
            p.y >= label_pos.y - label_height && p.y <= label_pos.y
        } else {
            false
        }
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
    fn test_line_segment_near_point_diagonal() {
        // Diagonal line from (110, 130) to (180, 140) - typical connection segment
        let seg = LineSegment::new(Point::new(110.0, 130.0), Point::new(180.0, 140.0));

        // Point very close to the line (about 2 units perpendicular distance)
        assert!(seg.is_near_point(Point::new(145.0, 133.0), 8.0));

        // Point on the line
        assert!(seg.is_near_point(Point::new(145.0, 135.0), 5.0));

        // Point far from the line
        assert!(!seg.is_near_point(Point::new(145.0, 160.0), 8.0));
    }

    #[test]
    fn test_line_segment_near_point_vertical() {
        // Vertical line segment
        let seg = LineSegment::new(Point::new(110.0, 120.0), Point::new(110.0, 130.0));

        // Point on the line
        assert!(seg.is_near_point(Point::new(110.0, 125.0), 5.0));

        // Point near the line
        assert!(seg.is_near_point(Point::new(113.0, 125.0), 5.0));

        // Point far from the line
        assert!(!seg.is_near_point(Point::new(120.0, 125.0), 5.0));
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
