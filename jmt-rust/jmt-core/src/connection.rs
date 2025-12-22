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

/// Connection routing style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RoutingStyle {
    /// Straight line directly between source and target
    Direct,
    /// Orthogonal routing with automatic obstacle avoidance (default)
    #[default]
    OrthogonalAuto,
    /// U-shape: exits and returns from the same direction
    UShape,
    /// L-shape: single right-angle turn
    LShape,
    /// S-shape (Step): two right-angle turns
    SShape,
    /// Smooth curved arc using Bezier curve
    Arc,
}

impl RoutingStyle {
    /// Get display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Direct => "Direct",
            Self::OrthogonalAuto => "Orthogonal (Auto)",
            Self::UShape => "U-Shape",
            Self::LShape => "L-Shape",
            Self::SShape => "S-Shape (Step)",
            Self::Arc => "Arc",
        }
    }

    /// Get all variants for UI iteration
    pub fn all() -> &'static [RoutingStyle] {
        &[
            Self::Direct,
            Self::OrthogonalAuto,
            Self::UShape,
            Self::LShape,
            Self::SShape,
            Self::Arc,
        ]
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
    /// Routing style for this connection
    #[serde(default)]
    pub routing_style: RoutingStyle,
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
            routing_style: RoutingStyle::OrthogonalAuto,
            segments: Vec::new(),
            path: Vec::new(),
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
            routing_style: RoutingStyle::OrthogonalAuto,
            segments: Vec::new(),
            path: Vec::new(),
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

    /// Calculate the line segments for this connection based on routing style
    pub fn calculate_segments(
        &mut self,
        source_bounds: &Rect,
        target_bounds: &Rect,
        stub_len: f32,
    ) {
        self.segments.clear();
        self.path.clear();

        match self.routing_style {
            RoutingStyle::Direct => {
                self.calculate_direct_segments(source_bounds, target_bounds);
            }
            RoutingStyle::LShape => {
                self.calculate_lshape_segments(source_bounds, target_bounds);
            }
            RoutingStyle::SShape | RoutingStyle::OrthogonalAuto => {
                // OrthogonalAuto uses S-shape by default; obstacle avoidance handled separately
                self.calculate_sshape_segments(source_bounds, target_bounds, stub_len);
            }
            RoutingStyle::UShape => {
                self.calculate_ushape_segments(source_bounds, target_bounds, stub_len);
            }
            RoutingStyle::Arc => {
                self.calculate_arc_segments(source_bounds, target_bounds, stub_len);
            }
        }

        // Populate path from segments for non-Arc styles
        if self.path.is_empty() {
            for seg in &self.segments {
                self.path.push(PathSegment::Line(*seg));
            }
        }
    }

    /// Direct routing: straight line between source and target
    fn calculate_direct_segments(&mut self, source_bounds: &Rect, target_bounds: &Rect) {
        let source_point = self.get_side_point(source_bounds, self.source_side, self.source_offset);
        let target_point = self.get_side_point(target_bounds, self.target_side, self.target_offset);

        self.segments.push(LineSegment::new(source_point, target_point));
    }

    /// L-shape routing: single right-angle turn
    fn calculate_lshape_segments(&mut self, source_bounds: &Rect, target_bounds: &Rect) {
        let source_point = self.get_side_point(source_bounds, self.source_side, self.source_offset);
        let target_point = self.get_side_point(target_bounds, self.target_side, self.target_offset);

        // Determine corner point based on source side orientation
        let corner = if self.source_side.is_vertical() {
            // Source exits top/bottom, turn horizontally to reach target
            Point::new(source_point.x, target_point.y)
        } else {
            // Source exits left/right, turn vertically to reach target
            Point::new(target_point.x, source_point.y)
        };

        self.segments.push(LineSegment::new(source_point, corner));
        self.segments.push(LineSegment::new(corner, target_point));
    }

    /// S-shape routing: two right-angle turns (stub-based)
    fn calculate_sshape_segments(&mut self, source_bounds: &Rect, target_bounds: &Rect, stub_len: f32) {
        let source_point = self.get_side_point(source_bounds, self.source_side, self.source_offset);
        let target_point = self.get_side_point(target_bounds, self.target_side, self.target_offset);
        let source_stub = self.get_stub_point(source_point, self.source_side, stub_len);
        let target_stub = self.get_stub_point(target_point, self.target_side, stub_len);

        self.segments.push(LineSegment::new(source_point, source_stub));
        self.segments.push(LineSegment::new(source_stub, target_stub));
        self.segments.push(LineSegment::new(target_stub, target_point));
    }

    /// U-shape routing: exits and returns from same direction
    fn calculate_ushape_segments(&mut self, source_bounds: &Rect, target_bounds: &Rect, stub_len: f32) {
        let source_point = self.get_side_point(source_bounds, self.source_side, self.source_offset);
        let target_point = self.get_side_point(target_bounds, self.target_side, self.target_offset);
        let source_stub = self.get_stub_point(source_point, self.source_side, stub_len);
        let target_stub = self.get_stub_point(target_point, self.target_side, stub_len);

        let bulge_distance = stub_len * 2.0;

        let (mid1, mid2) = match self.source_side {
            Side::Top => {
                let y = source_stub.y.min(target_stub.y) - bulge_distance;
                (Point::new(source_stub.x, y), Point::new(target_stub.x, y))
            }
            Side::Bottom => {
                let y = source_stub.y.max(target_stub.y) + bulge_distance;
                (Point::new(source_stub.x, y), Point::new(target_stub.x, y))
            }
            Side::Left => {
                let x = source_stub.x.min(target_stub.x) - bulge_distance;
                (Point::new(x, source_stub.y), Point::new(x, target_stub.y))
            }
            Side::Right => {
                let x = source_stub.x.max(target_stub.x) + bulge_distance;
                (Point::new(x, source_stub.y), Point::new(x, target_stub.y))
            }
            Side::None => (source_stub, target_stub),
        };

        self.segments.push(LineSegment::new(source_point, source_stub));
        self.segments.push(LineSegment::new(source_stub, mid1));
        self.segments.push(LineSegment::new(mid1, mid2));
        self.segments.push(LineSegment::new(mid2, target_stub));
        self.segments.push(LineSegment::new(target_stub, target_point));
    }

    /// Arc routing: smooth quadratic Bezier curve
    fn calculate_arc_segments(&mut self, source_bounds: &Rect, target_bounds: &Rect, _stub_len: f32) {
        let source_point = self.get_side_point(source_bounds, self.source_side, self.source_offset);
        let target_point = self.get_side_point(target_bounds, self.target_side, self.target_offset);

        // Calculate the midpoint between source and target
        let mid_x = (source_point.x + target_point.x) / 2.0;
        let mid_y = (source_point.y + target_point.y) / 2.0;

        // Calculate perpendicular offset for control point
        let dx = target_point.x - source_point.x;
        let dy = target_point.y - source_point.y;
        let length = (dx * dx + dy * dy).sqrt();

        // Offset perpendicular to the line, proportional to length (min 30px)
        let offset = (length * 0.3).max(30.0);

        // Perpendicular direction (rotate 90 degrees)
        // For horizontal connections, this creates vertical bulge
        // For vertical connections, this creates horizontal bulge
        let (perp_x, perp_y) = if length > 0.001 {
            (-dy / length, dx / length)
        } else {
            (0.0, -1.0)
        };

        // Control point is midpoint offset perpendicular to the line
        // Use the stub directions to determine which way to curve
        let curve_direction = match (self.source_side, self.target_side) {
            (Side::Right, Side::Left) | (Side::Left, Side::Right) => -1.0, // Curve upward for horizontal
            (Side::Bottom, Side::Top) | (Side::Top, Side::Bottom) => -1.0, // Curve left for vertical
            _ => 1.0,
        };

        let control = Point::new(
            mid_x + perp_x * offset * curve_direction,
            mid_y + perp_y * offset * curve_direction,
        );

        // Add the Bezier curve to path
        self.path.push(PathSegment::QuadraticBezier {
            start: source_point,
            control,
            end: target_point,
        });

        // Approximate as line segments for hit testing
        self.approximate_bezier_as_lines(source_point, control, target_point, 8);
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

    /// Calculate segments with obstacle avoidance for OrthogonalAuto routing
    pub fn calculate_segments_with_obstacles(
        &mut self,
        source_bounds: &Rect,
        target_bounds: &Rect,
        obstacles: &[Rect],
        stub_len: f32,
    ) {
        self.segments.clear();
        self.path.clear();

        let source_point = self.get_side_point(source_bounds, self.source_side, self.source_offset);
        let target_point = self.get_side_point(target_bounds, self.target_side, self.target_offset);
        let source_stub = self.get_stub_point(source_point, self.source_side, stub_len);
        let target_stub = self.get_stub_point(target_point, self.target_side, stub_len);

        // Try simple S-shape first
        let simple_segments = vec![
            LineSegment::new(source_point, source_stub),
            LineSegment::new(source_stub, target_stub),
            LineSegment::new(target_stub, target_point),
        ];

        // Check if the middle segment (stub to stub) intersects any obstacle
        let middle_seg = &simple_segments[1];
        let has_obstacle = obstacles.iter().any(|obs| middle_seg.intersects_rect(obs));

        if !has_obstacle {
            // No obstacle, use simple S-shape
            self.segments = simple_segments;
        } else {
            // Need to route around obstacles
            self.calculate_obstacle_avoiding_route(
                source_point, source_stub, target_stub, target_point,
                obstacles, stub_len
            );
        }

        // Populate path from segments
        for seg in &self.segments {
            self.path.push(PathSegment::Line(*seg));
        }
    }

    /// Route around obstacles using waypoints
    fn calculate_obstacle_avoiding_route(
        &mut self,
        source_point: Point,
        source_stub: Point,
        target_stub: Point,
        target_point: Point,
        obstacles: &[Rect],
        stub_len: f32,
    ) {
        const MARGIN: f32 = 15.0;

        // Collect potential waypoints from obstacle corners (with margin)
        let mut waypoints = Vec::new();
        for obs in obstacles {
            waypoints.push(Point::new(obs.x1 - MARGIN, obs.y1 - MARGIN)); // top-left
            waypoints.push(Point::new(obs.x2 + MARGIN, obs.y1 - MARGIN)); // top-right
            waypoints.push(Point::new(obs.x1 - MARGIN, obs.y2 + MARGIN)); // bottom-left
            waypoints.push(Point::new(obs.x2 + MARGIN, obs.y2 + MARGIN)); // bottom-right
        }

        // Add stub endpoints as potential points
        waypoints.push(source_stub);
        waypoints.push(target_stub);

        // Find the best path through waypoints using a greedy approach
        // This is a simplified A* - for better results, a full pathfinding could be used
        let path = self.find_orthogonal_path(
            source_stub, target_stub, &waypoints, obstacles, stub_len
        );

        // Build segments: source -> source_stub -> path -> target_stub -> target
        self.segments.push(LineSegment::new(source_point, source_stub));

        if path.is_empty() {
            // No path found, fall back to direct stub-to-stub
            self.segments.push(LineSegment::new(source_stub, target_stub));
        } else {
            let mut prev = source_stub;
            for &wp in &path {
                // Route orthogonally to waypoint
                if (prev.x - wp.x).abs() > 0.5 && (prev.y - wp.y).abs() > 0.5 {
                    // Need two segments for orthogonal routing
                    let mid = Point::new(wp.x, prev.y);
                    self.segments.push(LineSegment::new(prev, mid));
                    self.segments.push(LineSegment::new(mid, wp));
                } else {
                    self.segments.push(LineSegment::new(prev, wp));
                }
                prev = wp;
            }
            // Final leg to target_stub
            if (prev.x - target_stub.x).abs() > 0.5 && (prev.y - target_stub.y).abs() > 0.5 {
                let mid = Point::new(target_stub.x, prev.y);
                self.segments.push(LineSegment::new(prev, mid));
                self.segments.push(LineSegment::new(mid, target_stub));
            } else if prev != target_stub {
                self.segments.push(LineSegment::new(prev, target_stub));
            }
        }

        self.segments.push(LineSegment::new(target_stub, target_point));
    }

    /// Find an orthogonal path from start to end avoiding obstacles
    /// Returns a list of waypoints to visit
    fn find_orthogonal_path(
        &self,
        start: Point,
        end: Point,
        waypoints: &[Point],
        obstacles: &[Rect],
        _stub_len: f32,
    ) -> Vec<Point> {
        // Simple greedy approach: find the waypoint that gets us closest to the end
        // while avoiding obstacles

        let mut path = Vec::new();
        let mut current = start;
        let mut visited = std::collections::HashSet::new();

        const MAX_ITERATIONS: usize = 10;

        for _ in 0..MAX_ITERATIONS {
            // Check if we can go directly to end
            if self.can_reach_orthogonally(current, end, obstacles) {
                return path;
            }

            // Find best waypoint
            let mut best_waypoint = None;
            let mut best_distance = f32::MAX;

            for &wp in waypoints {
                // Skip if already visited or same as current
                let key = (wp.x as i32, wp.y as i32);
                if visited.contains(&key) {
                    continue;
                }
                if (wp.x - current.x).abs() < 1.0 && (wp.y - current.y).abs() < 1.0 {
                    continue;
                }

                // Check if we can reach this waypoint
                if self.can_reach_orthogonally(current, wp, obstacles) {
                    let dist_to_wp = current.distance_to(wp);
                    let dist_from_wp = wp.distance_to(end);
                    let total_dist = dist_to_wp + dist_from_wp;

                    if total_dist < best_distance {
                        best_distance = total_dist;
                        best_waypoint = Some(wp);
                    }
                }
            }

            match best_waypoint {
                Some(wp) => {
                    path.push(wp);
                    visited.insert((wp.x as i32, wp.y as i32));
                    current = wp;
                }
                None => break, // No valid waypoint found
            }
        }

        path
    }

    /// Check if two points can be connected orthogonally without hitting obstacles
    fn can_reach_orthogonally(&self, from: Point, to: Point, obstacles: &[Rect]) -> bool {
        // Try horizontal then vertical
        let mid1 = Point::new(to.x, from.y);
        let seg1a = LineSegment::new(from, mid1);
        let seg1b = LineSegment::new(mid1, to);
        let path1_clear = !obstacles.iter().any(|o| seg1a.intersects_rect(o) || seg1b.intersects_rect(o));

        if path1_clear {
            return true;
        }

        // Try vertical then horizontal
        let mid2 = Point::new(from.x, to.y);
        let seg2a = LineSegment::new(from, mid2);
        let seg2b = LineSegment::new(mid2, to);
        let path2_clear = !obstacles.iter().any(|o| seg2a.intersects_rect(o) || seg2b.intersects_rect(o));

        path2_clear
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
