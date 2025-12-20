//! Main application state and update loop

use eframe::egui;
use jmt_core::{Diagram, EditMode, NodeType};
use jmt_core::geometry::{Point, Rect};

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
        let name = format!("Diagram {}", self.diagrams.len() + 1);
        let diagram = Diagram::new(&name);
        self.diagrams.push(DiagramState::new(diagram));
        self.active_diagram = self.diagrams.len() - 1;
        self.status_message = format!("Created new diagram: {}", name);
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
        self.edit_mode = mode;
        self.pending_connection_source = None;
        self.status_message = format!("Mode: {}", mode.display_name());
    }

    /// Handle canvas click
    fn handle_canvas_click(&mut self, pos: egui::Pos2) {
        let point = Point::new(pos.x, pos.y);
        let edit_mode = self.edit_mode;
        let pending_source = self.pending_connection_source;

        let Some(state) = self.current_diagram_mut() else {
            return;
        };

        match edit_mode {
            EditMode::Arrow => {
                // Try to select a node or connection
                if let Some(node_id) = state.diagram.find_node_at(point) {
                    let name = state.diagram.find_node(node_id)
                        .map(|n| n.name().to_string())
                        .unwrap_or_default();
                    state.diagram.select_node(node_id);
                    self.status_message = format!("Selected: {}", name);
                } else if let Some(conn_id) = state.diagram.find_connection_at(point, 10.0) {
                    state.diagram.select_connection(conn_id);
                    self.status_message = "Selected connection".to_string();
                } else {
                    state.diagram.clear_selection();
                    self.status_message = "Ready".to_string();
                }
            }
            EditMode::AddState => {
                state.diagram.push_undo();
                let id = state.diagram.add_node(NodeType::State, pos.x, pos.y);
                state.diagram.select_node(id);
                state.modified = true;
                self.status_message = "Added state".to_string();
            }
            EditMode::AddInitial => {
                state.diagram.push_undo();
                let id = state.diagram.add_node(NodeType::Initial, pos.x, pos.y);
                state.diagram.select_node(id);
                state.modified = true;
                self.status_message = "Added initial pseudo-state".to_string();
            }
            EditMode::AddFinal => {
                state.diagram.push_undo();
                let id = state.diagram.add_node(NodeType::Final, pos.x, pos.y);
                state.diagram.select_node(id);
                state.modified = true;
                self.status_message = "Added final pseudo-state".to_string();
            }
            EditMode::AddChoice => {
                state.diagram.push_undo();
                let id = state.diagram.add_node(NodeType::Choice, pos.x, pos.y);
                state.diagram.select_node(id);
                state.modified = true;
                self.status_message = "Added choice pseudo-state".to_string();
            }
            EditMode::AddJunction => {
                state.diagram.push_undo();
                let id = state.diagram.add_node(NodeType::Junction, pos.x, pos.y);
                state.diagram.select_node(id);
                state.modified = true;
                self.status_message = "Added junction pseudo-state".to_string();
            }
            EditMode::AddFork => {
                state.diagram.push_undo();
                let id = state.diagram.add_node(NodeType::Fork, pos.x, pos.y);
                state.diagram.select_node(id);
                state.modified = true;
                self.status_message = "Added fork pseudo-state".to_string();
            }
            EditMode::AddJoin => {
                state.diagram.push_undo();
                let id = state.diagram.add_node(NodeType::Join, pos.x, pos.y);
                state.diagram.select_node(id);
                state.modified = true;
                self.status_message = "Added join pseudo-state".to_string();
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
                    self.pending_connection_source = None;
                    self.status_message = "Click a node to start connection".to_string();
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
                    let label = if diagram_state.modified {
                        format!("{}*", name)
                    } else {
                        name.clone()
                    };

                    if ui.selectable_label(i == self.active_diagram, &label).clicked() {
                        self.active_diagram = i;
                    }
                }

                if ui.button("+").clicked() {
                    self.new_diagram();
                }
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

            // Handle mouse clicks
            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    self.handle_canvas_click(pos);
                }
            }

            // Handle drag start - determine if we're dragging nodes or marquee selecting
            if response.drag_started() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let point = Point::new(pos.x, pos.y);

                    // Check if we clicked on a node
                    let clicked_node_id = self.current_diagram()
                        .and_then(|state| state.diagram.find_node_at(point));

                    if let Some(node_id) = clicked_node_id {
                        // Dragging on a node - switch to Arrow mode and start dragging
                        if self.edit_mode != EditMode::Arrow {
                            self.set_edit_mode(EditMode::Arrow);
                        }

                        // Select the node if not already selected
                        if let Some(state) = self.current_diagram_mut() {
                            let already_selected = state.diagram.selected_nodes().contains(&node_id);
                            if !already_selected {
                                // Select this node (this allows click-and-drag in one motion)
                                state.diagram.select_node(node_id);
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
                    }
                }
            }

            // Handle dragging
            if response.dragged() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if self.edit_mode == EditMode::Arrow {
                        if self.dragging_nodes {
                            // Move selected nodes
                            if let Some(state) = self.current_diagram_mut() {
                                let selected = state.diagram.selected_nodes();
                                if !selected.is_empty() {
                                    let delta = response.drag_delta();
                                    for id in selected {
                                        if let Some(node) = state.diagram.find_node_mut(id) {
                                            node.translate(delta.x, delta.y);
                                        }
                                    }
                                    state.diagram.recalculate_connections();
                                    state.modified = true;
                                }
                            }
                        } else {
                            // Update marquee selection rectangle
                            self.selection_rect.current = Some(pos);
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

            // Handle drag end
            if response.drag_stopped() {
                if self.edit_mode == EditMode::Arrow {
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
                }

                // Clear selection rect and reset state
                self.selection_rect.clear();
                self.dragging_nodes = false;
            }
        });
    }
}
