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

    /// Check if this segment intersects a rectangle (for obstacle detection)
    /// Uses parametric line-box intersection
    pub fn intersects_rect(&self, rect: &Rect) -> bool {
        // Quick check: if both endpoints are outside on same side, no intersection
        let dx = self.end.x - self.start.x;
        let dy = self.end.y - self.start.y;

        // Check endpoints
        let p1_inside = rect.contains_point(self.start);
        let p2_inside = rect.contains_point(self.end);
        if p1_inside || p2_inside {
            return true;
        }

        // Parametric intersection: find t where line enters/exits box
        let mut t_min = 0.0f32;
        let mut t_max = 1.0f32;

        // Check X bounds
        if dx.abs() > 0.001 {
            let t1 = (rect.x1 - self.start.x) / dx;
            let t2 = (rect.x2 - self.start.x) / dx;
            let (t_near, t_far) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
            t_min = t_min.max(t_near);
            t_max = t_max.min(t_far);
            if t_min > t_max {
                return false;
            }
        } else {
            // Line is vertical; must be within x bounds
            if self.start.x < rect.x1 || self.start.x > rect.x2 {
                return false;
            }
        }

        // Check Y bounds
        if dy.abs() > 0.001 {
            let t1 = (rect.y1 - self.start.y) / dy;
            let t2 = (rect.y2 - self.start.y) / dy;
            let (t_near, t_far) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
            t_min = t_min.max(t_near);
            t_max = t_max.min(t_far);
            if t_min > t_max {
                return false;
            }
        } else {
            // Line is horizontal; must be within y bounds
            if self.start.y < rect.y1 || self.start.y > rect.y2 {
                return false;
            }
        }

        t_min <= t_max && t_max >= 0.0 && t_min <= 1.0
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

/// A segment that can be either a line or a curve
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PathSegment {
    /// Straight line segment
    Line(LineSegment),
    /// Quadratic Bezier curve (start, control, end)
    QuadraticBezier {
        start: Point,
        control: Point,
        end: Point,
    },
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
    /// User-defined intermediate points along the connection path
    #[serde(default)]
    pub pivot_points: Vec<Point>,
    /// Whether each segment is curved (index 0 = first segment, etc.)
    /// Length should match number of segments (pivot_points.len() + 1)
    #[serde(default)]
    pub segment_curves: Vec<bool>,
    /// Line segments making up the connection path (for hit testing and simple rendering)
    #[serde(skip)]
    pub segments: Vec<LineSegment>,
    /// Path segments including curves (for rendering arcs)
    #[serde(skip)]
    pub path: Vec<PathSegment>,
    /// Whether this connection is selected
    #[serde(skip)]
    pub selected: bool,
    /// Whether the label is being hovered/selected (for interaction)
    #[serde(skip)]
    pub label_selected: bool,
    /// Sequential ID for this connection (e.g., "conn0001")
    #[serde(default)]
    pub seq_id: String,
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
            pivot_points: Vec::new(),
            segment_curves: Vec::new(),
            segments: Vec::new(),
            path: Vec::new(),
            selected: false,
            label_selected: false,
            seq_id: String::new(),
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
            pivot_points: Vec::new(),
            segment_curves: Vec::new(),
            segments: Vec::new(),
            path: Vec::new(),
            selected: false,
            label_selected: false,
            seq_id: String::new(),
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

    /// Determine the best side for a connection endpoint based on the direction to a reference point
    /// Uses 45-degree quadrants: if the reference point is more above/below than left/right, use Top/Bottom
    fn determine_side_from_direction(node_center: Point, reference_point: Point) -> Side {
        let dx = reference_point.x - node_center.x;
        let dy = reference_point.y - node_center.y;

        // Use 45-degree quadrants
        if dx.abs() > dy.abs() {
            // More horizontal than vertical
            if dx > 0.0 { Side::Right } else { Side::Left }
        } else {
            // More vertical than horizontal (or equal)
            if dy > 0.0 { Side::Bottom } else { Side::Top }
        }
    }

    /// Calculate the line segments for this connection using pivot points
    /// Auto-adjusts source/target side based on the direction to first/last pivot point
    pub fn calculate_segments(
        &mut self,
        source_bounds: &Rect,
        target_bounds: &Rect,
        _stub_len: f32,
    ) {
        self.segments.clear();
        self.path.clear();

        // Auto-adjust sides based on pivot points or target/source positions
        if !self.pivot_points.is_empty() {
            // Use first pivot to determine source side
            let first_pivot = self.pivot_points[0];
            self.source_side = Self::determine_side_from_direction(source_bounds.center(), first_pivot);
            self.source_offset = 0.0; // Reset offset when auto-adjusting

            // Use last pivot to determine target side
            let last_pivot = self.pivot_points[self.pivot_points.len() - 1];
            self.target_side = Self::determine_side_from_direction(target_bounds.center(), last_pivot);
            self.target_offset = 0.0; // Reset offset when auto-adjusting
        } else {
            // No pivot points - use direction to opposite node
            self.source_side = Self::determine_side_from_direction(source_bounds.center(), target_bounds.center());
            self.target_side = Self::determine_side_from_direction(target_bounds.center(), source_bounds.center());
        }

        let source_point = self.get_side_point(source_bounds, self.source_side, self.source_offset);
        let target_point = self.get_side_point(target_bounds, self.target_side, self.target_offset);

        // Build list of all points: source -> pivots -> target
        let mut points = vec![source_point];
        points.extend(self.pivot_points.iter().cloned());
        points.push(target_point);

        // Ensure segment_curves has correct length
        let num_segments = points.len() - 1;
        while self.segment_curves.len() < num_segments {
            self.segment_curves.push(false); // Default to straight
        }
        self.segment_curves.truncate(num_segments);

        // Create segments between consecutive points
        for i in 0..num_segments {
            let start = points[i];
            let end = points[i + 1];

            if self.segment_curves.get(i).copied().unwrap_or(false) {
                // Curved segment - create Bezier
                let control = self.calculate_curve_control(start, end, i);
                self.path.push(PathSegment::QuadraticBezier { start, control, end });
                self.approximate_bezier_as_lines(start, control, end, 8);
            } else {
                // Straight segment
                let segment = LineSegment::new(start, end);
                self.segments.push(segment);
                self.path.push(PathSegment::Line(segment));
            }
        }
    }

    /// Calculate control point for a curved segment
    fn calculate_curve_control(&self, start: Point, end: Point, seg_idx: usize) -> Point {
        // Control point perpendicular to segment at midpoint
        let mid_x = (start.x + end.x) / 2.0;
        let mid_y = (start.y + end.y) / 2.0;
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let offset = (len * 0.3).max(20.0);

        // Perpendicular direction
        let (perp_x, perp_y) = (-dy / len, dx / len);

        // Alternate curve direction for adjacent segments
        let dir = if seg_idx % 2 == 0 { 1.0 } else { -1.0 };

        Point::new(mid_x + perp_x * offset * dir, mid_y + perp_y * offset * dir)
    }

    /// Approximate a quadratic Bezier as line segments for hit testing
    fn approximate_bezier_as_lines(&mut self, start: Point, control: Point, end: Point, steps: usize) {
        for i in 0..steps {
            let t0 = i as f32 / steps as f32;
            let t1 = (i + 1) as f32 / steps as f32;

            let p0 = Self::quadratic_bezier_point(start, control, end, t0);
            let p1 = Self::quadratic_bezier_point(start, control, end, t1);

            self.segments.push(LineSegment::new(p0, p1));
        }
    }

    /// Calculate point on quadratic Bezier curve at parameter t
    fn quadratic_bezier_point(start: Point, control: Point, end: Point, t: f32) -> Point {
        let inv_t = 1.0 - t;
        Point::new(
            inv_t * inv_t * start.x + 2.0 * inv_t * t * control.x + t * t * end.x,
            inv_t * inv_t * start.y + 2.0 * inv_t * t * control.y + t * t * end.y,
        )
    }

    /// Returns index of pivot point near the given position, or None
    pub fn find_pivot_at(&self, pos: Point, tolerance: f32) -> Option<usize> {
        self.pivot_points.iter()
            .position(|p| {
                let dx = p.x - pos.x;
                let dy = p.y - pos.y;
                (dx * dx + dy * dy).sqrt() <= tolerance
            })
    }

    /// Returns which segment index a point is near (for inserting pivots)
    pub fn find_segment_at(&self, pos: Point, tolerance: f32) -> usize {
        for (i, seg) in self.segments.iter().enumerate() {
            if seg.is_near_point(pos, tolerance) {
                return i;
            }
        }
        0 // Default to first segment
    }

    /// Check if position is near the source or target attachment point
    /// Returns Some(true) for source, Some(false) for target, None if neither
    pub fn find_endpoint_at(&self, pos: Point, source_bounds: &Rect, target_bounds: &Rect, tolerance: f32) -> Option<bool> {
        let source_pt = self.get_side_point(source_bounds, self.source_side, self.source_offset);
        let target_pt = self.get_side_point(target_bounds, self.target_side, self.target_offset);

        let src_dist = ((pos.x - source_pt.x).powi(2) + (pos.y - source_pt.y).powi(2)).sqrt();
        let tgt_dist = ((pos.x - target_pt.x).powi(2) + (pos.y - target_pt.y).powi(2)).sqrt();

        if src_dist <= tolerance {
            Some(true)  // Source endpoint
        } else if tgt_dist <= tolerance {
            Some(false)  // Target endpoint
        } else {
            None
        }
    }

    /// Get the point on a side of a rectangle for connection attachment
    /// This is made public so renderer can access it for drawing handles
    pub fn get_side_point(&self, bounds: &Rect, side: Side, offset: f32) -> Point {
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
