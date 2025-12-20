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

/// State for a single open diagram
pub struct DiagramState {
    pub diagram: Diagram,
    pub canvas: DiagramCanvas,
    pub modified: bool,
}

impl DiagramState {
    pub fn new(diagram: Diagram) -> Self {
        Self {
            canvas: DiagramCanvas::new(),
            diagram,
            modified: false,
        }
    }
}

/// Double-click detection threshold in milliseconds
const DOUBLE_CLICK_TIME_MS: u128 = 500;
/// Maximum distance (in pixels) between clicks to count as double-click
const DOUBLE_CLICK_DISTANCE: f32 = 10.0;

/// The main JMT application
pub struct JmtApp {
    /// Open diagrams
    diagrams: Vec<DiagramState>,
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
            cursor_pos: None,
            resize_state: ResizeState::default(),
            lasso_points: Vec::new(),
            last_click_time: None,
            last_click_pos: None,
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

    /// Set the edit mode
    pub fn set_edit_mode(&mut self, mode: EditMode) {
        // Special handling: If switching to Connect mode with multiple nodes selected,
        // connect them in sequence automatically
        if mode == EditMode::Connect {
            if let Some(state) = self.current_diagram_mut() {
                let selected = state.diagram.selected_nodes_in_order();
                if selected.len() >= 2 {
                    // Connect nodes in sequence: 1->2, 2->3, 3->4, etc.
                    state.diagram.push_undo();
                    let mut connections_made = 0;
                    for i in 0..selected.len() - 1 {
                        let source = selected[i];
                        let target = selected[i + 1];
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

        self.edit_mode = mode;
        self.pending_connection_source = None;
        self.status_message = format!("Mode: {}", mode.display_name());
    }

    /// Handle canvas click
    /// If `switch_to_arrow` is true, switch back to Arrow mode after adding element
    /// If `ctrl_held` is true, toggle selection instead of replacing it
    fn handle_canvas_click(&mut self, pos: egui::Pos2, switch_to_arrow: bool, ctrl_held: bool) {
        let point = Point::new(pos.x, pos.y);
        let edit_mode = self.edit_mode;
        let pending_source = self.pending_connection_source;

        let Some(state) = self.current_diagram_mut() else {
            return;
        };

        match edit_mode {
            EditMode::Arrow => {
                // Try to select any element (node, lifeline, actor, use case, action, etc.)
                if let Some(element_id) = state.diagram.find_element_at(point) {
                    let name = state.diagram.get_element_name(element_id)
                        .unwrap_or_default();

                    if ctrl_held {
                        // Ctrl+Click: Toggle selection
                        state.diagram.toggle_element_selection(element_id);
                        let selected_count = state.diagram.selected_nodes().len();
                        self.status_message = format!("Selected {} element(s)", selected_count);
                    } else {
                        // Regular click: Select only this element
                        state.diagram.select_element(element_id);
                        self.status_message = format!("Selected: {}", name);
                    }
                } else if let Some(conn_id) = state.diagram.find_connection_at(point, 10.0) {
                    state.diagram.select_connection(conn_id);
                    self.status_message = "Selected connection".to_string();
                } else {
                    // Only clear selection if Ctrl is not held
                    if !ctrl_held {
                        state.diagram.clear_selection();
                        self.status_message = "Ready".to_string();
                    }
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
    fn render_cursor_preview(&self, painter: &egui::Painter, pos: egui::Pos2) {
        let preview_alpha = 128u8; // Semi-transparent

        match self.edit_mode {
            EditMode::AddState => {
                // Draw a ghost state rectangle
                let width = self.current_diagram()
                    .map(|s| s.diagram.settings.default_state_width)
                    .unwrap_or(100.0);
                let height = self.current_diagram()
                    .map(|s| s.diagram.settings.default_state_height)
                    .unwrap_or(60.0);
                let rect = egui::Rect::from_center_size(pos, egui::Vec2::new(width, height));
                let rounding = self.current_diagram()
                    .map(|s| s.diagram.settings.corner_rounding)
                    .unwrap_or(12.0);

                painter.rect(
                    rect,
                    egui::Rounding::same(rounding),
                    egui::Color32::from_rgba_unmultiplied(255, 255, 204, preview_alpha),
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                );
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "State",
                    egui::FontId::proportional(12.0),
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha),
                );
            }
            EditMode::AddInitial => {
                // Draw a ghost initial state (filled circle)
                let radius = 8.0;
                painter.circle_filled(
                    pos,
                    radius,
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha),
                );
            }
            EditMode::AddFinal => {
                // Draw a ghost final state (double circle)
                let outer_radius = 10.0;
                let inner_radius = 6.0;
                painter.circle_stroke(
                    pos,
                    outer_radius,
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                );
                painter.circle_filled(
                    pos,
                    inner_radius,
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha),
                );
            }
            EditMode::AddChoice | EditMode::AddJunction => {
                // Draw a ghost diamond
                let size = 10.0;
                let points = vec![
                    egui::Pos2::new(pos.x, pos.y - size),
                    egui::Pos2::new(pos.x + size, pos.y),
                    egui::Pos2::new(pos.x, pos.y + size),
                    egui::Pos2::new(pos.x - size, pos.y),
                    egui::Pos2::new(pos.x, pos.y - size),
                ];
                painter.add(egui::Shape::line(
                    points,
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                ));
            }
            EditMode::AddFork | EditMode::AddJoin => {
                // Draw a ghost bar
                let width = 60.0;
                let height = 6.0;
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
                        8.0,
                        egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(255, 165, 0, preview_alpha)),
                    );
                } else {
                    // Show connection start indicator
                    let size = 6.0;
                    painter.line_segment(
                        [egui::Pos2::new(pos.x - size, pos.y), egui::Pos2::new(pos.x + size, pos.y)],
                        egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
                    );
                    painter.line_segment(
                        [egui::Pos2::new(pos.x, pos.y - size), egui::Pos2::new(pos.x, pos.y + size)],
                        egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, preview_alpha)),
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

    /// Handle keyboard input
    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) {
            if let Some(state) = self.current_diagram_mut() {
                state.diagram.push_undo();
                state.diagram.delete_selected();
                state.modified = true;
                self.status_message = "Deleted".to_string();
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
    }
}

impl eframe::App for JmtApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard input
        self.handle_keyboard(ctx);

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

            // Canvas
            let available_size = ui.available_size();
            let (response, painter) = ui.allocate_painter(available_size, egui::Sense::click_and_drag());

            // Draw background
            painter.rect_filled(response.rect, 0.0, egui::Color32::WHITE);

            // Track cursor position for preview
            self.cursor_pos = response.hover_pos();

            // Draw the diagram
            if let Some(state) = self.current_diagram() {
                state.canvas.render(&state.diagram, &painter, response.rect);
            }

            // Draw cursor preview for add modes
            if let Some(pos) = self.cursor_pos {
                self.render_cursor_preview(&painter, pos);
            }

            // Handle mouse clicks with custom double-click detection (500ms window)
            // Double-click in add mode will add element AND switch back to arrow mode
            let ctrl_held = ui.input(|i| i.modifiers.ctrl);
            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let now = Instant::now();

                    // Check if this is a double-click (within time and distance threshold)
                    let is_double_click = if let (Some(last_time), Some(last_pos)) = (self.last_click_time, self.last_click_pos) {
                        let time_diff = now.duration_since(last_time).as_millis();
                        let distance = ((pos.x - last_pos.x).powi(2) + (pos.y - last_pos.y).powi(2)).sqrt();
                        time_diff <= DOUBLE_CLICK_TIME_MS && distance <= DOUBLE_CLICK_DISTANCE
                    } else {
                        false
                    };

                    if is_double_click {
                        // Double-click: add element and switch to arrow mode
                        // Clear the last click to prevent triple-click being detected as another double
                        self.last_click_time = None;
                        self.last_click_pos = None;
                        self.handle_canvas_click(pos, true, ctrl_held);
                    } else {
                        // Single click: record time/pos for potential double-click detection
                        self.last_click_time = Some(now);
                        self.last_click_pos = Some(pos);
                        self.handle_canvas_click(pos, false, ctrl_held);
                    }
                }
            }

            // Handle drag start - determine if we're resizing, dragging nodes, or marquee selecting
            if response.drag_started() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let point = Point::new(pos.x, pos.y);
                    let corner_margin = 10.0; // Size of corner hit area

                    // First, check if we clicked on a corner of a selected resizable node
                    let mut corner_info: Option<(NodeId, Corner)> = None;
                    if let Some(state) = self.current_diagram() {
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
                    }

                    if let Some((node_id, corner)) = corner_info {
                        // We're starting a resize operation
                        if self.edit_mode != EditMode::Arrow {
                            self.set_edit_mode(EditMode::Arrow);
                        }
                        if let Some(state) = self.current_diagram_mut() {
                            state.diagram.push_undo();
                        }
                        self.resize_state.start(node_id, corner);
                        self.dragging_nodes = false;
                        self.selection_rect.clear();
                        self.status_message = "Resizing...".to_string();
                    } else {
                        // Check if we clicked on any element (for dragging)
                        let clicked_element_id = self.current_diagram()
                            .and_then(|state| state.diagram.find_element_at(point));

                        if let Some(element_id) = clicked_element_id {
                            // Dragging on an element - switch to Arrow mode and start dragging
                            if self.edit_mode != EditMode::Arrow {
                                self.set_edit_mode(EditMode::Arrow);
                            }

                            // Select the element if not already selected
                            if let Some(state) = self.current_diagram_mut() {
                                let already_selected = state.diagram.selected_elements_in_order().contains(&element_id);
                                if !already_selected {
                                    // Select this element (this allows click-and-drag in one motion)
                                    state.diagram.select_element(element_id);
                                }
                                // Push undo before we start moving
                                state.diagram.push_undo();
                            }
                            self.dragging_nodes = true;
                            self.selection_rect.clear();
                        } else if self.edit_mode == EditMode::Arrow {
                            // We're starting a marquee selection (only in Arrow mode)
                            self.dragging_nodes = false;
                            self.selection_rect.start = Some(pos);
                            self.selection_rect.current = Some(pos);
                            // Clear current selection when starting a new marquee
                            if let Some(state) = self.current_diagram_mut() {
                                state.diagram.clear_selection();
                            }
                        } else if self.edit_mode == EditMode::Lasso {
                            // We're starting a lasso selection
                            self.dragging_nodes = false;
                            self.lasso_points.clear();
                            self.lasso_points.push(pos);
                            // Clear current selection when starting a new lasso
                            if let Some(state) = self.current_diagram_mut() {
                                state.diagram.clear_selection();
                            }
                        }
                    }
                }
            }

            // Handle dragging
            if response.dragged() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let delta = response.drag_delta();

                    if self.resize_state.is_active() {
                        // Handle resize
                        let node_id = self.resize_state.node_id.unwrap();
                        let corner = self.resize_state.corner;
                        let min_width = 40.0;
                        let min_height = 30.0;

                        if let Some(state) = self.current_diagram_mut() {
                            if let Some(node) = state.diagram.find_node_mut(node_id) {
                                node.resize_from_corner(corner, delta.x, delta.y, min_width, min_height);
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
                                    for id in selected {
                                        state.diagram.translate_element(id, delta.x, delta.y);
                                    }
                                    state.diagram.recalculate_connections();
                                    state.modified = true;
                                }
                            }
                        } else {
                            // Update marquee selection rectangle
                            self.selection_rect.current = Some(pos);
                        }
                    } else if self.edit_mode == EditMode::Lasso {
                        // Add points to the lasso path (with some distance threshold to avoid too many points)
                        if let Some(last) = self.lasso_points.last() {
                            let dist = ((pos.x - last.x).powi(2) + (pos.y - last.y).powi(2)).sqrt();
                            if dist > 3.0 {
                                self.lasso_points.push(pos);
                            }
                        }
                    }
                }
            }

            // Draw selection rectangle if active
            if self.selection_rect.is_active() {
                if let Some(rect) = self.selection_rect.to_egui_rect() {
                    // Draw selection rectangle with semi-transparent fill
                    painter.rect(
                        rect,
                        egui::Rounding::ZERO,
                        egui::Color32::from_rgba_unmultiplied(100, 150, 255, 50),
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 150, 255)),
                    );
                }
            }

            // Draw lasso path if active
            if self.lasso_points.len() > 1 {
                // Draw the lasso line
                painter.add(egui::Shape::line(
                    self.lasso_points.clone(),
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 150, 255)),
                ));

                // Draw closing line (dashed effect using dotted line to start point)
                if self.lasso_points.len() > 2 {
                    if let (Some(first), Some(last)) = (self.lasso_points.first(), self.lasso_points.last()) {
                        painter.line_segment(
                            [*last, *first],
                            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(100, 150, 255, 128)),
                        );
                    }
                }
            }

            // Handle drag end
            if response.drag_stopped() {
                if self.resize_state.is_active() {
                    // Finished resizing
                    self.resize_state.clear();
                    self.status_message = "Ready".to_string();
                } else if self.edit_mode == EditMode::Arrow {
                    if self.dragging_nodes {
                        // Undo was already pushed at drag start, nothing to do here
                    } else {
                        // Complete marquee selection
                        if let Some(rect) = self.selection_rect.to_core_rect() {
                            if let Some(state) = self.current_diagram_mut() {
                                state.diagram.select_nodes_in_rect(&rect);
                                let count = state.diagram.selected_nodes().len();
                                if count > 0 {
                                    self.status_message = format!("Selected {} node(s)", count);
                                } else {
                                    self.status_message = "Ready".to_string();
                                }
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
            }
        });
    }
}
