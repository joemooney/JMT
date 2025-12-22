//! Main application state and update loop

use eframe::egui;
use jmt_core::{Diagram, EditMode, NodeType, DiagramType};
use jmt_core::geometry::{Point, Rect};
use jmt_core::node::{Corner, NodeId};
use std::time::Instant;

use crate::canvas::DiagramCanvas;
use crate::panels::{MenuBar, Toolbar, PropertiesPanel, StatusBar};

/// State for rectangular marquee selection
#[derive(Debug, Clone, Default)]
pub struct SelectionRect {
    /// Starting point of the selection (where drag began)
    pub start: Option<egui::Pos2>,
    /// Current end point (current mouse position during drag)
    pub current: Option<egui::Pos2>,
}

impl SelectionRect {
    /// Check if a selection is active
    pub fn is_active(&self) -> bool {
        self.start.is_some() && self.current.is_some()
    }

    /// Get the selection rectangle as an egui Rect
    pub fn to_egui_rect(&self) -> Option<egui::Rect> {
        if let (Some(start), Some(current)) = (self.start, self.current) {
            Some(egui::Rect::from_two_pos(start, current))
        } else {
            None
        }
    }

    /// Get the selection rectangle as a core Rect
    pub fn to_core_rect(&self) -> Option<Rect> {
        if let (Some(start), Some(current)) = (self.start, self.current) {
            let min_x = start.x.min(current.x);
            let min_y = start.y.min(current.y);
            let max_x = start.x.max(current.x);
            let max_y = start.y.max(current.y);
            Some(Rect::new(min_x, min_y, max_x, max_y))
        } else {
            None
        }
    }

    /// Clear the selection
    pub fn clear(&mut self) {
        self.start = None;
        self.current = None;
    }
}

/// State for resizing a node by its corner
#[derive(Debug, Clone, Default)]
pub struct ResizeState {
    /// The node being resized
    pub node_id: Option<NodeId>,
    /// Which corner is being dragged
    pub corner: Corner,
}

impl ResizeState {
    /// Check if a resize is active
    pub fn is_active(&self) -> bool {
        self.node_id.is_some() && self.corner != Corner::None
    }

    /// Start a resize operation
    pub fn start(&mut self, node_id: NodeId, corner: Corner) {
        self.node_id = Some(node_id);
        self.corner = corner;
    }

    /// Clear the resize state
    pub fn clear(&mut self) {
        self.node_id = None;
        self.corner = Corner::None;
    }
}

/// Type of pivot/endpoint drag operation
#[derive(Debug, Clone, Copy)]
enum PivotDragType {
    /// Dragging a pivot point (index in pivot_points)
    Pivot(usize),
    /// Dragging the source endpoint
    Source,
    /// Dragging the target endpoint
    Target,
}

/// Represents a clickable/selectable candidate at a point
#[derive(Debug, Clone, Copy, PartialEq)]
enum ClickCandidate {
    /// A node
    Node(NodeId),
    /// A connection endpoint (connection_id, is_source)
    Endpoint(uuid::Uuid, bool),
    /// A pivot point (connection_id, pivot_index)
    Pivot(uuid::Uuid, usize),
    /// A connection label
    Label(uuid::Uuid),
    /// A connection line
    Connection(uuid::Uuid),
}

/// Selection disambiguation mode when multiple items overlap
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SelectionMode {
    /// Click repeatedly to cycle through overlapping items
    #[default]
    Cycle,
    /// Show a magnifying loupe to pick from overlapping items
    Loupe,
}

/// State for the magnifying loupe
#[derive(Debug, Clone, Default)]
pub struct LoupeState {
    /// Whether the loupe is currently visible
    pub visible: bool,
    /// Center position of the loupe in diagram coordinates
    pub center: Option<Point>,
    /// Candidates available for selection in the loupe
    pub candidates: Vec<ClickCandidate>,
    /// Radius of the source area being magnified (in diagram coordinates)
    pub source_radius: f32,
    /// Radius of the loupe display (in screen coordinates)
    pub display_radius: f32,
    /// Magnification factor
    pub magnification: f32,
    /// Whether the loupe just opened this frame (skip click-outside detection)
    pub just_opened: bool,
}

impl LoupeState {
    pub fn new() -> Self {
        Self {
            visible: false,
            center: None,
            candidates: Vec::new(),
            source_radius: 30.0,  // Area to magnify
            display_radius: 120.0, // Size of loupe on screen
            magnification: 4.0,
            just_opened: false,
        }
    }

    pub fn show(&mut self, center: Point, candidates: Vec<ClickCandidate>) {
        self.visible = true;
        self.center = Some(center);
        self.candidates = candidates;
        self.just_opened = true; // Skip click-outside detection this frame
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.center = None;
        self.candidates.clear();
        self.just_opened = false;
    }

    /// Call at the start of each frame to clear the just_opened flag
    pub fn begin_frame(&mut self) {
        self.just_opened = false;
    }
}

/// Application-wide settings (configurable by user)
#[derive(Debug, Clone)]
pub struct AppSettings {
    // Selection settings
    /// Radius for detecting overlapping items (pixels in diagram space)
    pub selection_sensitivity: f32,
    /// Hit tolerance for pivot points and endpoints
    pub pivot_hit_tolerance: f32,
    /// Hit tolerance for connection lines
    pub connection_hit_tolerance: f32,
    /// Hit margin for resize corners
    pub corner_hit_margin: f32,
    /// Distance threshold for click cycling
    pub click_cycle_distance: f32,

    // Loupe settings
    /// Size of the loupe popup (screen pixels)
    pub loupe_display_radius: f32,

    // Double-click settings
    /// Double-click time threshold (milliseconds)
    pub double_click_time_ms: u128,
    /// Maximum distance for double-click detection
    pub double_click_distance: f32,

    // Visual settings
    /// Show debug info in status bar
    pub show_debug_info: bool,
    /// Highlight nodes on hover
    pub highlight_on_hover: bool,

    // Grid settings
    /// Enable grid snapping
    pub snap_to_grid: bool,
    /// Grid size (pixels)
    pub grid_size: f32,
    /// Show grid
    pub show_grid: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            // Selection
            selection_sensitivity: 12.0,
            pivot_hit_tolerance: 12.0,
            connection_hit_tolerance: 12.0,
            corner_hit_margin: 15.0,
            click_cycle_distance: 15.0,

            // Loupe
            loupe_display_radius: 120.0,

            // Double-click
            double_click_time_ms: 500,
            double_click_distance: 10.0,

            // Visual
            show_debug_info: false,
            highlight_on_hover: true,

            // Grid
            snap_to_grid: false,
            grid_size: 10.0,
            show_grid: false,
        }
    }
}

/// State for a single open diagram
pub struct DiagramState {
    pub diagram: Diagram,
    pub canvas: DiagramCanvas,
    pub modified: bool,
    /// File path where this diagram is saved (None if not yet saved)
    pub file_path: Option<std::path::PathBuf>,
}

impl DiagramState {
    pub fn new(diagram: Diagram) -> Self {
        Self {
            canvas: DiagramCanvas::new(),
            diagram,
            modified: false,
            file_path: None,
        }
    }

    /// Create with an existing file path (for opened files)
    pub fn with_path(mut diagram: Diagram, path: std::path::PathBuf) -> Self {
        // Ensure the diagram settings also have the path
        diagram.settings.file_path = Some(path.to_string_lossy().to_string());
        Self {
            canvas: DiagramCanvas::new(),
            diagram,
            modified: false,
            file_path: Some(path),
        }
    }
}

/// Minimum zoom level (25%)
const MIN_ZOOM: f32 = 0.25;
/// Maximum zoom level (400%)
const MAX_ZOOM: f32 = 4.0;
/// Zoom step for buttons
const ZOOM_STEP: f32 = 0.1;
/// Zoom step for mouse wheel
const ZOOM_WHEEL_STEP: f32 = 0.1;
/// Minimum distance between pivot points
const MIN_PIVOT_DISTANCE: f32 = 20.0;

/// The main JMT application
pub struct JmtApp {
    /// Open diagrams
    pub diagrams: Vec<DiagramState>,
    /// Currently active diagram index
    active_diagram: usize,
    /// Current edit mode
    pub edit_mode: EditMode,
    /// Status message
    status_message: String,
    /// Pending connection source (when in Connect mode)
    pending_connection_source: Option<uuid::Uuid>,
    /// Active selection rectangle for marquee selection
    pub selection_rect: SelectionRect,
    /// Whether we're currently dragging nodes (vs marquee selecting)
    dragging_nodes: bool,
    /// Connection ID of the label we're currently dragging
    dragging_label: Option<uuid::Uuid>,
    /// Region separator being dragged: (state_id, region_index)
    /// region_index refers to the region whose top edge is being dragged
    dragging_separator: Option<(uuid::Uuid, usize)>,
    /// Pivot point being dragged: (connection_id, pivot_index)
    dragging_pivot: Option<(uuid::Uuid, usize)>,
    /// Endpoint being dragged: (connection_id, is_source)
    dragging_endpoint: Option<(uuid::Uuid, bool)>,
    /// Selected pivot point for deletion: (connection_id, pivot_index)
    selected_pivot: Option<(uuid::Uuid, usize)>,
    /// Current cursor position on canvas (for preview rendering)
    pub cursor_pos: Option<egui::Pos2>,
    /// Active resize state (when resizing a node by corner)
    resize_state: ResizeState,
    /// Lasso selection points (freeform polygon)
    lasso_points: Vec<egui::Pos2>,
    /// Time of last click (for custom double-click detection)
    last_click_time: Option<Instant>,
    /// Position of last click (for custom double-click detection)
    last_click_pos: Option<egui::Pos2>,
    /// Current zoom level (1.0 = 100%)
    pub zoom_level: f32,
    /// State ID for sub-statemachine preview popup (None = no preview shown)
    preview_substatemachine: Option<uuid::Uuid>,
    /// Click cycling: list of candidates at the last click position
    click_cycle_candidates: Vec<ClickCandidate>,
    /// Click cycling: current index in the candidates list
    click_cycle_index: usize,
    /// Click cycling: position where the cycle started (in diagram coordinates)
    click_cycle_pos: Option<Point>,
    /// Selection disambiguation mode
    pub selection_mode: SelectionMode,
    /// Magnifying loupe state
    loupe: LoupeState,
    /// Application settings
    pub settings: AppSettings,
    /// Whether to show the settings window
    pub show_settings_window: bool,
}

impl Default for JmtApp {
    fn default() -> Self {
        // Create a default diagram to start with
        let diagram = Diagram::new("Untitled");
        let diagram_state = DiagramState::new(diagram);

        Self {
            diagrams: vec![diagram_state],
            active_diagram: 0,
            edit_mode: EditMode::Arrow,
            status_message: String::from("Ready"),
            pending_connection_source: None,
            selection_rect: SelectionRect::default(),
            dragging_nodes: false,
            dragging_label: None,
            dragging_separator: None,
            dragging_pivot: None,
            dragging_endpoint: None,
            selected_pivot: None,
            cursor_pos: None,
            resize_state: ResizeState::default(),
            lasso_points: Vec::new(),
            last_click_time: None,
            last_click_pos: None,
            zoom_level: 1.0,
            preview_substatemachine: None,
            click_cycle_candidates: Vec::new(),
            click_cycle_index: 0,
            click_cycle_pos: None,
            selection_mode: SelectionMode::default(),
            loupe: LoupeState::new(),
            settings: AppSettings::default(),
            show_settings_window: false,
        }
    }
}

impl JmtApp {
    /// Create a new application instance
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    /// Get the current diagram
    pub fn current_diagram(&self) -> Option<&DiagramState> {
        self.diagrams.get(self.active_diagram)
    }

    /// Get the current diagram mutably
    pub fn current_diagram_mut(&mut self) -> Option<&mut DiagramState> {
        self.diagrams.get_mut(self.active_diagram)
    }

    /// Get the current diagram index (if any diagrams are open)
    pub fn current_diagram_idx(&self) -> Option<usize> {
        if self.diagrams.is_empty() {
            None
        } else {
            Some(self.active_diagram)
        }
    }

    /// Check if a point is on a pivot point or endpoint of a connection
    /// Returns (connection_id, drag_type) if found
    /// First checks the selected connection, then checks all connections for endpoints
    fn check_pivot_or_endpoint_at(&self, pos: Point) -> Option<(uuid::Uuid, PivotDragType)> {
        let state = self.current_diagram()?;

        let pivot_tolerance = self.settings.pivot_hit_tolerance;

        // First, check the selected connection for pivot points and endpoints
        if let Some(conn_id) = state.diagram.selected_connection() {
            if let Some(conn) = state.diagram.find_connection(conn_id) {
                // First check pivot points (gold circles) - only for selected connection
                if let Some(pivot_idx) = conn.find_pivot_at(pos, pivot_tolerance) {
                    return Some((conn_id, PivotDragType::Pivot(pivot_idx)));
                }

                // Check endpoints
                if let (Some(source_node), Some(target_node)) = (
                    state.diagram.find_node(conn.source_id),
                    state.diagram.find_node(conn.target_id),
                ) {
                    let source_bounds = source_node.bounds();
                    let target_bounds = target_node.bounds();

                    if let Some(is_source) = conn.find_endpoint_at(pos, source_bounds, target_bounds, pivot_tolerance) {
                        if is_source {
                            return Some((conn_id, PivotDragType::Source));
                        } else {
                            return Some((conn_id, PivotDragType::Target));
                        }
                    }
                }
            }
        }

        // If not found on selected connection, check all connections for endpoint hits
        // This allows clicking directly on any connection's endpoint to start dragging
        for conn in state.diagram.connections() {
            if let (Some(source_node), Some(target_node)) = (
                state.diagram.find_node(conn.source_id),
                state.diagram.find_node(conn.target_id),
            ) {
                let source_bounds = source_node.bounds();
                let target_bounds = target_node.bounds();

                if let Some(is_source) = conn.find_endpoint_at(pos, source_bounds, target_bounds, pivot_tolerance) {
                    if is_source {
                        return Some((conn.id, PivotDragType::Source));
                    } else {
                        return Some((conn.id, PivotDragType::Target));
                    }
                }
            }
        }

        None
    }

    /// Find all clickable candidates at a point for selection cycling
    /// Returns candidates in priority order (endpoints first, then nodes sorted by size)
    fn find_all_candidates_at(&self, pos: Point) -> Vec<ClickCandidate> {
        let mut candidates = Vec::new();
        let Some(state) = self.current_diagram() else {
            return candidates;
        };

        let pivot_tolerance = self.settings.pivot_hit_tolerance;
        let connection_tolerance = self.settings.connection_hit_tolerance;

        eprintln!("[CANDIDATES] Checking at ({:.1}, {:.1})", pos.x, pos.y);

        // 1. Check pivot points on selected connection
        if let Some(conn_id) = state.diagram.selected_connection() {
            if let Some(conn) = state.diagram.find_connection(conn_id) {
                for (idx, pivot) in conn.pivot_points.iter().enumerate() {
                    let dx = pos.x - pivot.x;
                    let dy = pos.y - pivot.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist <= pivot_tolerance {
                        eprintln!("[CANDIDATES] Pivot {} hit (dist: {:.1})", idx, dist);
                        candidates.push(ClickCandidate::Pivot(conn_id, idx));
                    }
                }
            }
        }

        // 2. Check connection endpoints
        for conn in state.diagram.connections() {
            if let (Some(source_node), Some(target_node)) = (
                state.diagram.find_node(conn.source_id),
                state.diagram.find_node(conn.target_id),
            ) {
                let source_bounds = source_node.bounds();
                let target_bounds = target_node.bounds();

                // Check source endpoint
                let source_pt = conn.get_side_point(source_bounds, conn.source_side, conn.source_offset);
                let dx = pos.x - source_pt.x;
                let dy = pos.y - source_pt.y;
                let dist = (dx * dx + dy * dy).sqrt();
                eprintln!("[CANDIDATES] Source endpoint at ({:.1}, {:.1}), dist: {:.1}, tolerance: {}",
                    source_pt.x, source_pt.y, dist, pivot_tolerance);
                if dist <= pivot_tolerance {
                    candidates.push(ClickCandidate::Endpoint(conn.id, true));
                }

                // Check target endpoint
                let target_pt = conn.get_side_point(target_bounds, conn.target_side, conn.target_offset);
                let dx = pos.x - target_pt.x;
                let dy = pos.y - target_pt.y;
                let dist = (dx * dx + dy * dy).sqrt();
                eprintln!("[CANDIDATES] Target endpoint at ({:.1}, {:.1}), dist: {:.1}, tolerance: {}",
                    target_pt.x, target_pt.y, dist, pivot_tolerance);
                if dist <= pivot_tolerance {
                    candidates.push(ClickCandidate::Endpoint(conn.id, false));
                }
            }
        }

        // 3. Check connection labels
        for conn in state.diagram.connections() {
            if let Some(bounds) = conn.label_bounds() {
                if bounds.contains_point(pos) {
                    candidates.push(ClickCandidate::Label(conn.id));
                }
            }
        }

        // 4. Check nodes - collect all containing nodes with their areas
        let mut node_candidates: Vec<(NodeId, f32, String)> = Vec::new();
        for node in state.diagram.nodes() {
            let bounds = node.bounds();
            let contains = node.contains_point(pos);
            eprintln!("[CANDIDATES] Node '{}' bounds: ({:.1},{:.1})-({:.1},{:.1}), contains: {}",
                node.name(), bounds.x1, bounds.y1, bounds.x2, bounds.y2, contains);
            if contains {
                let area = bounds.width() * bounds.height();
                node_candidates.push((node.id(), area, node.name().to_string()));
            }
        }
        // Sort by area (smallest first) so innermost nodes come first
        node_candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        for (node_id, area, name) in node_candidates {
            eprintln!("[CANDIDATES] Adding node '{}' (area: {:.0})", name, area);
            candidates.push(ClickCandidate::Node(node_id));
        }

        // 5. Check connection lines (lowest priority)
        for conn in state.diagram.connections() {
            if conn.is_near_point(pos, connection_tolerance) {
                // Only add if not already in candidates as endpoint or label
                let already_has = candidates.iter().any(|c| matches!(c,
                    ClickCandidate::Endpoint(id, _) | ClickCandidate::Label(id) if *id == conn.id));
                if !already_has {
                    candidates.push(ClickCandidate::Connection(conn.id));
                }
            }
        }

        candidates
    }

    /// Select a click candidate and set up appropriate state for dragging/selection
    /// Returns true if the candidate was successfully selected
    fn select_candidate(&mut self, candidate: ClickCandidate) -> bool {
        match candidate {
            ClickCandidate::Pivot(conn_id, idx) => {
                if let Some(state) = self.current_diagram_mut() {
                    if state.diagram.selected_connection() != Some(conn_id) {
                        state.diagram.select_connection(conn_id);
                    }
                    state.diagram.push_undo();
                }
                self.dragging_pivot = Some((conn_id, idx));
                self.selected_pivot = Some((conn_id, idx));
                self.dragging_nodes = false;
                self.dragging_label = None;
                self.dragging_endpoint = None;
                self.selection_rect.clear();
                self.status_message = format!("Pivot point {} (Delete to remove)", idx + 1);
                true
            }
            ClickCandidate::Endpoint(conn_id, is_source) => {
                if let Some(state) = self.current_diagram_mut() {
                    if state.diagram.selected_connection() != Some(conn_id) {
                        state.diagram.select_connection(conn_id);
                    }
                    state.diagram.push_undo();
                }
                self.dragging_endpoint = Some((conn_id, is_source));
                self.dragging_pivot = None;
                self.selected_pivot = None;
                self.dragging_nodes = false;
                self.dragging_label = None;
                self.selection_rect.clear();
                self.status_message = if is_source {
                    "Source endpoint".to_string()
                } else {
                    "Target endpoint".to_string()
                };
                true
            }
            ClickCandidate::Label(conn_id) => {
                if let Some(state) = self.current_diagram_mut() {
                    state.diagram.select_connection_label(conn_id);
                    state.diagram.push_undo();
                    // Mark label as no longer adjoined when user starts dragging
                    if let Some(conn) = state.diagram.find_connection_mut(conn_id) {
                        if conn.text_adjoined {
                            conn.text_adjoined = false;
                            if conn.label_offset.is_none() {
                                conn.label_offset = Some((0.0, -15.0));
                            }
                        }
                    }
                }
                self.dragging_label = Some(conn_id);
                self.dragging_nodes = false;
                self.dragging_pivot = None;
                self.dragging_endpoint = None;
                self.selected_pivot = None;
                self.selection_rect.clear();
                self.status_message = "Connection label".to_string();
                true
            }
            ClickCandidate::Node(node_id) => {
                if self.edit_mode != EditMode::Arrow {
                    self.set_edit_mode(EditMode::Arrow);
                }
                if let Some(state) = self.current_diagram_mut() {
                    let already_selected = state.diagram.selected_nodes().contains(&node_id);
                    if !already_selected {
                        state.diagram.clear_selection();
                        state.diagram.select_node(node_id);
                    }
                    state.diagram.push_undo();
                }
                self.dragging_nodes = true;
                self.dragging_label = None;
                self.dragging_pivot = None;
                self.dragging_endpoint = None;
                self.selected_pivot = None;
                self.selection_rect.clear();
                // Get node name for status
                let name = self.current_diagram()
                    .and_then(|s| s.diagram.find_node(node_id))
                    .map(|n| n.name().to_string())
                    .unwrap_or_else(|| "Node".to_string());
                self.status_message = name;
                true
            }
            ClickCandidate::Connection(conn_id) => {
                if let Some(state) = self.current_diagram_mut() {
                    state.diagram.select_connection(conn_id);
                }
                self.dragging_nodes = false;
                self.dragging_label = None;
                self.dragging_pivot = None;
                self.dragging_endpoint = None;
                self.selected_pivot = None;
                self.selection_rect.clear();
                self.status_message = "Connection".to_string();
                true
            }
        }
    }

    /// Handle click with disambiguation for overlapping items
    /// Returns true if a candidate was selected (or loupe was shown)
    fn handle_click_with_disambiguation(&mut self, pos: Point) -> bool {
        eprintln!("[CLICK] Position: ({:.1}, {:.1}), mode: {:?}", pos.x, pos.y, self.selection_mode);

        // If loupe is visible and we clicked outside it, hide it
        if self.loupe.visible {
            // Loupe click handling is done separately
            return false;
        }

        // Find all candidates at this position
        let candidates = self.find_all_candidates_at(pos);
        eprintln!("[CLICK] Found {} candidates at position:", candidates.len());
        for (i, c) in candidates.iter().enumerate() {
            eprintln!("[CLICK]   {}: {:?}", i + 1, c);
        }

        if candidates.is_empty() {
            eprintln!("[CLICK] No candidates found - clearing state");
            self.click_cycle_candidates.clear();
            self.click_cycle_index = 0;
            self.click_cycle_pos = None;
            self.loupe.hide();
            return false;
        }

        // If only one candidate, select it directly regardless of mode
        if candidates.len() == 1 {
            let candidate = candidates[0];
            eprintln!("[CLICK] Single candidate - selecting: {:?}", candidate);
            self.select_candidate(candidate);
            self.click_cycle_candidates.clear();
            self.click_cycle_pos = None;
            return true;
        }

        // Multiple candidates - use disambiguation mode
        let cycle_distance = self.settings.click_cycle_distance;
        match self.selection_mode {
            SelectionMode::Cycle => {
                // Check if this is a "same location" click for cycling
                let is_same_location = self.click_cycle_pos
                    .map(|prev| {
                        let dx = pos.x - prev.x;
                        let dy = pos.y - prev.y;
                        let dist = (dx * dx + dy * dy).sqrt();
                        eprintln!("[CLICK] Distance from last click: {:.1} (threshold: {})", dist, cycle_distance);
                        dist <= cycle_distance
                    })
                    .unwrap_or(false);

                if is_same_location && !self.click_cycle_candidates.is_empty() {
                    // Cycle to next candidate
                    self.click_cycle_index = (self.click_cycle_index + 1) % self.click_cycle_candidates.len();
                    let candidate = self.click_cycle_candidates[self.click_cycle_index];
                    let count = self.click_cycle_candidates.len();
                    let idx = self.click_cycle_index + 1;
                    eprintln!("[CLICK] Cycling to candidate {}/{}: {:?}", idx, count, candidate);
                    self.select_candidate(candidate);
                    self.status_message = format!("{} ({}/{})", self.status_message, idx, count);
                    return true;
                }

                // New location - store cycling state and select first
                self.click_cycle_candidates = candidates.clone();
                self.click_cycle_index = 0;
                self.click_cycle_pos = Some(pos);

                let candidate = candidates[0];
                eprintln!("[CLICK] Selecting first candidate: {:?}", candidate);
                self.select_candidate(candidate);
                self.status_message = format!("{} (1/{}) - click again to cycle", self.status_message, candidates.len());
                true
            }
            SelectionMode::Loupe => {
                // Show the magnifying loupe
                eprintln!("[CLICK] Showing loupe with {} candidates", candidates.len());
                self.loupe.show(pos, candidates);
                self.status_message = format!("Select from {} items in loupe", self.loupe.candidates.len());
                true
            }
        }
    }

    /// Handle click with cycling support for overlapping items (legacy name)
    fn handle_click_with_cycling(&mut self, pos: Point) -> bool {
        self.handle_click_with_disambiguation(pos)
    }

    /// Find the nearest side and offset on a node bounds for a given point
    fn find_nearest_side_and_offset(bounds: &Rect, pos: Point) -> (jmt_core::node::Side, f32) {
        use jmt_core::node::Side;

        let center_x = (bounds.x1 + bounds.x2) / 2.0;
        let center_y = (bounds.y1 + bounds.y2) / 2.0;

        // Distance to each side
        let dist_top = (pos.y - bounds.y1).abs();
        let dist_bottom = (pos.y - bounds.y2).abs();
        let dist_left = (pos.x - bounds.x1).abs();
        let dist_right = (pos.x - bounds.x2).abs();

        let min_dist = dist_top.min(dist_bottom).min(dist_left).min(dist_right);

        // Calculate max offset (ensure it's never negative)
        let max_h_offset = (bounds.width() / 2.0 - 10.0).max(0.0);
        let max_v_offset = (bounds.height() / 2.0 - 10.0).max(0.0);

        if min_dist == dist_top {
            let offset = (pos.x - center_x).clamp(-max_h_offset, max_h_offset);
            (Side::Top, offset)
        } else if min_dist == dist_bottom {
            let offset = (pos.x - center_x).clamp(-max_h_offset, max_h_offset);
            (Side::Bottom, offset)
        } else if min_dist == dist_left {
            let offset = (pos.y - center_y).clamp(-max_v_offset, max_v_offset);
            (Side::Left, offset)
        } else {
            let offset = (pos.y - center_y).clamp(-max_v_offset, max_v_offset);
            (Side::Right, offset)
        }
    }

    /// Create a new diagram
    pub fn new_diagram(&mut self) {
        self.new_diagram_of_type(DiagramType::StateMachine);
    }

    /// Create a new diagram of a specific type
    pub fn new_diagram_of_type(&mut self, diagram_type: DiagramType) {
        let type_name = diagram_type.display_name();
        let name = format!("{} {}", type_name, self.diagrams.len() + 1);
        let mut diagram = Diagram::new(&name);
        diagram.diagram_type = diagram_type;
        self.diagrams.push(DiagramState::new(diagram));
        self.active_diagram = self.diagrams.len() - 1;
        self.edit_mode = EditMode::Arrow;
        self.status_message = format!("Created new {}", type_name);
    }

    /// Close the current diagram
    pub fn close_diagram(&mut self) {
        if self.diagrams.len() > 1 {
            self.diagrams.remove(self.active_diagram);
            if self.active_diagram >= self.diagrams.len() {
                self.active_diagram = self.diagrams.len() - 1;
            }
        }
    }

    /// Open a sub-statemachine for the given state node
    pub fn open_substatemachine(&mut self, state_id: NodeId) {
        // Get the sub-statemachine info from the state
        let substatemachine_info = {
            let Some(diagram_state) = self.current_diagram() else { return; };
            let Some(node) = diagram_state.diagram.find_node(state_id) else { return; };
            let Some(state) = node.as_state() else { return; };

            match &state.substatemachine_path {
                Some(path) if !path.is_empty() => {
                    // External file
                    Some((path.clone(), state.name.clone(), state.title.clone()))
                }
                Some(_) => {
                    // Embedded - create view from regions
                    Some((String::new(), state.name.clone(), state.title.clone()))
                }
                None => None,
            }
        };

        if let Some((path, state_name, title)) = substatemachine_info {
            if path.is_empty() {
                // Embedded sub-statemachine - create a new diagram view
                self.open_embedded_substatemachine(state_id, &state_name, &title);
            } else {
                // External file - load it
                self.open_file_at_path(&path);
            }
        }
    }

    /// Create a new sub-statemachine for the given state
    pub fn create_substatemachine(&mut self, state_id: NodeId) {
        // Mark the state as having an embedded sub-statemachine
        if let Some(diagram_state) = self.current_diagram_mut() {
            if let Some(node) = diagram_state.diagram.find_node_mut(state_id) {
                if let Some(state) = node.as_state_mut() {
                    state.substatemachine_path = Some(String::new()); // Embedded by default
                    diagram_state.modified = true;
                    self.status_message = format!("Created sub-statemachine for '{}'", state.name);
                }
            }
        }
    }

    /// Open an embedded sub-statemachine as a new tab
    fn open_embedded_substatemachine(&mut self, state_id: NodeId, state_name: &str, title: &str) {
        // Extract child nodes and connections from the parent state
        let (nodes, connections) = if let Some(diagram_state) = self.current_diagram() {
            diagram_state.diagram.extract_substatemachine_contents(state_id)
        } else {
            (Vec::new(), Vec::new())
        };

        // Create a new diagram with the extracted contents
        let display_name = if title.is_empty() { state_name } else { title };
        let name = format!("{} (sub)", display_name);
        let mut diagram = Diagram::new(&name);
        diagram.title = display_name.to_string();
        diagram.diagram_type = DiagramType::StateMachine;

        // Import the extracted nodes and connections
        let node_count = nodes.len();
        diagram.import_nodes_and_connections(nodes, connections);

        self.diagrams.push(DiagramState::new(diagram));
        self.active_diagram = self.diagrams.len() - 1;
        self.edit_mode = EditMode::Arrow;

        if node_count > 0 {
            self.status_message = format!("Opened sub-statemachine: {} ({} nodes)", display_name, node_count);
        } else {
            self.status_message = format!("Opened sub-statemachine: {} (empty)", display_name);
        }
    }

    /// Open a diagram file at the given path
    #[cfg(not(target_arch = "wasm32"))]
    fn open_file_at_path(&mut self, path: &str) {
        use std::path::PathBuf;

        // Resolve relative path based on current diagram's location
        let resolved_path = if let Some(current_state) = self.current_diagram() {
            if let Some(current_path) = &current_state.file_path {
                if let Some(parent) = current_path.parent() {
                    parent.join(path)
                } else {
                    PathBuf::from(path)
                }
            } else {
                PathBuf::from(path)
            }
        } else {
            PathBuf::from(path)
        };

        match std::fs::read_to_string(&resolved_path) {
            Ok(content) => {
                match serde_json::from_str::<Diagram>(&content) {
                    Ok(mut diagram) => {
                        diagram.recalculate_connections();
                        let state = DiagramState::with_path(diagram, resolved_path.clone());
                        self.diagrams.push(state);
                        self.active_diagram = self.diagrams.len() - 1;
                        let filename = resolved_path.file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "file".to_string());
                        self.status_message = format!("Opened: {}", filename);
                    }
                    Err(e) => {
                        self.status_message = format!("Error parsing file: {}", e);
                    }
                }
            }
            Err(e) => {
                self.status_message = format!("Error reading file '{}': {}", path, e);
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn open_file_at_path(&mut self, path: &str) {
        self.status_message = format!("Cannot open external files in web mode: {}", path);
    }

    /// Check if a point is on a sub-statemachine icon and return the state ID
    fn check_substatemachine_icon_at(&self, point: Point) -> Option<NodeId> {
        let state = self.current_diagram()?;
        let zoom = self.zoom_level;

        for node in state.diagram.nodes() {
            if let Some(state_node) = node.as_state() {
                if state_node.has_substatemachine() && !state_node.show_expanded {
                    // Check if click is in icon area (bottom-right corner)
                    let bounds = node.bounds();
                    let icon_size = 16.0 / zoom; // Unscale for diagram coords
                    let margin = 4.0 / zoom;

                    let icon_left = bounds.x2 - icon_size - margin;
                    let icon_top = bounds.y2 - icon_size - margin;
                    let icon_right = bounds.x2 - margin;
                    let icon_bottom = bounds.y2 - margin;

                    if point.x >= icon_left && point.x <= icon_right
                        && point.y >= icon_top && point.y <= icon_bottom
                    {
                        return Some(node.id());
                    }
                }
            }
        }
        None
    }

    /// Render the sub-statemachine preview popup
    fn render_substatemachine_preview(&mut self, ctx: &egui::Context) {
        if let Some(state_id) = self.preview_substatemachine {
            let (state_name, title) = {
                let Some(diagram_state) = self.current_diagram() else {
                    self.preview_substatemachine = None;
                    return;
                };
                let Some(node) = diagram_state.diagram.find_node(state_id) else {
                    self.preview_substatemachine = None;
                    return;
                };
                let Some(state) = node.as_state() else {
                    self.preview_substatemachine = None;
                    return;
                };
                (state.name.clone(), state.title.clone())
            };

            let display_name = if title.is_empty() { &state_name } else { &title };

            let mut open = true;
            egui::Window::new(format!("Sub-Statemachine: {}", display_name))
                .collapsible(true)
                .resizable(true)
                .default_size([300.0, 200.0])
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label("Preview of sub-statemachine contents");
                    ui.separator();

                    // Placeholder for actual preview rendering
                    ui.label("(Diagram preview would be shown here)");

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Open in Tab").clicked() {
                            // Open the sub-statemachine in a new tab
                            self.open_substatemachine(state_id);
                            self.preview_substatemachine = None;
                        }
                    });
                });

            if !open {
                self.preview_substatemachine = None;
            }
        }
    }

    /// Save the current diagram (prompts for path if not yet saved)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&mut self) {
        if let Some(state) = self.current_diagram() {
            if state.file_path.is_some() {
                // Has a path, save directly
                self.save_to_current_path();
            } else {
                // No path yet, do Save As
                self.save_as();
            }
        }
    }

    /// Save the current diagram to its current path
    #[cfg(not(target_arch = "wasm32"))]
    fn save_to_current_path(&mut self) {
        if let Some(state) = self.current_diagram() {
            if let Some(path) = &state.file_path {
                let json = serde_json::to_string_pretty(&state.diagram);
                match json {
                    Ok(content) => {
                        match std::fs::write(path, &content) {
                            Ok(_) => {
                                let filename = path.file_name()
                                    .map(|s| s.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "file".to_string());
                                self.status_message = format!("Saved: {}", filename);
                                // Mark as not modified after successful save
                                if let Some(state) = self.current_diagram_mut() {
                                    state.modified = false;
                                }
                            }
                            Err(e) => {
                                self.status_message = format!("Error saving: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        self.status_message = format!("Error serializing: {}", e);
                    }
                }
            }
        }
    }

    /// Save As - always prompts for a new file path
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_as(&mut self) {
        use rfd::FileDialog;

        if self.current_diagram().is_none() {
            return;
        }

        // Get diagram name for default filename
        let default_name = self.current_diagram()
            .map(|s| s.diagram.settings.name.clone())
            .unwrap_or_else(|| "diagram".to_string());

        let dialog = FileDialog::new()
            .set_title("Save Diagram As")
            .add_filter("JMT Diagram", &["jmt"])
            .add_filter("JSON", &["json"])
            .set_file_name(&format!("{}.jmt", default_name));

        if let Some(path) = dialog.save_file() {
            // Update the file path in both places
            if let Some(state) = self.current_diagram_mut() {
                state.file_path = Some(path.clone());
                // Also update the diagram settings so it gets serialized
                state.diagram.settings.file_path = Some(path.to_string_lossy().to_string());
            }
            // Now save to this path
            self.save_to_current_path();
        }
    }

    /// Open a diagram from file
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open(&mut self) {
        use rfd::FileDialog;

        let dialog = FileDialog::new()
            .set_title("Open Diagram")
            .add_filter("JMT Diagram", &["jmt"])
            .add_filter("JSON", &["json"])
            .add_filter("All Files", &["*"]);

        if let Some(path) = dialog.pick_file() {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str::<Diagram>(&content) {
                        Ok(mut diagram) => {
                            // Recalculate connection routing (segments are not serialized)
                            diagram.recalculate_connections();
                            let state = DiagramState::with_path(diagram, path.clone());
                            self.diagrams.push(state);
                            self.active_diagram = self.diagrams.len() - 1;
                            let filename = path.file_name()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_else(|| "file".to_string());
                            self.status_message = format!("Opened: {}", filename);
                        }
                        Err(e) => {
                            self.status_message = format!("Error parsing file: {}", e);
                        }
                    }
                }
                Err(e) => {
                    self.status_message = format!("Error reading file: {}", e);
                }
            }
        }
    }

    /// Export the current diagram as PNG
    #[cfg(not(target_arch = "wasm32"))]
    pub fn export_png(&mut self, autocrop: bool) {
        use rfd::FileDialog;
        use image::{ImageBuffer, Rgba};

        let state = match self.current_diagram() {
            Some(s) => s,
            None => return,
        };

        // Get diagram name for default filename
        let default_name = state.diagram.settings.name.clone();

        let dialog = FileDialog::new()
            .set_title("Export as PNG")
            .add_filter("PNG Image", &["png"])
            .set_file_name(&format!("{}.png", default_name));

        if let Some(path) = dialog.save_file() {
            // Calculate diagram bounds
            let (min_x, min_y, max_x, max_y) = self.calculate_diagram_bounds();

            if max_x <= min_x || max_y <= min_y {
                self.status_message = "No elements to export".to_string();
                return;
            }

            // Add margin
            let margin = if autocrop { 20.0 } else { 50.0 };
            let (x_offset, y_offset, width, height) = if autocrop {
                (min_x - margin, min_y - margin,
                 (max_x - min_x + margin * 2.0) as u32,
                 (max_y - min_y + margin * 2.0) as u32)
            } else {
                // Use fixed canvas size with content positioned
                let canvas_width = (max_x + margin * 2.0).max(800.0) as u32;
                let canvas_height = (max_y + margin * 2.0).max(600.0) as u32;
                (0.0, 0.0, canvas_width, canvas_height)
            };

            // Create image buffer with white background
            let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_pixel(
                width, height,
                Rgba([255, 255, 255, 255])
            );

            // Render diagram elements to the image
            if let Some(state) = self.current_diagram() {
                Self::render_diagram_to_image(&mut img, &state.diagram, x_offset, y_offset);
            }

            // Save the image
            match img.save(&path) {
                Ok(_) => {
                    let filename = path.file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "file".to_string());
                    self.status_message = format!("Exported: {}", filename);
                }
                Err(e) => {
                    self.status_message = format!("Error exporting: {}", e);
                }
            }
        }
    }

    /// Calculate the bounding box of all diagram elements
    #[cfg(not(target_arch = "wasm32"))]
    fn calculate_diagram_bounds(&self) -> (f32, f32, f32, f32) {
        let state = match self.current_diagram() {
            Some(s) => s,
            None => return (0.0, 0.0, 0.0, 0.0),
        };

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        // State machine nodes
        for node in state.diagram.nodes() {
            let bounds = node.bounds();
            min_x = min_x.min(bounds.x1);
            min_y = min_y.min(bounds.y1);
            max_x = max_x.max(bounds.x2);
            max_y = max_y.max(bounds.y2);
        }

        // Connection endpoints
        for conn in state.diagram.connections() {
            for seg in &conn.segments {
                min_x = min_x.min(seg.start.x).min(seg.end.x);
                min_y = min_y.min(seg.start.y).min(seg.end.y);
                max_x = max_x.max(seg.start.x).max(seg.end.x);
                max_y = max_y.max(seg.start.y).max(seg.end.y);
            }
        }

        (min_x, min_y, max_x, max_y)
    }

    /// Render diagram to an image buffer
    #[cfg(not(target_arch = "wasm32"))]
    fn render_diagram_to_image(
        img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
        diagram: &Diagram,
        x_offset: f32,
        y_offset: f32,
    ) {
        use image::Rgba;

        let state_fill = Rgba([255, 255, 204, 255]); // Light yellow
        let state_stroke = Rgba([0, 0, 0, 255]); // Black
        let pseudo_fill = Rgba([0, 0, 0, 255]); // Black for initial/final
        let connection_color = Rgba([100, 100, 100, 255]); // Gray

        // Draw states
        for node in diagram.nodes() {
            let bounds = node.bounds();
            let x1 = (bounds.x1 - x_offset) as i32;
            let y1 = (bounds.y1 - y_offset) as i32;
            let x2 = (bounds.x2 - x_offset) as i32;
            let y2 = (bounds.y2 - y_offset) as i32;

            match node {
                jmt_core::node::Node::State(state) => {
                    // Draw filled rounded rectangle with proper corners
                    let corner_radius = 12; // Match the diagram settings
                    Self::draw_filled_rounded_rect(img, x1, y1, x2, y2, corner_radius, state_fill);
                    Self::draw_rounded_rect_outline(img, x1, y1, x2, y2, corner_radius, state_stroke);

                    // Determine if activities should be shown
                    let show_activities = state.should_show_activities(diagram.settings.show_activities);

                    // Draw state name and activities
                    let name = node.name();
                    let cx = (x1 + x2) / 2;

                    if show_activities {
                        // Draw name near top
                        let name_y = y1 + 15;
                        Self::draw_text_centered(img, cx, name_y, name, state_stroke);

                        // Draw separator line below name
                        let sep_y = y1 + 22;
                        Self::draw_line(img, x1 + 5, sep_y, x2 - 5, sep_y, state_stroke);

                        // Draw activities below separator
                        let mut activity_y = sep_y + 12;
                        let activity_x = x1 + 8;

                        if !state.entry_activity.is_empty() {
                            let text = format!("entry / {}", state.entry_activity);
                            Self::draw_text_left(img, activity_x, activity_y, &text, state_stroke);
                            activity_y += 12;
                        }
                        if !state.exit_activity.is_empty() {
                            let text = format!("exit / {}", state.exit_activity);
                            Self::draw_text_left(img, activity_x, activity_y, &text, state_stroke);
                            activity_y += 12;
                        }
                        if !state.do_activity.is_empty() {
                            let text = format!("do / {}", state.do_activity);
                            Self::draw_text_left(img, activity_x, activity_y, &text, state_stroke);
                        }
                    } else {
                        // Just center the name
                        let cy = (y1 + y2) / 2;
                        Self::draw_text_centered(img, cx, cy, name, state_stroke);
                    }
                }
                jmt_core::node::Node::Pseudo(ps) => {
                    let cx = ((bounds.x1 + bounds.x2) / 2.0 - x_offset) as i32;
                    let cy = ((bounds.y1 + bounds.y2) / 2.0 - y_offset) as i32;
                    let radius = ((bounds.x2 - bounds.x1) / 2.0) as i32;

                    match ps.kind {
                        jmt_core::node::PseudoStateKind::Initial => {
                            Self::draw_filled_circle(img, cx, cy, radius, pseudo_fill);
                        }
                        jmt_core::node::PseudoStateKind::Final => {
                            Self::draw_circle_outline(img, cx, cy, radius, pseudo_fill);
                            Self::draw_filled_circle(img, cx, cy, radius - 4, pseudo_fill);
                        }
                        jmt_core::node::PseudoStateKind::Choice | jmt_core::node::PseudoStateKind::Junction => {
                            // Diamond shape
                            Self::draw_diamond(img, cx, cy, radius, state_stroke);
                        }
                        jmt_core::node::PseudoStateKind::Fork | jmt_core::node::PseudoStateKind::Join => {
                            // Thick bar
                            Self::draw_filled_rect(img, x1, y1, x2, y2, pseudo_fill);
                        }
                    }
                }
            }
        }

        // Draw connections
        for conn in diagram.connections() {
            for seg in &conn.segments {
                let x1 = (seg.start.x - x_offset) as i32;
                let y1 = (seg.start.y - y_offset) as i32;
                let x2 = (seg.end.x - x_offset) as i32;
                let y2 = (seg.end.y - y_offset) as i32;
                Self::draw_line(img, x1, y1, x2, y2, connection_color);
            }

            // Draw arrowhead at target
            if let Some(last_seg) = conn.segments.last() {
                let x2 = (last_seg.end.x - x_offset) as i32;
                let y2 = (last_seg.end.y - y_offset) as i32;
                let x1 = (last_seg.start.x - x_offset) as i32;
                let y1 = (last_seg.start.y - y_offset) as i32;
                Self::draw_arrowhead(img, x1, y1, x2, y2, connection_color);
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn draw_filled_rect(img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, x1: i32, y1: i32, x2: i32, y2: i32, color: image::Rgba<u8>) {
        let (width, height) = img.dimensions();
        for y in y1.max(0)..y2.min(height as i32) {
            for x in x1.max(0)..x2.min(width as i32) {
                img.put_pixel(x as u32, y as u32, color);
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn draw_rect_outline(img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, x1: i32, y1: i32, x2: i32, y2: i32, color: image::Rgba<u8>) {
        Self::draw_line(img, x1, y1, x2, y1, color); // Top
        Self::draw_line(img, x1, y2, x2, y2, color); // Bottom
        Self::draw_line(img, x1, y1, x1, y2, color); // Left
        Self::draw_line(img, x2, y1, x2, y2, color); // Right
    }

    /// Draw a filled rounded rectangle
    #[cfg(not(target_arch = "wasm32"))]
    fn draw_filled_rounded_rect(img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, x1: i32, y1: i32, x2: i32, y2: i32, radius: i32, color: image::Rgba<u8>) {
        let (width, height) = img.dimensions();
        let r = radius.min((x2 - x1) / 2).min((y2 - y1) / 2);

        for y in y1.max(0)..y2.min(height as i32) {
            for x in x1.max(0)..x2.min(width as i32) {
                // Check if point is inside rounded rect
                let in_rect = if x < x1 + r && y < y1 + r {
                    // Top-left corner
                    let dx = x - (x1 + r);
                    let dy = y - (y1 + r);
                    dx * dx + dy * dy <= r * r
                } else if x > x2 - r && y < y1 + r {
                    // Top-right corner
                    let dx = x - (x2 - r);
                    let dy = y - (y1 + r);
                    dx * dx + dy * dy <= r * r
                } else if x < x1 + r && y > y2 - r {
                    // Bottom-left corner
                    let dx = x - (x1 + r);
                    let dy = y - (y2 - r);
                    dx * dx + dy * dy <= r * r
                } else if x > x2 - r && y > y2 - r {
                    // Bottom-right corner
                    let dx = x - (x2 - r);
                    let dy = y - (y2 - r);
                    dx * dx + dy * dy <= r * r
                } else {
                    true
                };

                if in_rect {
                    img.put_pixel(x as u32, y as u32, color);
                }
            }
        }
    }

    /// Draw a rounded rectangle outline
    #[cfg(not(target_arch = "wasm32"))]
    fn draw_rounded_rect_outline(img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, x1: i32, y1: i32, x2: i32, y2: i32, radius: i32, color: image::Rgba<u8>) {
        let r = radius.min((x2 - x1) / 2).min((y2 - y1) / 2);

        // Draw straight edges (excluding corners)
        Self::draw_line(img, x1 + r, y1, x2 - r, y1, color); // Top
        Self::draw_line(img, x1 + r, y2, x2 - r, y2, color); // Bottom
        Self::draw_line(img, x1, y1 + r, x1, y2 - r, color); // Left
        Self::draw_line(img, x2, y1 + r, x2, y2 - r, color); // Right

        // Draw corner arcs using circle algorithm
        Self::draw_corner_arc(img, x1 + r, y1 + r, r, 2, color); // Top-left
        Self::draw_corner_arc(img, x2 - r, y1 + r, r, 1, color); // Top-right
        Self::draw_corner_arc(img, x1 + r, y2 - r, r, 3, color); // Bottom-left
        Self::draw_corner_arc(img, x2 - r, y2 - r, r, 0, color); // Bottom-right
    }

    /// Draw a quarter circle arc (quadrant: 0=BR, 1=TR, 2=TL, 3=BL)
    #[cfg(not(target_arch = "wasm32"))]
    fn draw_corner_arc(img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, cx: i32, cy: i32, r: i32, quadrant: u8, color: image::Rgba<u8>) {
        let (width, height) = img.dimensions();
        let mut x = 0;
        let mut y = r;
        let mut d = 3 - 2 * r;

        while x <= y {
            let points = match quadrant {
                0 => [(cx + x, cy + y), (cx + y, cy + x)], // Bottom-right
                1 => [(cx + x, cy - y), (cx + y, cy - x)], // Top-right
                2 => [(cx - x, cy - y), (cx - y, cy - x)], // Top-left
                3 => [(cx - x, cy + y), (cx - y, cy + x)], // Bottom-left
                _ => [(cx, cy), (cx, cy)],
            };

            for (px, py) in points {
                if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                    img.put_pixel(px as u32, py as u32, color);
                }
            }

            if d < 0 {
                d += 4 * x + 6;
            } else {
                d += 4 * (x - y) + 10;
                y -= 1;
            }
            x += 1;
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn draw_line(img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, x1: i32, y1: i32, x2: i32, y2: i32, color: image::Rgba<u8>) {
        let (width, height) = img.dimensions();
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;
        let mut x = x1;
        let mut y = y1;

        loop {
            if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                img.put_pixel(x as u32, y as u32, color);
            }
            if x == x2 && y == y2 { break; }
            let e2 = 2 * err;
            if e2 > -dy { err -= dy; x += sx; }
            if e2 < dx { err += dx; y += sy; }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn draw_filled_circle(img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, cx: i32, cy: i32, radius: i32, color: image::Rgba<u8>) {
        let (width, height) = img.dimensions();
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
                    let x = cx + dx;
                    let y = cy + dy;
                    if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                        img.put_pixel(x as u32, y as u32, color);
                    }
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn draw_circle_outline(img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, cx: i32, cy: i32, radius: i32, color: image::Rgba<u8>) {
        let (width, height) = img.dimensions();
        let r2_outer = radius * radius;
        let r2_inner = (radius - 2) * (radius - 2);
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let d2 = dx * dx + dy * dy;
                if d2 <= r2_outer && d2 >= r2_inner {
                    let x = cx + dx;
                    let y = cy + dy;
                    if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                        img.put_pixel(x as u32, y as u32, color);
                    }
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn draw_diamond(img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, cx: i32, cy: i32, radius: i32, color: image::Rgba<u8>) {
        // Draw diamond outline
        Self::draw_line(img, cx, cy - radius, cx + radius, cy, color); // Top to right
        Self::draw_line(img, cx + radius, cy, cx, cy + radius, color); // Right to bottom
        Self::draw_line(img, cx, cy + radius, cx - radius, cy, color); // Bottom to left
        Self::draw_line(img, cx - radius, cy, cx, cy - radius, color); // Left to top
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn draw_arrowhead(img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, x1: i32, y1: i32, x2: i32, y2: i32, color: image::Rgba<u8>) {
        let dx = (x2 - x1) as f32;
        let dy = (y2 - y1) as f32;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1.0 { return; }

        let ux = dx / len;
        let uy = dy / len;

        let arrow_len = 10.0;
        let arrow_width = 5.0;

        let ax = x2 as f32 - ux * arrow_len;
        let ay = y2 as f32 - uy * arrow_len;

        let p1x = (ax - uy * arrow_width) as i32;
        let p1y = (ay + ux * arrow_width) as i32;
        let p2x = (ax + uy * arrow_width) as i32;
        let p2y = (ay - ux * arrow_width) as i32;

        Self::draw_line(img, x2, y2, p1x, p1y, color);
        Self::draw_line(img, x2, y2, p2x, p2y, color);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn draw_text_centered(img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, cx: i32, cy: i32, text: &str, color: image::Rgba<u8>) {
        // Simple text rendering - draw each character as a small pattern
        // This is a simplified version; for production, use a proper font rendering library
        let char_width = 6;
        let char_height = 10;
        let text_width = text.len() as i32 * char_width;
        let start_x = cx - text_width / 2;
        let start_y = cy - char_height / 2;

        for (i, c) in text.chars().enumerate() {
            let x = start_x + i as i32 * char_width;
            Self::draw_simple_char(img, x, start_y, c, color);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn draw_text_left(img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, x: i32, y: i32, text: &str, color: image::Rgba<u8>) {
        // Left-aligned text rendering
        let char_width = 6;

        for (i, c) in text.chars().enumerate() {
            let char_x = x + i as i32 * char_width;
            Self::draw_simple_char(img, char_x, y, c, color);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn draw_simple_char(img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, x: i32, y: i32, c: char, color: image::Rgba<u8>) {
        // Very simple bitmap font for basic characters
        let (width, height) = img.dimensions();
        let patterns: &[(char, &[u8])] = &[
            ('A', &[0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001]),
            ('B', &[0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110]),
            ('C', &[0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110]),
            ('D', &[0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110]),
            ('E', &[0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111]),
            ('F', &[0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000]),
            ('G', &[0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110]),
            ('H', &[0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001]),
            ('I', &[0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110]),
            ('J', &[0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100]),
            ('K', &[0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001]),
            ('L', &[0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111]),
            ('M', &[0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001]),
            ('N', &[0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001]),
            ('O', &[0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110]),
            ('P', &[0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000]),
            ('Q', &[0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101]),
            ('R', &[0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001]),
            ('S', &[0b01110, 0b10001, 0b10000, 0b01110, 0b00001, 0b10001, 0b01110]),
            ('T', &[0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100]),
            ('U', &[0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110]),
            ('V', &[0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100]),
            ('W', &[0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001]),
            ('X', &[0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001]),
            ('Y', &[0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100]),
            ('Z', &[0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111]),
            ('0', &[0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110]),
            ('1', &[0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110]),
            ('2', &[0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111]),
            ('3', &[0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110]),
            ('4', &[0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010]),
            ('5', &[0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110]),
            ('6', &[0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110]),
            ('7', &[0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000]),
            ('8', &[0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110]),
            ('9', &[0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100]),
            ('a', &[0b00000, 0b00000, 0b01110, 0b00001, 0b01111, 0b10001, 0b01111]),
            ('b', &[0b10000, 0b10000, 0b10110, 0b11001, 0b10001, 0b10001, 0b11110]),
            ('c', &[0b00000, 0b00000, 0b01110, 0b10000, 0b10000, 0b10001, 0b01110]),
            ('d', &[0b00001, 0b00001, 0b01101, 0b10011, 0b10001, 0b10001, 0b01111]),
            ('e', &[0b00000, 0b00000, 0b01110, 0b10001, 0b11111, 0b10000, 0b01110]),
            ('f', &[0b00110, 0b01001, 0b01000, 0b11100, 0b01000, 0b01000, 0b01000]),
            ('g', &[0b00000, 0b01111, 0b10001, 0b10001, 0b01111, 0b00001, 0b01110]),
            ('h', &[0b10000, 0b10000, 0b10110, 0b11001, 0b10001, 0b10001, 0b10001]),
            ('i', &[0b00100, 0b00000, 0b01100, 0b00100, 0b00100, 0b00100, 0b01110]),
            ('j', &[0b00010, 0b00000, 0b00110, 0b00010, 0b00010, 0b10010, 0b01100]),
            ('k', &[0b10000, 0b10000, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010]),
            ('l', &[0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110]),
            ('m', &[0b00000, 0b00000, 0b11010, 0b10101, 0b10101, 0b10001, 0b10001]),
            ('n', &[0b00000, 0b00000, 0b10110, 0b11001, 0b10001, 0b10001, 0b10001]),
            ('o', &[0b00000, 0b00000, 0b01110, 0b10001, 0b10001, 0b10001, 0b01110]),
            ('p', &[0b00000, 0b00000, 0b11110, 0b10001, 0b11110, 0b10000, 0b10000]),
            ('q', &[0b00000, 0b00000, 0b01101, 0b10011, 0b01111, 0b00001, 0b00001]),
            ('r', &[0b00000, 0b00000, 0b10110, 0b11001, 0b10000, 0b10000, 0b10000]),
            ('s', &[0b00000, 0b00000, 0b01110, 0b10000, 0b01110, 0b00001, 0b11110]),
            ('t', &[0b01000, 0b01000, 0b11100, 0b01000, 0b01000, 0b01001, 0b00110]),
            ('u', &[0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b10011, 0b01101]),
            ('v', &[0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100]),
            ('w', &[0b00000, 0b00000, 0b10001, 0b10001, 0b10101, 0b10101, 0b01010]),
            ('x', &[0b00000, 0b00000, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001]),
            ('y', &[0b00000, 0b00000, 0b10001, 0b10001, 0b01111, 0b00001, 0b01110]),
            ('z', &[0b00000, 0b00000, 0b11111, 0b00010, 0b00100, 0b01000, 0b11111]),
            (' ', &[0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000]),
            ('_', &[0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111]),
            ('/', &[0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000]),
            ('-', &[0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000]),
            ('.', &[0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100]),
            (':', &[0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b01100, 0b00000]),
        ];

        // Search for exact character first, then fall back to uppercase
        let pattern = patterns.iter().find(|(ch, _)| *ch == c)
            .or_else(|| patterns.iter().find(|(ch, _)| *ch == c.to_ascii_uppercase()));
        if let Some((_, bits)) = pattern {
            for (row, &byte) in bits.iter().enumerate() {
                for col in 0..5 {
                    if (byte >> (4 - col)) & 1 == 1 {
                        let px = x + col;
                        let py = y + row as i32;
                        if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                            img.put_pixel(px as u32, py as u32, color);
                        }
                    }
                }
            }
        }
    }

    /// Set the edit mode
    pub fn set_edit_mode(&mut self, mode: EditMode) {
        // Special handling: If switching to Connect mode with multiple nodes selected,
        // connect them in sequence automatically
        if mode == EditMode::Connect {
            if let Some(state) = self.current_diagram_mut() {
                let selected = state.diagram.selected_nodes_in_order();
                if selected.len() >= 2 {
                    // Determine ordering: explicit (Ctrl+Click) or position-based (marquee)
                    let use_selection_order = state.diagram.has_explicit_selection_order();

                    // Get nodes with positions for sorting
                    let mut nodes_with_pos: Vec<_> = selected.iter()
                        .filter_map(|id| {
                            state.diagram.find_node(*id).map(|n| {
                                let center = n.bounds().center();
                                (*id, center.x)
                            })
                        })
                        .collect();

                    // Sort by x position if marquee selection (not explicit order)
                    if !use_selection_order {
                        nodes_with_pos.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                    }

                    let ordered_ids: Vec<_> = nodes_with_pos.iter().map(|(id, _)| *id).collect();

                    if ordered_ids.len() >= 2 {
                        // Connect nodes in sequence: 1->2, 2->3, 3->4, etc.
                        state.diagram.push_undo();
                        let mut connections_made = 0;
                        for i in 0..ordered_ids.len() - 1 {
                            let source = ordered_ids[i];
                            let target = ordered_ids[i + 1];
                            if state.diagram.add_connection(source, target).is_some() {
                                connections_made += 1;
                            }
                        }
                        if connections_made > 0 {
                            state.modified = true;
                            self.status_message = format!("Created {} connection(s)", connections_made);
                            // Stay in Arrow mode after auto-connecting
                            self.edit_mode = EditMode::Arrow;
                            self.pending_connection_source = None;
                            return;
                        }
                    }
                }
            }
        }

        self.edit_mode = mode;
        self.pending_connection_source = None;
        self.status_message = format!("Mode: {}", mode.display_name());
    }

    /// Zoom in by one step
    pub fn zoom_in(&mut self) {
        self.zoom_level = (self.zoom_level + ZOOM_STEP).min(MAX_ZOOM);
        self.status_message = format!("Zoom: {:.0}%", self.zoom_level * 100.0);
    }

    /// Zoom out by one step
    pub fn zoom_out(&mut self) {
        self.zoom_level = (self.zoom_level - ZOOM_STEP).max(MIN_ZOOM);
        self.status_message = format!("Zoom: {:.0}%", self.zoom_level * 100.0);
    }

    /// Reset zoom to 100%
    pub fn reset_zoom(&mut self) {
        self.zoom_level = 1.0;
        self.status_message = "Zoom: 100%".to_string();
    }

    /// Zoom by a delta (positive = zoom in, negative = zoom out)
    pub fn zoom_by(&mut self, delta: f32) {
        self.zoom_level = (self.zoom_level + delta).clamp(MIN_ZOOM, MAX_ZOOM);
        self.status_message = format!("Zoom: {:.0}%", self.zoom_level * 100.0);
    }

    /// Handle canvas click
    /// If `switch_to_arrow` is true, switch back to Arrow mode after adding element
    /// If `ctrl_held` is true, toggle selection instead of replacing it
    fn handle_canvas_click(&mut self, pos: egui::Pos2, switch_to_arrow: bool, ctrl_held: bool) {
        let point = Point::new(pos.x, pos.y);
        let edit_mode = self.edit_mode;
        let pending_source = self.pending_connection_source;
        let connection_tolerance = self.settings.connection_hit_tolerance;

        let Some(state) = self.current_diagram_mut() else {
            return;
        };

        match edit_mode {
            EditMode::Arrow => {
                // First check for connection labels (highest priority for selection)
                if let Some(conn_id) = state.diagram.find_connection_label_at(point) {
                    state.diagram.select_connection_label(conn_id);
                    self.status_message = "Selected connection label (drag to move)".to_string();
                    return;
                }

                // Check connections before nodes - connections are thin lines so if user
                // clicks on one, that's what they intended (even if inside a state)
                if let Some(conn_id) = state.diagram.find_connection_at(point, connection_tolerance) {
                    state.diagram.select_connection(conn_id);
                    self.status_message = "Selected connection".to_string();
                } else if let Some(element_id) = state.diagram.find_element_at(point) {
                    // Try to select any element (node, lifeline, actor, use case, action, etc.)
                    let name = state.diagram.get_element_name(element_id)
                        .unwrap_or_default();

                    // Check if this node has a placement error
                    let has_error = state.diagram.find_node(element_id)
                        .map(|n| n.has_error())
                        .unwrap_or(false);

                    if ctrl_held {
                        // Ctrl+Click: Toggle selection
                        state.diagram.toggle_element_selection(element_id);
                        let selected_count = state.diagram.selected_nodes().len();
                        self.status_message = format!("Selected {} element(s)", selected_count);
                    } else {
                        // Regular click: Select only this element
                        state.diagram.select_element(element_id);
                        if has_error {
                            let overlaps = state.diagram.find_overlapping_nodes(element_id);
                            if overlaps.is_empty() {
                                self.status_message = format!("Selected: {} ⚠ PLACEMENT ERROR: Unknown overlap", name);
                            } else {
                                let overlap_names: Vec<_> = overlaps.iter()
                                    .map(|(n, d)| format!("{} ({})", n, d))
                                    .collect();
                                self.status_message = format!("Selected: {} ⚠ PLACEMENT ERROR: Overlapping: {}", name, overlap_names.join(", "));
                            }
                        } else {
                            self.status_message = format!("Selected: {}", name);
                        }
                    }
                } else {
                    // Only clear selection if Ctrl is not held
                    if !ctrl_held {
                        state.diagram.clear_selection();
                    }
                    self.status_message = "Ready".to_string();
                }
            }
            EditMode::AddState => {
                state.diagram.push_undo();
                let id = state.diagram.add_node(NodeType::State, pos.x, pos.y);
                state.diagram.select_node(id);
                state.modified = true;
                self.status_message = "Added state".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddInitial => {
                state.diagram.push_undo();
                let id = state.diagram.add_node(NodeType::Initial, pos.x, pos.y);
                state.diagram.select_node(id);
                state.modified = true;
                self.status_message = "Added initial pseudo-state".to_string();
                // Auto-switch back to Arrow mode (typically only one initial state)
                self.edit_mode = EditMode::Arrow;
            }
            EditMode::AddFinal => {
                state.diagram.push_undo();
                let id = state.diagram.add_node(NodeType::Final, pos.x, pos.y);
                state.diagram.select_node(id);
                state.modified = true;
                self.status_message = "Added final pseudo-state".to_string();
                // Auto-switch back to Arrow mode (typically only one final state)
                self.edit_mode = EditMode::Arrow;
            }
            EditMode::AddChoice => {
                state.diagram.push_undo();
                let id = state.diagram.add_node(NodeType::Choice, pos.x, pos.y);
                state.diagram.select_node(id);
                state.modified = true;
                self.status_message = "Added choice pseudo-state".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddJunction => {
                state.diagram.push_undo();
                let id = state.diagram.add_node(NodeType::Junction, pos.x, pos.y);
                state.diagram.select_node(id);
                state.modified = true;
                self.status_message = "Added junction pseudo-state".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddFork => {
                state.diagram.push_undo();
                let id = state.diagram.add_node(NodeType::Fork, pos.x, pos.y);
                state.diagram.select_node(id);
                state.modified = true;
                self.status_message = "Added fork pseudo-state".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddJoin => {
                state.diagram.push_undo();
                let id = state.diagram.add_node(NodeType::Join, pos.x, pos.y);
                state.diagram.select_node(id);
                state.modified = true;
                self.status_message = "Added join pseudo-state".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::Connect => {
                if let Some(node_id) = state.diagram.find_node_at(point) {
                    if let Some(source_id) = pending_source {
                        // Complete the connection
                        state.diagram.push_undo();
                        if let Some(conn_id) = state.diagram.add_connection(source_id, node_id) {
                            state.diagram.select_connection(conn_id);
                            state.modified = true;
                            self.status_message = "Connection created".to_string();
                        } else {
                            self.status_message = "Cannot connect these nodes".to_string();
                        }
                        self.pending_connection_source = None;
                    } else {
                        // Start the connection
                        self.pending_connection_source = Some(node_id);
                        self.status_message = "Click target node to complete connection".to_string();
                    }
                } else {
                    // Clicked outside any node - switch back to Arrow mode
                    self.pending_connection_source = None;
                    self.edit_mode = EditMode::Arrow;
                    self.status_message = "Ready".to_string();
                }
            }

            // === Sequence Diagram Elements ===
            EditMode::AddLifeline => {
                state.diagram.push_undo();
                let _id = state.diagram.add_lifeline("Object", pos.x, pos.y);
                state.modified = true;
                self.status_message = "Added lifeline".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddActivation => {
                // TODO: Activations need to be attached to lifelines
                self.status_message = "Click on a lifeline to add activation".to_string();
            }
            EditMode::AddFragment => {
                state.diagram.push_undo();
                state.diagram.add_combined_fragment(pos.x, pos.y, 200.0, 150.0);
                state.modified = true;
                self.status_message = "Added combined fragment".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddSyncMessage | EditMode::AddAsyncMessage |
            EditMode::AddReturnMessage | EditMode::AddSelfMessage | EditMode::AddMessage => {
                // TODO: Messages need source and target lifelines
                self.status_message = "Click on source lifeline, then target lifeline".to_string();
            }

            // === Use Case Diagram Elements ===
            EditMode::AddActor => {
                state.diagram.push_undo();
                let _id = state.diagram.add_actor("Actor", pos.x, pos.y);
                state.modified = true;
                self.status_message = "Added actor".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddUseCase => {
                state.diagram.push_undo();
                let _id = state.diagram.add_use_case("Use Case", pos.x, pos.y);
                state.modified = true;
                self.status_message = "Added use case".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddSystemBoundary => {
                state.diagram.push_undo();
                let _id = state.diagram.add_system_boundary("System", pos.x, pos.y, 300.0, 400.0);
                state.modified = true;
                self.status_message = "Added system boundary".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddAssociation | EditMode::AddInclude |
            EditMode::AddExtend | EditMode::AddGeneralization => {
                // TODO: Relationships need source and target elements
                self.status_message = "Click on source element, then target element".to_string();
            }

            // === Activity Diagram Elements ===
            EditMode::AddAction => {
                state.diagram.push_undo();
                let _id = state.diagram.add_action("Action", pos.x, pos.y);
                state.modified = true;
                self.status_message = "Added action".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddDecision => {
                state.diagram.push_undo();
                state.diagram.add_decision_node(pos.x, pos.y);
                state.modified = true;
                self.status_message = "Added decision node".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddSendSignal => {
                state.diagram.push_undo();
                state.diagram.add_send_signal("Send", pos.x, pos.y);
                state.modified = true;
                self.status_message = "Added send signal".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddAcceptEvent => {
                state.diagram.push_undo();
                state.diagram.add_accept_event("Accept", pos.x, pos.y);
                state.modified = true;
                self.status_message = "Added accept event".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddTimeEvent => {
                state.diagram.push_undo();
                state.diagram.add_time_event("Time", pos.x, pos.y);
                state.modified = true;
                self.status_message = "Added time event".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddSwimlane => {
                state.diagram.push_undo();
                let _id = state.diagram.add_swimlane("Lane", pos.x, pos.y, 200.0, 400.0);
                state.modified = true;
                self.status_message = "Added swimlane".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddObjectNode => {
                state.diagram.push_undo();
                state.diagram.add_object_node("Object", pos.x, pos.y);
                state.modified = true;
                self.status_message = "Added object node".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }
            EditMode::AddDataStore => {
                state.diagram.push_undo();
                state.diagram.add_data_store("DataStore", pos.x, pos.y);
                state.modified = true;
                self.status_message = "Added data store".to_string();
                if switch_to_arrow {
                    self.edit_mode = EditMode::Arrow;
                }
            }

            _ => {}
        }
    }

    /// Render cursor preview for add modes
    fn render_cursor_preview(&self, painter: &egui::Painter, pos: egui::Pos2, zoom: f32) {
        let preview_alpha = 128u8; // Semi-transparent

        match self.edit_mode {
            EditMode::AddState => {
                // Draw a ghost state rectangle
                let width = self.current_diagram()
                    .map(|s| s.diagram.settings.default_state_width)
                    .unwrap_or(100.0) * zoom;
                let height = self.current_diagram()
                    .map(|s| s.diagram.settings.default_state_height)
                    .unwrap_or(60.0) * zoom;
                let rect = egui::Rect::from_center_size(pos, egui::Vec2::new(width, height));
                let rounding = self.current_diagram()
                    .map(|s| s.diagram.settings.corner_rounding)
                    .unwrap_or(12.0) * zoom;

                painter.rect(
                    rect,
                    egui::Rounding::same(rounding),
                    egui::Color32::from_rgba_unmultiplied(255, 255, 204, preview_alpha),
                    egui::Stroke::new(zoom, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                );
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "State",
                    egui::FontId::proportional(12.0 * zoom),
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha),
                );
            }
            EditMode::AddInitial => {
                // Draw a ghost initial state (filled circle)
                let radius = 8.0 * zoom;
                painter.circle_filled(
                    pos,
                    radius,
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha),
                );
            }
            EditMode::AddFinal => {
                // Draw a ghost final state (double circle)
                let outer_radius = 10.0 * zoom;
                let inner_radius = 6.0 * zoom;
                painter.circle_stroke(
                    pos,
                    outer_radius,
                    egui::Stroke::new(zoom, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                );
                painter.circle_filled(
                    pos,
                    inner_radius,
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha),
                );
            }
            EditMode::AddChoice | EditMode::AddJunction => {
                // Draw a ghost diamond
                let size = 10.0 * zoom;
                let points = vec![
                    egui::Pos2::new(pos.x, pos.y - size),
                    egui::Pos2::new(pos.x + size, pos.y),
                    egui::Pos2::new(pos.x, pos.y + size),
                    egui::Pos2::new(pos.x - size, pos.y),
                    egui::Pos2::new(pos.x, pos.y - size),
                ];
                painter.add(egui::Shape::line(
                    points,
                    egui::Stroke::new(zoom, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                ));
            }
            EditMode::AddFork | EditMode::AddJoin => {
                // Draw a ghost bar
                let width = 60.0 * zoom;
                let height = 6.0 * zoom;
                let rect = egui::Rect::from_center_size(pos, egui::Vec2::new(width, height));
                painter.rect_filled(
                    rect,
                    egui::Rounding::ZERO,
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha),
                );
            }
            EditMode::Connect => {
                // Draw a small arrow icon at cursor
                if self.pending_connection_source.is_some() {
                    // Show we're waiting for target
                    painter.circle_stroke(
                        pos,
                        8.0 * zoom,
                        egui::Stroke::new(2.0 * zoom, egui::Color32::from_rgba_unmultiplied(255, 165, 0, preview_alpha)),
                    );
                } else {
                    // Show connection start indicator
                    let size = 6.0 * zoom;
                    painter.line_segment(
                        [egui::Pos2::new(pos.x - size, pos.y), egui::Pos2::new(pos.x + size, pos.y)],
                        egui::Stroke::new(2.0 * zoom, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                    );
                    painter.line_segment(
                        [egui::Pos2::new(pos.x, pos.y - size), egui::Pos2::new(pos.x, pos.y + size)],
                        egui::Stroke::new(2.0 * zoom, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                    );
                }
            }
            EditMode::Arrow => {
                // No preview needed for selection mode
            }

            // === Use Case Diagram Previews ===
            EditMode::AddActor => {
                // Draw a ghost stick figure
                let stroke = egui::Stroke::new(1.5, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha));
                let head_y = pos.y - 20.0;
                let head_r = 8.0;
                let body_top = head_y + head_r;
                let body_bottom = body_top + 20.0;
                let arm_y = body_top + 8.0;
                let leg_bottom = body_bottom + 18.0;

                painter.circle_stroke(egui::Pos2::new(pos.x, head_y), head_r, stroke);
                painter.line_segment([egui::Pos2::new(pos.x, body_top), egui::Pos2::new(pos.x, body_bottom)], stroke);
                painter.line_segment([egui::Pos2::new(pos.x - 15.0, arm_y), egui::Pos2::new(pos.x + 15.0, arm_y)], stroke);
                painter.line_segment([egui::Pos2::new(pos.x, body_bottom), egui::Pos2::new(pos.x - 12.0, leg_bottom)], stroke);
                painter.line_segment([egui::Pos2::new(pos.x, body_bottom), egui::Pos2::new(pos.x + 12.0, leg_bottom)], stroke);
            }
            EditMode::AddUseCase => {
                // Draw a ghost ellipse
                let radius = egui::Vec2::new(50.0, 30.0);
                painter.add(egui::Shape::ellipse_stroke(
                    pos,
                    radius,
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                ));
                painter.text(
                    pos,
                    egui::Align2::CENTER_CENTER,
                    "Use Case",
                    egui::FontId::proportional(11.0),
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha),
                );
            }
            EditMode::AddSystemBoundary => {
                // Draw a ghost rectangle
                let rect = egui::Rect::from_center_size(pos, egui::Vec2::new(150.0, 200.0));
                painter.rect(
                    rect,
                    egui::Rounding::same(4.0),
                    egui::Color32::from_rgba_unmultiplied(245, 245, 245, 80),
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                );
                painter.text(
                    egui::Pos2::new(pos.x, rect.top() + 15.0),
                    egui::Align2::CENTER_CENTER,
                    "System",
                    egui::FontId::proportional(12.0),
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha),
                );
            }

            // === Sequence Diagram Previews ===
            EditMode::AddLifeline => {
                // Draw a ghost lifeline
                let head_rect = egui::Rect::from_center_size(
                    egui::Pos2::new(pos.x, pos.y - 40.0),
                    egui::Vec2::new(80.0, 30.0),
                );
                painter.rect(
                    head_rect,
                    egui::Rounding::same(2.0),
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, preview_alpha),
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                );
                // Dashed line
                let line_top = head_rect.bottom();
                let line_bottom = pos.y + 60.0;
                let mut y = line_top;
                while y < line_bottom {
                    let end_y = (y + 6.0).min(line_bottom);
                    painter.line_segment(
                        [egui::Pos2::new(pos.x, y), egui::Pos2::new(pos.x, end_y)],
                        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                    );
                    y += 10.0;
                }
                painter.text(
                    head_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Object",
                    egui::FontId::proportional(11.0),
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha),
                );
            }

            // === Activity Diagram Previews ===
            EditMode::AddAction => {
                // Draw a ghost action (rounded rectangle)
                let rect = egui::Rect::from_center_size(pos, egui::Vec2::new(100.0, 40.0));
                painter.rect(
                    rect,
                    egui::Rounding::same(10.0),
                    egui::Color32::from_rgba_unmultiplied(200, 230, 255, preview_alpha),
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                );
                painter.text(
                    pos,
                    egui::Align2::CENTER_CENTER,
                    "Action",
                    egui::FontId::proportional(11.0),
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha),
                );
            }
            EditMode::AddDecision => {
                // Draw a ghost diamond
                let size = 15.0;
                let points = vec![
                    egui::Pos2::new(pos.x, pos.y - size),
                    egui::Pos2::new(pos.x + size, pos.y),
                    egui::Pos2::new(pos.x, pos.y + size),
                    egui::Pos2::new(pos.x - size, pos.y),
                ];
                painter.add(egui::Shape::convex_polygon(
                    points,
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, preview_alpha),
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                ));
            }
            EditMode::AddSendSignal => {
                // Draw a ghost pentagon (send signal)
                let w = 50.0;
                let h = 25.0;
                let points = vec![
                    egui::Pos2::new(pos.x - w/2.0, pos.y - h/2.0),
                    egui::Pos2::new(pos.x + w/2.0 - 10.0, pos.y - h/2.0),
                    egui::Pos2::new(pos.x + w/2.0, pos.y),
                    egui::Pos2::new(pos.x + w/2.0 - 10.0, pos.y + h/2.0),
                    egui::Pos2::new(pos.x - w/2.0, pos.y + h/2.0),
                ];
                painter.add(egui::Shape::convex_polygon(
                    points,
                    egui::Color32::from_rgba_unmultiplied(255, 230, 200, preview_alpha),
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                ));
            }
            EditMode::AddAcceptEvent => {
                // Draw a ghost concave pentagon
                let w = 50.0;
                let h = 25.0;
                let points = vec![
                    egui::Pos2::new(pos.x - w/2.0, pos.y - h/2.0),
                    egui::Pos2::new(pos.x + w/2.0, pos.y - h/2.0),
                    egui::Pos2::new(pos.x + w/2.0, pos.y + h/2.0),
                    egui::Pos2::new(pos.x - w/2.0, pos.y + h/2.0),
                    egui::Pos2::new(pos.x - w/2.0 + 10.0, pos.y),
                ];
                painter.add(egui::Shape::convex_polygon(
                    points,
                    egui::Color32::from_rgba_unmultiplied(200, 255, 200, preview_alpha),
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                ));
            }
            EditMode::AddSwimlane => {
                // Draw a ghost swimlane
                let rect = egui::Rect::from_center_size(pos, egui::Vec2::new(100.0, 200.0));
                painter.rect(
                    rect,
                    egui::Rounding::ZERO,
                    egui::Color32::from_rgba_unmultiplied(230, 230, 255, 80),
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                );
                // Header line
                painter.line_segment(
                    [egui::Pos2::new(rect.left(), rect.top() + 25.0), egui::Pos2::new(rect.right(), rect.top() + 25.0)],
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                );
                painter.text(
                    egui::Pos2::new(pos.x, rect.top() + 12.0),
                    egui::Align2::CENTER_CENTER,
                    "Lane",
                    egui::FontId::proportional(11.0),
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha),
                );
            }

            _ => {}
        }
    }

    /// Show the settings window
    fn show_settings_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_settings_window;

        egui::Window::new("⚙ Settings")
            .open(&mut open)
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("Selection");
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label("Selection sensitivity:");
                        ui.add(egui::Slider::new(&mut self.settings.selection_sensitivity, 5.0..=30.0)
                            .suffix(" px"));
                    });
                    ui.label("  ↳ Radius for detecting overlapping items");

                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.label("Pivot/endpoint hit tolerance:");
                        ui.add(egui::Slider::new(&mut self.settings.pivot_hit_tolerance, 5.0..=25.0)
                            .suffix(" px"));
                    });
                    ui.label("  ↳ How close you need to click to grab a pivot point");

                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.label("Connection hit tolerance:");
                        ui.add(egui::Slider::new(&mut self.settings.connection_hit_tolerance, 5.0..=25.0)
                            .suffix(" px"));
                    });
                    ui.label("  ↳ How close you need to click to select a connection");

                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.label("Corner resize margin:");
                        ui.add(egui::Slider::new(&mut self.settings.corner_hit_margin, 5.0..=30.0)
                            .suffix(" px"));
                    });
                    ui.label("  ↳ Size of resize handles at node corners");

                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.label("Click cycle distance:");
                        ui.add(egui::Slider::new(&mut self.settings.click_cycle_distance, 5.0..=30.0)
                            .suffix(" px"));
                    });
                    ui.label("  ↳ Max distance between clicks for cycling");

                    ui.separator();
                    ui.heading("Loupe");
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label("Loupe size:");
                        ui.add(egui::Slider::new(&mut self.settings.loupe_display_radius, 80.0..=200.0)
                            .suffix(" px"));
                    });
                    // Update loupe state when setting changes
                    self.loupe.display_radius = self.settings.loupe_display_radius;

                    ui.separator();
                    ui.heading("Double-Click");
                    ui.add_space(4.0);

                    let mut dc_time = self.settings.double_click_time_ms as f32;
                    ui.horizontal(|ui| {
                        ui.label("Double-click time:");
                        ui.add(egui::Slider::new(&mut dc_time, 200.0..=800.0)
                            .suffix(" ms"));
                    });
                    self.settings.double_click_time_ms = dc_time as u128;

                    ui.horizontal(|ui| {
                        ui.label("Double-click distance:");
                        ui.add(egui::Slider::new(&mut self.settings.double_click_distance, 5.0..=20.0)
                            .suffix(" px"));
                    });

                    ui.separator();
                    ui.heading("Visual");
                    ui.add_space(4.0);

                    ui.checkbox(&mut self.settings.show_debug_info, "Show debug info in status bar");
                    ui.checkbox(&mut self.settings.highlight_on_hover, "Highlight elements on hover");

                    ui.separator();
                    ui.heading("Grid");
                    ui.add_space(4.0);

                    ui.checkbox(&mut self.settings.show_grid, "Show grid");
                    ui.checkbox(&mut self.settings.snap_to_grid, "Snap to grid");

                    ui.horizontal(|ui| {
                        ui.label("Grid size:");
                        ui.add(egui::Slider::new(&mut self.settings.grid_size, 5.0..=50.0)
                            .suffix(" px"));
                    });

                    ui.separator();
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        if ui.button("Reset to Defaults").clicked() {
                            self.settings = AppSettings::default();
                            self.loupe.display_radius = self.settings.loupe_display_radius;
                        }
                    });
                });
            });

        self.show_settings_window = open;
    }

    /// Handle keyboard input
    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        // Don't handle Delete/Backspace if a text field has focus (e.g., properties panel)
        let text_edit_has_focus = ctx.memory(|m| m.focused().is_some()) && ctx.wants_keyboard_input();

        if !text_edit_has_focus && ctx.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) {
            // First check if we have a selected pivot point to delete
            if let Some((conn_id, pivot_idx)) = self.selected_pivot.take() {
                if let Some(state) = self.current_diagram_mut() {
                    state.diagram.push_undo();
                    if let Some(conn) = state.diagram.find_connection_mut(conn_id) {
                        if pivot_idx < conn.pivot_points.len() {
                            conn.pivot_points.remove(pivot_idx);
                            // Also remove the corresponding segment curve flag
                            if pivot_idx < conn.segment_curves.len() {
                                conn.segment_curves.remove(pivot_idx);
                            }
                        }
                    }
                    state.diagram.recalculate_connections();
                    state.modified = true;
                    self.status_message = "Pivot point deleted".to_string();
                }
            } else {
                // No pivot selected, delete selected elements as before
                if let Some(state) = self.current_diagram_mut() {
                    state.diagram.push_undo();
                    state.diagram.delete_selected();
                    state.modified = true;
                    self.status_message = "Deleted".to_string();
                }
            }
        }

        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z)) {
            if ctx.input(|i| i.modifiers.shift) {
                // Redo
                if let Some(state) = self.current_diagram_mut() {
                    if state.diagram.redo() {
                        self.status_message = "Redo".to_string();
                    }
                }
            } else {
                // Undo
                if let Some(state) = self.current_diagram_mut() {
                    if state.diagram.undo() {
                        self.status_message = "Undo".to_string();
                    }
                }
            }
        }

        // Escape to cancel connection
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.pending_connection_source = None;
            self.set_edit_mode(EditMode::Arrow);
        }

        // Ctrl+S to Save, Ctrl+Shift+S to Save As
        #[cfg(not(target_arch = "wasm32"))]
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
            if ctx.input(|i| i.modifiers.shift) {
                self.save_as();
            } else {
                self.save();
            }
        }

        // Ctrl+O to Open
        #[cfg(not(target_arch = "wasm32"))]
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::O)) {
            self.open();
        }

        // Ctrl+Arrow keys to move selected nodes by 1 pixel
        if !text_edit_has_focus {
            let ctrl = ctx.input(|i| i.modifiers.ctrl);
            if ctrl {
                let mut dx = 0.0_f32;
                let mut dy = 0.0_f32;

                if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                    dx = -1.0;
                }
                if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                    dx = 1.0;
                }
                if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    dy = -1.0;
                }
                if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    dy = 1.0;
                }

                if dx != 0.0 || dy != 0.0 {
                    if let Some(state) = self.current_diagram_mut() {
                        let selected = state.diagram.selected_elements_in_order();
                        if !selected.is_empty() {
                            // Push undo on first movement
                            state.diagram.push_undo();

                            // Build set of children to avoid double-moving
                            let mut children_of_selected: std::collections::HashSet<uuid::Uuid> = std::collections::HashSet::new();
                            for &id in &selected {
                                if let Some(node) = state.diagram.find_node(id) {
                                    if let Some(s) = node.as_state() {
                                        for region in &s.regions {
                                            for &child_id in &region.children {
                                                children_of_selected.insert(child_id);
                                            }
                                        }
                                    }
                                }
                            }

                            for id in selected {
                                if children_of_selected.contains(&id) {
                                    continue;
                                }
                                if state.diagram.find_node(id).is_some() {
                                    state.diagram.translate_node_with_children(id, dx, dy);
                                } else {
                                    state.diagram.translate_element(id, dx, dy);
                                }
                            }
                            state.diagram.recalculate_connections();
                            state.modified = true;
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for JmtApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard input
        self.handle_keyboard(ctx);

        // Settings window
        if self.show_settings_window {
            self.show_settings_window(ctx);
        }

        // Top panel - Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            MenuBar::show(ui, self);
        });

        // Top panel - Toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            Toolbar::show(ui, self);
        });

        // Bottom panel - Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            StatusBar::show(ui, &self.status_message);
        });

        // Right panel - Properties
        egui::SidePanel::right("properties")
            .min_width(200.0)
            .show(ctx, |ui| {
                PropertiesPanel::show(ui, self);
            });

        // Central panel - Canvas with tabs
        egui::CentralPanel::default().show(ctx, |ui| {
            // Diagram tabs
            ui.horizontal(|ui| {
                for (i, diagram_state) in self.diagrams.iter().enumerate() {
                    let name = &diagram_state.diagram.settings.name;
                    let type_icon = diagram_state.diagram.diagram_type.icon();
                    let label = if diagram_state.modified {
                        format!("{} {}*", type_icon, name)
                    } else {
                        format!("{} {}", type_icon, name)
                    };

                    if ui.selectable_label(i == self.active_diagram, &label).clicked() {
                        self.active_diagram = i;
                    }
                }

                // New diagram dropdown
                egui::menu::menu_button(ui, "+ New", |ui| {
                    if ui.button("State Machine").clicked() {
                        self.new_diagram_of_type(DiagramType::StateMachine);
                        ui.close_menu();
                    }
                    if ui.button("Sequence").clicked() {
                        self.new_diagram_of_type(DiagramType::Sequence);
                        ui.close_menu();
                    }
                    if ui.button("Use Case").clicked() {
                        self.new_diagram_of_type(DiagramType::UseCase);
                        ui.close_menu();
                    }
                    if ui.button("Activity").clicked() {
                        self.new_diagram_of_type(DiagramType::Activity);
                        ui.close_menu();
                    }
                });
            });

            ui.separator();

            // Handle Ctrl+MouseWheel for zooming (before ScrollArea consumes the scroll)
            let ctrl_held_for_zoom = ui.input(|i| i.modifiers.ctrl);
            if ctrl_held_for_zoom {
                let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
                if scroll_delta != 0.0 {
                    // Zoom in/out based on scroll direction
                    let zoom_delta = scroll_delta * ZOOM_WHEEL_STEP / 50.0;
                    self.zoom_by(zoom_delta);
                }
            }

            // Calculate content bounds to determine scroll area size
            let content_bounds = self.current_diagram()
                .map(|s| s.diagram.content_bounds())
                .unwrap_or(jmt_core::geometry::Rect::new(0.0, 0.0, 800.0, 600.0));

            let zoom = self.zoom_level;

            // Canvas with scroll bars
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .show(ui, |ui| {
                    // Make the canvas at least as big as content bounds, scaled by zoom
                    let canvas_width = (content_bounds.x2 * zoom).max(ui.available_width());
                    let canvas_height = (content_bounds.y2 * zoom).max(ui.available_height());
                    let canvas_size = egui::vec2(canvas_width, canvas_height);

                    let (response, painter) = ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());

                    // Draw background with light gray to distinguish from scrollbar
                    painter.rect_filled(response.rect, 0.0, egui::Color32::from_gray(252));

            // Track cursor position for preview
            self.cursor_pos = response.hover_pos();

            // Canvas origin in screen space (for coordinate transformation)
            let canvas_origin = response.rect.min;

            // Check if cursor is over interactive elements and change cursor accordingly
            // First priority: active drag states (mouse is pressed and dragging)
            let mut cursor_set = false;

            if self.resize_state.is_active() {
                // Show resize cursor while actively resizing
                let corner = self.resize_state.corner;
                match corner {
                    Corner::TopLeft | Corner::BottomRight => {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNwSe);
                    }
                    Corner::TopRight | Corner::BottomLeft => {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNeSw);
                    }
                    Corner::None => {}
                }
                cursor_set = true;
            } else if self.dragging_nodes {
                // Show grabbing cursor while dragging nodes
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                cursor_set = true;
            } else if self.dragging_pivot.is_some() {
                // Show grabbing cursor while dragging pivot points
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                cursor_set = true;
            } else if self.dragging_endpoint.is_some() {
                // Show grabbing cursor while dragging connection endpoints
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                cursor_set = true;
            } else if self.dragging_label.is_some() {
                // Show grabbing cursor while dragging labels
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                cursor_set = true;
            } else if self.dragging_separator.is_some() {
                // Show resize cursor while dragging region separators
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
                cursor_set = true;
            }

            // Second priority: hover states (cursor is over interactive elements)
            // Check if mouse button is currently pressed - affects cursor type
            let mouse_button_pressed = ui.ctx().input(|i| i.pointer.primary_down());

            if !cursor_set {
                if let Some(hover_pos) = self.cursor_pos {
                    let diagram_pos = Point::new(
                        (hover_pos.x - canvas_origin.x) / zoom,
                        (hover_pos.y - canvas_origin.y) / zoom
                    );
                    let corner_margin = self.settings.corner_hit_margin;
                    let connection_tolerance = self.settings.connection_hit_tolerance;

                    // Check: corners of resizable nodes (highest priority)
                    if let Some(state) = self.current_diagram() {
                        for node in state.diagram.nodes() {
                            if node.can_resize() {
                                let corner = node.get_corner(diagram_pos, corner_margin);
                                if corner != Corner::None {
                                    match corner {
                                        Corner::TopLeft | Corner::BottomRight => {
                                            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNwSe);
                                        }
                                        Corner::TopRight | Corner::BottomLeft => {
                                            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNeSw);
                                        }
                                        Corner::None => {}
                                    }
                                    cursor_set = true;
                                    break;
                                }
                            }
                        }

                        // Second check: region separators (for resizing)
                        let separator_tolerance = 5.0;
                        if !cursor_set {
                            if state.diagram.find_region_separator_at(diagram_pos.x, diagram_pos.y, separator_tolerance).is_some() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
                                cursor_set = true;
                            }
                        }

                        // Third check: connection labels (for dragging)
                        if !cursor_set {
                            if state.diagram.find_connection_label_at(diagram_pos).is_some() {
                                // Use grab/grabbing cursor for labels based on mouse button state
                                let cursor = if mouse_button_pressed {
                                    egui::CursorIcon::Grabbing
                                } else {
                                    egui::CursorIcon::Grab
                                };
                                ui.ctx().set_cursor_icon(cursor);
                                cursor_set = true;
                            }
                        }

                        // Fourth check: pivot points and endpoints
                        if !cursor_set {
                            if let Some((_conn_id, drag_type)) = self.check_pivot_or_endpoint_at(diagram_pos) {
                                // Show grab/grabbing cursor for pivot points and endpoints
                                let cursor = if mouse_button_pressed {
                                    egui::CursorIcon::Grabbing
                                } else {
                                    egui::CursorIcon::Grab
                                };
                                ui.ctx().set_cursor_icon(cursor);
                                cursor_set = true;
                                let _ = drag_type; // suppress unused warning
                            }
                        }

                        // Fifth check: nodes (for dragging)
                        if !cursor_set {
                            if let Some(node_id) = state.diagram.find_node_at(diagram_pos) {
                                // Show grab/grabbing cursor for nodes based on mouse button state
                                let cursor = if mouse_button_pressed {
                                    egui::CursorIcon::Grabbing
                                } else {
                                    egui::CursorIcon::Grab
                                };
                                ui.ctx().set_cursor_icon(cursor);
                                cursor_set = true;

                                // Show error tooltip if node has placement error
                                if let Some(node) = state.diagram.find_node(node_id) {
                                    if node.has_error() {
                                        let overlaps = state.diagram.find_overlapping_nodes(node_id);
                                        egui::show_tooltip_at_pointer(
                                            ui.ctx(),
                                            egui::LayerId::new(egui::Order::Foreground, egui::Id::new("error_tooltip")),
                                            egui::Id::new("placement_error"),
                                            |ui| {
                                                ui.colored_label(egui::Color32::RED, "⚠ Placement Error");
                                                if overlaps.is_empty() {
                                                    ui.label("This node has an unknown overlap error.");
                                                } else {
                                                    ui.label("This node is partially overlapping:");
                                                    for (name, desc) in &overlaps {
                                                        ui.label(format!("  • {} ({})", name, desc));
                                                    }
                                                }
                                                ui.label("Move it fully inside or fully outside.");
                                            }
                                        );
                                    }
                                }
                            }
                        }

                        // Sixth check: connections
                        if !cursor_set {
                            if let Some(_conn_id) = state.diagram.find_connection_at(diagram_pos, connection_tolerance) {
                                // Use crosshair cursor for connections
                                ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                                cursor_set = true;
                            }
                        }
                    }

                    // If nothing special, egui handles default cursor
                    let _ = cursor_set;
                }
            }

            // Draw the diagram with zoom
            if let Some(state) = self.current_diagram_mut() {
                state.canvas.render_with_zoom(&state.diagram, &painter, response.rect, zoom);
            }

            // Draw cursor preview for add modes
            if let Some(pos) = self.cursor_pos {
                self.render_cursor_preview(&painter, pos, zoom);
            }

            // Handle right-click to exit add/connect mode
            if response.secondary_clicked() {
                if self.edit_mode.is_add_node() || self.edit_mode == EditMode::Connect {
                    self.pending_connection_source = None; // Clear any pending connection
                    self.set_edit_mode(EditMode::Arrow);
                    self.status_message = "Switched to Arrow mode".to_string();
                }
            }

            // Handle mouse clicks with custom double-click detection (500ms window)
            // Double-click in add mode will add element AND switch back to arrow mode
            let ctrl_held = ui.input(|i| i.modifiers.ctrl);
            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    // Transform screen coordinates to diagram coordinates
                    let diagram_pos = egui::Pos2::new(
                        (pos.x - canvas_origin.x) / zoom,
                        (pos.y - canvas_origin.y) / zoom
                    );
                    let now = Instant::now();
                    let double_click_time = self.settings.double_click_time_ms;
                    let double_click_dist = self.settings.double_click_distance;

                    // Check if this is a double-click (within time and distance threshold)
                    let is_double_click = if let (Some(last_time), Some(last_pos)) = (self.last_click_time, self.last_click_pos) {
                        let time_diff = now.duration_since(last_time).as_millis();
                        let distance = ((pos.x - last_pos.x).powi(2) + (pos.y - last_pos.y).powi(2)).sqrt();
                        time_diff <= double_click_time && distance <= double_click_dist
                    } else {
                        false
                    };

                    if is_double_click {
                        // Double-click detected
                        self.last_click_time = None;
                        self.last_click_pos = None;

                        // Cache settings for use in closures
                        let connection_tolerance = self.settings.connection_hit_tolerance;
                        let pivot_tolerance = self.settings.pivot_hit_tolerance;

                        // First check if double-clicked on a connection label - re-adjoin it
                        let point = Point::new(diagram_pos.x, diagram_pos.y);
                        let label_clicked = self.current_diagram()
                            .and_then(|state| state.diagram.find_connection_label_at(point));

                        if let Some(conn_id) = label_clicked {
                            // Re-adjoin the label
                            if let Some(state) = self.current_diagram_mut() {
                                if let Some(conn) = state.diagram.find_connection_mut(conn_id) {
                                    conn.set_text_adjoined(true);
                                }
                                state.modified = true;
                                // Check for overlap and shift target if needed
                                state.diagram.adjust_for_label_overlap(conn_id);
                            }
                            self.status_message = "Label adjoined to connection".to_string();
                        } else {
                            // Check if double-clicked on a connection to add pivot point
                            let conn_clicked = self.current_diagram()
                                .and_then(|state| state.diagram.find_connection_at(point, connection_tolerance));

                            if let Some(conn_id) = conn_clicked {
                                // Check if we're clicking on an existing pivot point - delete it
                                let existing_pivot = self.current_diagram()
                                    .and_then(|state| state.diagram.find_connection(conn_id))
                                    .and_then(|conn| conn.find_pivot_at(point, pivot_tolerance));

                                if let Some(pivot_idx) = existing_pivot {
                                    // Delete the pivot point
                                    if let Some(state) = self.current_diagram_mut() {
                                        state.diagram.push_undo();
                                        if let Some(conn) = state.diagram.find_connection_mut(conn_id) {
                                            conn.pivot_points.remove(pivot_idx);
                                            if pivot_idx < conn.segment_curves.len() {
                                                conn.segment_curves.remove(pivot_idx);
                                            }
                                            state.modified = true;
                                        }
                                        state.diagram.recalculate_connections();
                                    }
                                    self.selected_pivot = None;
                                    self.status_message = "Pivot point removed".to_string();
                                } else {
                                    // Check if too close to existing pivot or endpoint
                                    let too_close = self.current_diagram()
                                        .and_then(|state| state.diagram.find_connection(conn_id))
                                        .map(|conn| {
                                            // Check distance to all existing pivots
                                            for pivot in &conn.pivot_points {
                                                let dx = pivot.x - point.x;
                                                let dy = pivot.y - point.y;
                                                if (dx * dx + dy * dy).sqrt() < MIN_PIVOT_DISTANCE {
                                                    return true;
                                                }
                                            }
                                            false
                                        })
                                        .unwrap_or(false);

                                    if too_close {
                                        self.status_message = "Too close to existing pivot point".to_string();
                                    } else {
                                        // Add pivot point at click location
                                        if let Some(state) = self.current_diagram_mut() {
                                            state.diagram.push_undo();
                                            if let Some(conn) = state.diagram.find_connection_mut(conn_id) {
                                                // Find which segment was clicked and insert pivot there
                                                let insert_idx = conn.find_segment_at(point, connection_tolerance);
                                                conn.pivot_points.insert(insert_idx, point);
                                                // Also insert a curve flag for the new segment created
                                                conn.segment_curves.insert(insert_idx, false);
                                                state.modified = true;
                                            }
                                            state.diagram.recalculate_connections();
                                        }
                                        self.status_message = "Pivot point added".to_string();
                                    }
                                }
                            } else {
                                // Check if double-clicked on a state with sub-statemachine
                                let sub_state = self.current_diagram()
                                    .and_then(|state| state.diagram.find_node_at(point))
                                    .and_then(|node_id| {
                                        self.current_diagram()
                                            .and_then(|state| state.diagram.find_node(node_id))
                                            .and_then(|node| node.as_state())
                                            .filter(|s| s.has_substatemachine())
                                            .map(|_| node_id)
                                    });

                                if let Some(state_id) = sub_state {
                                    // Open the sub-statemachine
                                    self.open_substatemachine(state_id);
                                } else if self.edit_mode.is_add_node() {
                                    // Double-click in add mode: first click already added node, switch to Arrow
                                    self.set_edit_mode(EditMode::Arrow);
                                    self.status_message = "Switched to Arrow mode".to_string();
                                } else {
                                    // For non-add modes, handle normally (e.g., Arrow mode selection)
                                    self.handle_canvas_click(diagram_pos, true, ctrl_held);
                                }
                            }
                        }
                    } else {
                        // Single click: record time/pos for potential double-click detection
                        self.last_click_time = Some(now);
                        self.last_click_pos = Some(pos);

                        // Check if clicked on a sub-statemachine icon
                        let click_point = Point::new(diagram_pos.x, diagram_pos.y);
                        if let Some(state_id) = self.check_substatemachine_icon_at(click_point) {
                            // Show preview popup
                            self.preview_substatemachine = Some(state_id);
                            self.status_message = "Sub-statemachine preview".to_string();
                        } else {
                            self.handle_canvas_click(diagram_pos, false, ctrl_held);
                        }
                    }
                }
            }

            // Handle drag start - determine if we're resizing, dragging nodes, or marquee selecting
            if response.drag_started() {
                if let Some(pos) = response.interact_pointer_pos() {
                    // Transform screen coordinates to diagram coordinates
                    let diagram_pos = egui::Pos2::new(
                        (pos.x - canvas_origin.x) / zoom,
                        (pos.y - canvas_origin.y) / zoom
                    );
                    let point = Point::new(diagram_pos.x, diagram_pos.y);
                    let corner_margin = self.settings.corner_hit_margin;

                    // First, check if we clicked on a corner of ANY resizable node
                    // (prioritize selected nodes, then check all nodes)
                    let mut corner_info: Option<(NodeId, Corner)> = None;
                    if let Some(state) = self.current_diagram() {
                        // Check selected nodes first
                        for node_id in state.diagram.selected_nodes() {
                            if let Some(node) = state.diagram.find_node(node_id) {
                                if node.can_resize() {
                                    let corner = node.get_corner(point, corner_margin);
                                    if corner != Corner::None {
                                        corner_info = Some((node_id, corner));
                                        break;
                                    }
                                }
                            }
                        }
                        // If no selected node corner found, check all nodes
                        if corner_info.is_none() {
                            for node in state.diagram.nodes() {
                                if node.can_resize() {
                                    let corner = node.get_corner(point, corner_margin);
                                    if corner != Corner::None {
                                        corner_info = Some((node.id(), corner));
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if let Some((node_id, corner)) = corner_info {
                        // We're starting a resize operation
                        self.status_message = "Resizing...".to_string();
                        if self.edit_mode != EditMode::Arrow {
                            self.set_edit_mode(EditMode::Arrow);
                        }
                        if let Some(state) = self.current_diagram_mut() {
                            // Select the node if not already selected
                            if !state.diagram.selected_nodes().contains(&node_id) {
                                state.diagram.clear_selection();
                                state.diagram.select_node(node_id);
                            }
                            state.diagram.push_undo();
                        }
                        self.resize_state.start(node_id, corner);
                        self.dragging_nodes = false;
                        self.dragging_separator = None;
                        self.selection_rect.clear();
                    } else {
                        // Check if we clicked on a region separator (for resizing regions)
                        let separator_tolerance = 5.0;
                        let clicked_separator = self.current_diagram()
                            .and_then(|state| state.diagram.find_region_separator_at(diagram_pos.x, diagram_pos.y, separator_tolerance));

                        if let Some((state_id, region_idx)) = clicked_separator {
                            // Start dragging a separator
                            self.status_message = "Resizing regions...".to_string();
                            if let Some(state) = self.current_diagram_mut() {
                                state.diagram.select_region(state_id, region_idx);
                                state.diagram.push_undo();
                            }
                            self.dragging_separator = Some((state_id, region_idx));
                            self.dragging_nodes = false;
                            self.dragging_label = None;
                            self.selection_rect.clear();
                        } else {
                            // Use click cycling to handle overlapping items
                            // This allows clicking multiple times to cycle through items at the same location
                            if !self.handle_click_with_cycling(point) {
                                // Nothing to select - start marquee or lasso selection
                                if self.edit_mode == EditMode::Arrow {
                                    // We're starting a marquee selection (only in Arrow mode)
                                    self.dragging_nodes = false;
                                    self.dragging_label = None;
                                    self.selected_pivot = None;
                                    self.selection_rect.start = Some(diagram_pos);
                                    self.selection_rect.current = Some(diagram_pos);
                                    // Clear current selection when starting a new marquee
                                    if let Some(state) = self.current_diagram_mut() {
                                        state.diagram.clear_selection();
                                    }
                                } else if self.edit_mode == EditMode::Lasso {
                                    // We're starting a lasso selection
                                    self.dragging_nodes = false;
                                    self.dragging_label = None;
                                    self.selected_pivot = None;
                                    self.lasso_points.clear();
                                    self.lasso_points.push(diagram_pos);
                                    // Clear current selection when starting a new lasso
                                    if let Some(state) = self.current_diagram_mut() {
                                        state.diagram.clear_selection();
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Handle dragging
            if response.dragged() {
                if let Some(pos) = response.interact_pointer_pos() {
                    // Transform screen coordinates to diagram coordinates
                    let diagram_pos = egui::Pos2::new(
                        (pos.x - canvas_origin.x) / zoom,
                        (pos.y - canvas_origin.y) / zoom
                    );
                    let delta = response.drag_delta();
                    // Scale delta to diagram space
                    let diagram_delta_x = delta.x / zoom;
                    let diagram_delta_y = delta.y / zoom;

                    if self.resize_state.is_active() {
                        // Handle resize
                        let node_id = self.resize_state.node_id.unwrap();
                        let corner = self.resize_state.corner;
                        let min_width = 40.0;
                        let min_height = 30.0;

                        if let Some(state) = self.current_diagram_mut() {
                            if let Some(node) = state.diagram.find_node_mut(node_id) {
                                node.resize_from_corner(corner, diagram_delta_x, diagram_delta_y, min_width, min_height);
                            }
                            state.diagram.recalculate_connections();
                            state.modified = true;
                        }
                    } else if let Some((state_id, region_idx)) = self.dragging_separator {
                        // Dragging a region separator
                        if let Some(state) = self.current_diagram_mut() {
                            state.diagram.move_region_separator(state_id, region_idx, diagram_delta_y);
                            state.modified = true;
                        }
                    } else if let Some(conn_id) = self.dragging_label {
                        // Dragging a connection label
                        if let Some(state) = self.current_diagram_mut() {
                            if let Some(conn) = state.diagram.find_connection_mut(conn_id) {
                                // Get the current midpoint to verify connection has segments
                                if conn.midpoint().is_some() {
                                    // Calculate new offset from midpoint
                                    let current_offset = conn.label_offset.unwrap_or((0.0, -15.0));
                                    let new_offset = (
                                        current_offset.0 + diagram_delta_x,
                                        current_offset.1 + diagram_delta_y,
                                    );
                                    conn.set_label_offset(Some(new_offset));
                                }
                            }
                            state.modified = true;
                        }
                    } else if let Some((conn_id, pivot_idx)) = self.dragging_pivot {
                        // Dragging a pivot point
                        let point = Point::new(diagram_pos.x, diagram_pos.y);
                        if let Some(state) = self.current_diagram_mut() {
                            if let Some(conn) = state.diagram.find_connection_mut(conn_id) {
                                if pivot_idx < conn.pivot_points.len() {
                                    conn.pivot_points[pivot_idx] = point;
                                }
                            }
                            state.diagram.recalculate_connections();
                            state.modified = true;
                        }
                    } else if let Some((conn_id, is_source)) = self.dragging_endpoint {
                        // Dragging an endpoint - snap to nearest side of connected node
                        let point = Point::new(diagram_pos.x, diagram_pos.y);
                        if let Some(state) = self.current_diagram_mut() {
                            let node_id = if is_source {
                                state.diagram.find_connection(conn_id).map(|c| c.source_id)
                            } else {
                                state.diagram.find_connection(conn_id).map(|c| c.target_id)
                            };

                            if let Some(nid) = node_id {
                                if let Some(node) = state.diagram.find_node(nid) {
                                    let bounds = node.bounds();
                                    let (side, offset) = Self::find_nearest_side_and_offset(bounds, point);

                                    if let Some(conn) = state.diagram.find_connection_mut(conn_id) {
                                        if is_source {
                                            conn.source_side = side;
                                            conn.source_offset = offset;
                                        } else {
                                            conn.target_side = side;
                                            conn.target_offset = offset;
                                        }
                                    }
                                }
                            }
                            state.diagram.recalculate_connections();
                            state.modified = true;
                        }
                    } else if self.edit_mode == EditMode::Arrow {
                        if self.dragging_nodes {
                            // Move selected elements (nodes, lifelines, actors, etc.)
                            if let Some(state) = self.current_diagram_mut() {
                                let selected = state.diagram.selected_elements_in_order();
                                if !selected.is_empty() {
                                    // For state machine nodes, move children with parents
                                    // Build set of all children of selected nodes to avoid double-moving
                                    let mut children_of_selected: std::collections::HashSet<uuid::Uuid> = std::collections::HashSet::new();
                                    for &id in &selected {
                                        if let Some(node) = state.diagram.find_node(id) {
                                            if let Some(s) = node.as_state() {
                                                for region in &s.regions {
                                                    for &child_id in &region.children {
                                                        children_of_selected.insert(child_id);
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    for id in selected {
                                        // Skip if this node is a child of another selected node
                                        if children_of_selected.contains(&id) {
                                            continue;
                                        }
                                        // For nodes, use translate_node_with_children to move children too
                                        if state.diagram.find_node(id).is_some() {
                                            state.diagram.translate_node_with_children(id, diagram_delta_x, diagram_delta_y);
                                        } else {
                                            // For other elements (lifelines, actors, etc.)
                                            state.diagram.translate_element(id, diagram_delta_x, diagram_delta_y);
                                        }
                                    }
                                    state.diagram.recalculate_connections();
                                    state.modified = true;
                                }
                            }
                        } else {
                            // Update marquee selection rectangle (in diagram space)
                            self.selection_rect.current = Some(diagram_pos);
                        }
                    } else if self.edit_mode == EditMode::Lasso {
                        // Add points to the lasso path (with some distance threshold to avoid too many points)
                        if let Some(last) = self.lasso_points.last() {
                            let dist = ((diagram_pos.x - last.x).powi(2) + (diagram_pos.y - last.y).powi(2)).sqrt();
                            if dist > 3.0 {
                                self.lasso_points.push(diagram_pos);
                            }
                        }
                    }
                }
            }

            // Draw selection rectangle if active (scale to screen space)
            if self.selection_rect.is_active() {
                if let Some(rect) = self.selection_rect.to_egui_rect() {
                    // Scale the rectangle to screen space and add canvas offset
                    let screen_rect = egui::Rect::from_min_max(
                        egui::Pos2::new(rect.min.x * zoom + canvas_origin.x, rect.min.y * zoom + canvas_origin.y),
                        egui::Pos2::new(rect.max.x * zoom + canvas_origin.x, rect.max.y * zoom + canvas_origin.y),
                    );
                    // Draw selection rectangle with semi-transparent fill
                    painter.rect(
                        screen_rect,
                        egui::Rounding::ZERO,
                        egui::Color32::from_rgba_unmultiplied(100, 150, 255, 50),
                        egui::Stroke::new(zoom, egui::Color32::from_rgb(100, 150, 255)),
                    );
                }

                // Draw all pivot points during marquee selection so user can see what they can select
                if let Some(state) = self.current_diagram() {
                    let handle_radius = 5.0 * zoom;
                    let handle_stroke = egui::Stroke::new(zoom, egui::Color32::BLACK);
                    let selection_rect = self.selection_rect.to_core_rect();

                    for conn in state.diagram.connections() {
                        for pivot in &conn.pivot_points {
                            let screen_pos = egui::Pos2::new(
                                pivot.x * zoom + canvas_origin.x,
                                pivot.y * zoom + canvas_origin.y,
                            );

                            // Check if pivot is inside selection rect - highlight it
                            let is_inside = selection_rect.as_ref()
                                .map(|r| r.contains_point(*pivot))
                                .unwrap_or(false);

                            let fill_color = if is_inside {
                                egui::Color32::from_rgb(100, 200, 100) // Green when inside selection
                            } else {
                                egui::Color32::from_rgba_unmultiplied(255, 215, 0, 180) // Semi-transparent gold
                            };

                            painter.circle_filled(screen_pos, handle_radius, fill_color);
                            painter.circle_stroke(screen_pos, handle_radius, handle_stroke);
                        }
                    }
                }
            }

            // Draw lasso path if active (scale to screen space)
            if self.lasso_points.len() > 1 {
                // Scale lasso points to screen space and add canvas offset
                let screen_points: Vec<egui::Pos2> = self.lasso_points.iter()
                    .map(|p| egui::Pos2::new(p.x * zoom + canvas_origin.x, p.y * zoom + canvas_origin.y))
                    .collect();

                // Draw the lasso line
                painter.add(egui::Shape::line(
                    screen_points.clone(),
                    egui::Stroke::new(2.0 * zoom, egui::Color32::from_rgb(100, 150, 255)),
                ));

                // Draw closing line (dashed effect using dotted line to start point)
                if screen_points.len() > 2 {
                    if let (Some(first), Some(last)) = (screen_points.first(), screen_points.last()) {
                        painter.line_segment(
                            [*last, *first],
                            egui::Stroke::new(zoom, egui::Color32::from_rgba_unmultiplied(100, 150, 255, 128)),
                        );
                    }
                }
            }

            // Handle loupe interactions (before drawing so we can modify state)
            let mut loupe_selection: Option<ClickCandidate> = None;
            let mut loupe_clicked_outside = false;

            if self.loupe.visible {
                // Check for Escape key to close loupe
                if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.loupe.hide();
                    self.status_message = "Selection cancelled".to_string();
                }

                // Check for clicks on loupe items
                if let Some(center) = self.loupe.center {
                    let loupe_screen_center = egui::Pos2::new(
                        center.x * zoom + canvas_origin.x,
                        center.y * zoom + canvas_origin.y,
                    );
                    let loupe_offset = egui::vec2(self.loupe.display_radius + 20.0, -self.loupe.display_radius - 20.0);
                    let loupe_center = loupe_screen_center + loupe_offset;
                    let loupe_center = egui::Pos2::new(
                        loupe_center.x.clamp(self.loupe.display_radius, response.rect.max.x - self.loupe.display_radius),
                        loupe_center.y.clamp(self.loupe.display_radius + response.rect.min.y, response.rect.max.y - self.loupe.display_radius),
                    );

                    let item_spacing = 35.0;
                    let start_y = loupe_center.y - (self.loupe.candidates.len() as f32 - 1.0) * item_spacing / 2.0;

                    // Check if click is on a loupe item (but not if loupe just opened this frame)
                    if ui.ctx().input(|i| i.pointer.primary_clicked()) && !self.loupe.just_opened {
                        if let Some(mouse_pos) = ui.ctx().input(|i| i.pointer.hover_pos()) {
                            // Check if click is inside loupe circle
                            let dist_to_loupe = ((mouse_pos.x - loupe_center.x).powi(2) + (mouse_pos.y - loupe_center.y).powi(2)).sqrt();
                            if dist_to_loupe <= self.loupe.display_radius {
                                // Check each item
                                for (i, candidate) in self.loupe.candidates.iter().enumerate() {
                                    let item_y = start_y + i as f32 * item_spacing;
                                    let item_center = egui::Pos2::new(loupe_center.x, item_y);
                                    let button_rect = egui::Rect::from_center_size(
                                        item_center,
                                        egui::vec2(self.loupe.display_radius * 1.6, 28.0),
                                    );
                                    if button_rect.contains(mouse_pos) {
                                        loupe_selection = Some(*candidate);
                                        break;
                                    }
                                }
                            } else {
                                // Clicked outside loupe - close it
                                loupe_clicked_outside = true;
                            }
                        }
                    }
                }

                // Clear just_opened flag at end of frame processing
                self.loupe.just_opened = false;
            }

            // Process loupe selection (after the borrow ends)
            if let Some(candidate) = loupe_selection {
                eprintln!("[LOUPE] Selecting: {:?}", candidate);
                self.select_candidate(candidate);
                self.loupe.hide();
            } else if loupe_clicked_outside {
                self.loupe.hide();
                self.status_message = "Selection cancelled".to_string();
            }

            // Draw magnifying loupe if visible
            if self.loupe.visible {
                if let Some(center) = self.loupe.center {
                    let loupe_screen_center = egui::Pos2::new(
                        center.x * zoom + canvas_origin.x,
                        center.y * zoom + canvas_origin.y,
                    );

                    // Position loupe slightly offset from click point so user can see both
                    let loupe_offset = egui::vec2(self.loupe.display_radius + 20.0, -self.loupe.display_radius - 20.0);
                    let loupe_center = loupe_screen_center + loupe_offset;

                    // Keep loupe on screen
                    let loupe_center = egui::Pos2::new(
                        loupe_center.x.clamp(self.loupe.display_radius, response.rect.max.x - self.loupe.display_radius),
                        loupe_center.y.clamp(self.loupe.display_radius + response.rect.min.y, response.rect.max.y - self.loupe.display_radius),
                    );

                    // Draw loupe background (dark circle with border)
                    painter.circle_filled(
                        loupe_center,
                        self.loupe.display_radius,
                        egui::Color32::from_rgb(40, 40, 50),
                    );
                    painter.circle_stroke(
                        loupe_center,
                        self.loupe.display_radius,
                        egui::Stroke::new(3.0, egui::Color32::from_rgb(100, 150, 255)),
                    );

                    // Draw crosshair at center
                    let cross_size = 10.0;
                    painter.line_segment(
                        [loupe_center - egui::vec2(cross_size, 0.0), loupe_center + egui::vec2(cross_size, 0.0)],
                        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100)),
                    );
                    painter.line_segment(
                        [loupe_center - egui::vec2(0.0, cross_size), loupe_center + egui::vec2(0.0, cross_size)],
                        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100)),
                    );

                    // Draw candidates as labeled items
                    let item_spacing = 35.0;
                    let start_y = loupe_center.y - (self.loupe.candidates.len() as f32 - 1.0) * item_spacing / 2.0;

                    for (i, candidate) in self.loupe.candidates.iter().enumerate() {
                        let item_y = start_y + i as f32 * item_spacing;
                        let item_center = egui::Pos2::new(loupe_center.x, item_y);

                        // Get candidate info
                        let (label, color) = match candidate {
                            ClickCandidate::Node(id) => {
                                let name = self.current_diagram()
                                    .and_then(|s| s.diagram.find_node(*id))
                                    .map(|n| n.name().to_string())
                                    .unwrap_or_else(|| "Node".to_string());
                                (format!("📦 {}", name), egui::Color32::from_rgb(255, 220, 150))
                            }
                            ClickCandidate::Endpoint(_, is_source) => {
                                let label = if *is_source { "⬤ Source" } else { "⬤ Target" };
                                (label.to_string(), egui::Color32::from_rgb(100, 149, 237))
                            }
                            ClickCandidate::Pivot(_, idx) => {
                                (format!("◉ Pivot {}", idx + 1), egui::Color32::GOLD)
                            }
                            ClickCandidate::Label(_) => {
                                ("📝 Label".to_string(), egui::Color32::WHITE)
                            }
                            ClickCandidate::Connection(_) => {
                                ("➜ Connection".to_string(), egui::Color32::LIGHT_GRAY)
                            }
                        };

                        // Draw selection button background
                        let button_rect = egui::Rect::from_center_size(
                            item_center,
                            egui::vec2(self.loupe.display_radius * 1.6, 28.0),
                        );

                        // Check if mouse is over this item for highlight
                        let mouse_over = ui.ctx().input(|i| {
                            i.pointer.hover_pos()
                                .map(|p| button_rect.contains(p))
                                .unwrap_or(false)
                        });

                        let bg_color = if mouse_over {
                            egui::Color32::from_rgb(80, 100, 140)
                        } else {
                            egui::Color32::from_rgb(60, 60, 70)
                        };

                        painter.rect_filled(button_rect, egui::Rounding::same(4.0), bg_color);
                        painter.rect_stroke(button_rect, egui::Rounding::same(4.0), egui::Stroke::new(1.0, color));

                        // Draw label
                        painter.text(
                            item_center,
                            egui::Align2::CENTER_CENTER,
                            &label,
                            egui::FontId::proportional(14.0),
                            color,
                        );
                    }

                    // Draw instruction text
                    painter.text(
                        egui::Pos2::new(loupe_center.x, loupe_center.y + self.loupe.display_radius - 15.0),
                        egui::Align2::CENTER_CENTER,
                        "Click to select, Esc to cancel",
                        egui::FontId::proportional(10.0),
                        egui::Color32::from_rgba_unmultiplied(200, 200, 200, 180),
                    );
                }
            }

            // Handle drag end
            if response.drag_stopped() {
                if self.dragging_separator.is_some() {
                    // Finished dragging a region separator
                    if let Some(state) = self.current_diagram_mut() {
                        state.diagram.clear_region_selection();
                    }
                    self.dragging_separator = None;
                    self.status_message = "Ready".to_string();
                } else if self.dragging_label.is_some() {
                    // Finished dragging a label
                    self.dragging_label = None;
                    self.status_message = "Label moved".to_string();
                } else if self.dragging_pivot.is_some() {
                    // Finished dragging a pivot point
                    self.dragging_pivot = None;
                    self.status_message = "Pivot point moved".to_string();
                } else if self.dragging_endpoint.is_some() {
                    // Finished dragging an endpoint
                    self.dragging_endpoint = None;
                    self.status_message = "Endpoint moved".to_string();
                } else if self.resize_state.is_active() {
                    // Finished resizing - re-evaluate all node regions
                    // since resizing a state may affect containment
                    if let Some(state) = self.current_diagram_mut() {
                        state.diagram.update_all_node_regions();
                    }
                    self.resize_state.clear();
                    self.status_message = "Ready".to_string();
                } else if self.edit_mode == EditMode::Arrow {
                    if self.dragging_nodes {
                        // Re-evaluate ALL node region assignments after drag
                        // This handles cases where:
                        // - A node was dragged into/out of another state
                        // - A parent state was moved, affecting child relationships
                        if let Some(state) = self.current_diagram_mut() {
                            state.diagram.update_all_node_regions();
                        }
                        self.dragging_nodes = false;
                    } else {
                        // Complete marquee selection
                        if let Some(rect) = self.selection_rect.to_core_rect() {
                            // First pass: find pivot points in rect (immutable borrow)
                            let pivot_info: Option<(uuid::Uuid, usize, usize)> = self.current_diagram()
                                .map(|state| {
                                    let mut pivots_selected = 0;
                                    let mut conn_to_select = None;
                                    for conn in state.diagram.connections() {
                                        for (idx, pivot) in conn.pivot_points.iter().enumerate() {
                                            if rect.contains_point(*pivot) {
                                                conn_to_select = Some((conn.id, idx));
                                                pivots_selected += 1;
                                            }
                                        }
                                    }
                                    conn_to_select.map(|(id, idx)| (id, idx, pivots_selected))
                                })
                                .flatten();

                            // Second pass: do the selection (mutable borrow)
                            if let Some(state) = self.current_diagram_mut() {
                                state.diagram.select_elements_in_rect(&rect);

                                // Select connection with pivot points
                                if let Some((conn_id, _, _)) = pivot_info {
                                    state.diagram.select_connection(conn_id);
                                }
                            }

                            // Set selected pivot (no borrow conflict now)
                            if let Some((conn_id, pivot_idx, _)) = pivot_info {
                                self.selected_pivot = Some((conn_id, pivot_idx));
                            }

                            // Build status message
                            let (node_count, conn_selected) = self.current_diagram()
                                .map(|state| (
                                    state.diagram.selected_nodes().len(),
                                    state.diagram.selected_connection().is_some()
                                ))
                                .unwrap_or((0, false));

                            let pivots_selected = pivot_info.map(|(_, _, count)| count).unwrap_or(0);

                            if node_count > 0 || conn_selected {
                                let mut parts = Vec::new();
                                if node_count > 0 {
                                    parts.push(format!("{} node(s)", node_count));
                                }
                                if pivots_selected > 0 {
                                    parts.push(format!("{} pivot(s)", pivots_selected));
                                }
                                self.status_message = format!("Selected {}", parts.join(", "));
                            } else {
                                self.status_message = "Ready".to_string();
                            }
                        }
                    }
                } else if self.edit_mode == EditMode::Lasso {
                    // Complete lasso selection
                    if self.lasso_points.len() >= 3 {
                        // Convert lasso points to core Points
                        let polygon: Vec<Point> = self.lasso_points
                            .iter()
                            .map(|p| Point::new(p.x, p.y))
                            .collect();

                        if let Some(state) = self.current_diagram_mut() {
                            state.diagram.select_elements_in_polygon(&polygon);
                            let count = state.diagram.selected_elements_in_order().len();
                            if count > 0 {
                                self.status_message = format!("Selected {} element(s)", count);
                            } else {
                                self.status_message = "Ready".to_string();
                            }
                        }
                    }
                    self.lasso_points.clear();
                }

                // Clear selection rect and reset state
                self.selection_rect.clear();
                self.dragging_nodes = false;
                self.dragging_label = None;
                self.dragging_pivot = None;
                self.dragging_endpoint = None;
            }
            }); // End ScrollArea
        });

        // Render sub-statemachine preview popup if active
        self.render_substatemachine_preview(ctx);
    }
}
