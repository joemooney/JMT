//! Diagram rendering using egui Painter

use eframe::egui::{self, Color32, Pos2, Rect, Rounding, Stroke, Vec2};
use jmt_core::{Diagram, Node, Connection, DiagramType};
use jmt_core::node::{PseudoStateKind, Side};
use jmt_core::geometry::Color;
use jmt_core::sequence::{Lifeline, Message, CombinedFragment};
use jmt_core::usecase::{Actor, UseCase, SystemBoundary, UseCaseRelationship, RelationshipKind, UseCaseElementKind};
use jmt_core::activity::{Action, ActionKind, Swimlane, ControlFlow};

/// Canvas for rendering diagrams
pub struct DiagramCanvas {
    /// Offset for screen coordinates (top-left of the canvas in screen space)
    offset: Pos2,
}

impl DiagramCanvas {
    pub fn new() -> Self {
        Self {
            offset: Pos2::ZERO,
        }
    }

    /// Render the diagram to the canvas with optional zoom
    pub fn render(&mut self, diagram: &Diagram, painter: &egui::Painter, rect: Rect) {
        self.render_with_zoom(diagram, painter, rect, 1.0);
    }

    /// Render the diagram to the canvas with zoom level
    pub fn render_with_zoom(&mut self, diagram: &Diagram, painter: &egui::Painter, rect: Rect, zoom: f32) {
        // Store offset for use by scale_pos and scale_rect
        self.offset = rect.min;

        match diagram.diagram_type {
            DiagramType::StateMachine => self.render_state_machine(diagram, painter, zoom),
            DiagramType::Sequence => self.render_sequence_diagram(diagram, painter, zoom),
            DiagramType::UseCase => self.render_use_case_diagram(diagram, painter, zoom),
            DiagramType::Activity => self.render_activity_diagram(diagram, painter, zoom),
        }
    }

    /// Scale a position by zoom factor and add screen offset
    #[inline]
    fn scale_pos(&self, x: f32, y: f32, zoom: f32) -> Pos2 {
        Pos2::new(x * zoom + self.offset.x, y * zoom + self.offset.y)
    }

    /// Scale a rect by zoom factor and add screen offset
    #[inline]
    fn scale_rect(&self, rect: &jmt_core::geometry::Rect, zoom: f32) -> Rect {
        Rect::from_min_max(
            Pos2::new(rect.x1 * zoom + self.offset.x, rect.y1 * zoom + self.offset.y),
            Pos2::new(rect.x2 * zoom + self.offset.x, rect.y2 * zoom + self.offset.y),
        )
    }

    /// Render a state machine diagram
    fn render_state_machine(&self, diagram: &Diagram, painter: &egui::Painter, zoom: f32) {
        // Render all nodes
        for node in diagram.nodes() {
            self.render_node(node, painter, &diagram.settings, zoom);
        }

        // Render all connections
        for conn in diagram.connections() {
            self.render_connection(conn, painter, &diagram.settings, zoom);
        }
    }

    /// Render a sequence diagram
    fn render_sequence_diagram(&self, diagram: &Diagram, painter: &egui::Painter, zoom: f32) {
        // Render combined fragments first (background)
        for fragment in &diagram.fragments {
            self.render_combined_fragment(fragment, painter, zoom);
        }

        // Render lifelines
        for lifeline in &diagram.lifelines {
            self.render_lifeline(lifeline, painter, zoom);
        }

        // Render messages
        for message in &diagram.messages {
            self.render_message(message, diagram, painter, zoom);
        }
    }

    /// Render a use case diagram
    fn render_use_case_diagram(&self, diagram: &Diagram, painter: &egui::Painter, zoom: f32) {
        // Render system boundaries first (background)
        for boundary in &diagram.system_boundaries {
            self.render_system_boundary(boundary, painter, zoom);
        }

        // Render use cases
        for use_case in &diagram.use_cases {
            self.render_use_case(use_case, painter, zoom);
        }

        // Render actors
        for actor in &diagram.actors {
            self.render_actor(actor, painter, zoom);
        }

        // Render relationships
        for rel in &diagram.uc_relationships {
            self.render_uc_relationship(rel, diagram, painter, zoom);
        }
    }

    /// Render an activity diagram
    fn render_activity_diagram(&self, diagram: &Diagram, painter: &egui::Painter, zoom: f32) {
        // Render swimlanes first (background)
        for swimlane in &diagram.swimlanes {
            self.render_swimlane(swimlane, painter, zoom);
        }

        // Render actions
        for action in &diagram.actions {
            self.render_action(action, painter, zoom);
        }

        // Render pseudo-states (initial, final, decision, fork, join)
        for node in diagram.nodes() {
            self.render_node(node, painter, &diagram.settings, zoom);
        }

        // Render control flows
        for flow in &diagram.control_flows {
            self.render_control_flow(flow, diagram, painter, zoom);
        }

        // Also render state machine connections for activity diagrams
        for conn in diagram.connections() {
            self.render_connection(conn, painter, &diagram.settings, zoom);
        }
    }

    /// Render a node
    fn render_node(&self, node: &Node, painter: &egui::Painter, settings: &jmt_core::DiagramSettings, zoom: f32) {
        match node {
            Node::State(state) => self.render_state(state, painter, settings, zoom),
            Node::Pseudo(pseudo) => self.render_pseudo_state(pseudo, painter, settings, zoom),
        }
    }

    /// Render a state node
    fn render_state(&self, state: &jmt_core::node::State, painter: &egui::Painter, settings: &jmt_core::DiagramSettings, zoom: f32) {
        let bounds = &state.bounds;
        let rect = self.scale_rect(bounds, zoom);

        // Fill color
        let fill_color = state.fill_color.unwrap_or(settings.state_color);
        let fill = color_to_egui(fill_color);

        // Draw rounded rectangle
        let rounding = Rounding::same(settings.corner_rounding * zoom);
        painter.rect(rect, rounding, fill, Stroke::new(zoom, Color32::BLACK));

        // Draw activities if they should be shown
        let show_activities = state.should_show_activities(settings.show_activities);
        let header_height = 24.0 * zoom;

        // Check if state has children (multiple regions OR any region with children)
        let has_children = state.regions.len() > 1 ||
            state.regions.iter().any(|r| !r.children.is_empty());

        // Draw state name - at top when has children, activities, or multiple regions
        let text_pos = if has_children {
            // State with children: name at top in header (25px from top)
            let composite_header = 25.0 * zoom;
            Pos2::new(rect.center().x, rect.min.y + composite_header / 2.0)
        } else if show_activities {
            // Center vertically between top and separator line
            Pos2::new(rect.center().x, rect.min.y + header_height / 2.0)
        } else {
            // Center in the state box when no children/activities
            Pos2::new(rect.center().x, rect.center().y)
        };

        painter.text(
            text_pos,
            egui::Align2::CENTER_CENTER,
            &state.name,
            egui::FontId::proportional(12.0 * zoom),
            Color32::BLACK,
        );

        if show_activities {
            // Draw separator line
            let line_y = rect.min.y + header_height;
            painter.line_segment(
                [Pos2::new(rect.min.x, line_y), Pos2::new(rect.max.x, line_y)],
                Stroke::new(zoom, Color32::BLACK),
            );

            // Draw activities text (supporting multi-line)
            let line_height = 12.0 * zoom;
            let mut y = line_y + 4.0 * zoom;

            if !state.entry_activity.is_empty() {
                let text = format!("entry / {}", state.entry_activity);
                for line in text.lines() {
                    painter.text(
                        Pos2::new(rect.min.x + 4.0 * zoom, y),
                        egui::Align2::LEFT_TOP,
                        line,
                        egui::FontId::proportional(10.0 * zoom),
                        Color32::BLACK,
                    );
                    y += line_height;
                }
            }
            if !state.exit_activity.is_empty() {
                let text = format!("exit / {}", state.exit_activity);
                for line in text.lines() {
                    painter.text(
                        Pos2::new(rect.min.x + 4.0 * zoom, y),
                        egui::Align2::LEFT_TOP,
                        line,
                        egui::FontId::proportional(10.0 * zoom),
                        Color32::BLACK,
                    );
                    y += line_height;
                }
            }
            if !state.do_activity.is_empty() {
                let text = format!("do / {}", state.do_activity);
                for line in text.lines() {
                    painter.text(
                        Pos2::new(rect.min.x + 4.0 * zoom, y),
                        egui::Align2::LEFT_TOP,
                        line,
                        egui::FontId::proportional(10.0 * zoom),
                        Color32::BLACK,
                    );
                    y += line_height;
                }
            }
        }

        // Draw region separators (dashed lines between regions)
        if state.regions.len() > 1 {
            for (i, region) in state.regions.iter().enumerate() {
                // Skip the first region (no separator above it)
                if i == 0 {
                    continue;
                }

                // Draw dashed line at top of each region (except first)
                let region_rect = self.scale_rect(&region.bounds, zoom);
                let y = region_rect.min.y;

                // Use orange color if region is selected, black otherwise
                let color = if region.has_focus {
                    Color32::from_rgb(255, 165, 0) // Orange
                } else {
                    Color32::BLACK
                };

                self.draw_dashed_line(
                    painter,
                    Pos2::new(rect.min.x, y),
                    Pos2::new(rect.max.x, y),
                    Stroke::new(zoom, color),
                    4.0 * zoom,  // dash length
                    2.0 * zoom,  // gap length
                );
            }
        }

        // Draw focus corners if selected
        if state.has_focus {
            self.render_focus_corners(rect, painter, settings.corner_size * zoom);
        }
    }

    /// Render a pseudo-state node
    fn render_pseudo_state(&self, pseudo: &jmt_core::node::PseudoState, painter: &egui::Painter, settings: &jmt_core::DiagramSettings, zoom: f32) {
        let bounds = &pseudo.bounds;
        let center = self.scale_pos(bounds.center().x, bounds.center().y, zoom);

        match pseudo.kind {
            PseudoStateKind::Initial => {
                // Filled black circle
                let radius = (bounds.width().min(bounds.height()) / 2.0 - 2.0) * zoom;
                painter.circle_filled(center, radius, Color32::BLACK);
            }
            PseudoStateKind::Final => {
                // Double circle (outer ring + inner filled)
                let outer_radius = (bounds.width().min(bounds.height()) / 2.0 - 2.0) * zoom;
                let inner_radius = outer_radius - 4.0 * zoom;
                painter.circle_stroke(center, outer_radius, Stroke::new(zoom, Color32::BLACK));
                painter.circle_filled(center, inner_radius, Color32::BLACK);
            }
            PseudoStateKind::Choice | PseudoStateKind::Junction => {
                // Diamond shape
                let half_w = bounds.width() / 2.0 * zoom;
                let half_h = bounds.height() / 2.0 * zoom;
                let points = vec![
                    Pos2::new(center.x, center.y - half_h),  // top
                    Pos2::new(center.x + half_w, center.y),  // right
                    Pos2::new(center.x, center.y + half_h),  // bottom
                    Pos2::new(center.x - half_w, center.y),  // left
                    Pos2::new(center.x, center.y - half_h),  // close
                ];
                painter.add(egui::Shape::line(points, Stroke::new(zoom, Color32::BLACK)));
            }
            PseudoStateKind::Fork | PseudoStateKind::Join => {
                // Thick horizontal or vertical bar
                let rect = self.scale_rect(bounds, zoom);
                painter.rect_filled(rect, Rounding::ZERO, Color32::BLACK);
            }
        }

        // Draw focus corners if selected
        if pseudo.has_focus {
            let rect = self.scale_rect(bounds, zoom);
            self.render_focus_corners(rect, painter, settings.pseudo_corner_size * zoom);
        }
    }

    /// Render focus corners for selected nodes
    fn render_focus_corners(&self, rect: Rect, painter: &egui::Painter, corner_size: f32) {
        let color = Color32::BLACK;

        // Top-left
        painter.rect_filled(
            Rect::from_min_size(rect.min, Vec2::splat(corner_size)),
            Rounding::ZERO,
            color,
        );

        // Top-right
        painter.rect_filled(
            Rect::from_min_size(
                Pos2::new(rect.max.x - corner_size, rect.min.y),
                Vec2::splat(corner_size),
            ),
            Rounding::ZERO,
            color,
        );

        // Bottom-left
        painter.rect_filled(
            Rect::from_min_size(
                Pos2::new(rect.min.x, rect.max.y - corner_size),
                Vec2::splat(corner_size),
            ),
            Rounding::ZERO,
            color,
        );

        // Bottom-right
        painter.rect_filled(
            Rect::from_min_size(
                Pos2::new(rect.max.x - corner_size, rect.max.y - corner_size),
                Vec2::splat(corner_size),
            ),
            Rounding::ZERO,
            color,
        );
    }

    /// Render a connection
    fn render_connection(&self, conn: &Connection, painter: &egui::Painter, settings: &jmt_core::DiagramSettings, zoom: f32) {
        let stroke = if conn.selected {
            Stroke::new(2.0 * zoom, Color32::from_rgb(255, 165, 0))  // Orange
        } else {
            Stroke::new(zoom, Color32::BLACK)
        };

        // Draw line segments
        for segment in &conn.segments {
            painter.line_segment(
                [
                    self.scale_pos(segment.start.x, segment.start.y, zoom),
                    self.scale_pos(segment.end.x, segment.end.y, zoom),
                ],
                stroke,
            );
        }

        // Draw arrowhead at target
        if let Some(end_point) = conn.end_point() {
            self.render_arrowhead(
                self.scale_pos(end_point.x, end_point.y, zoom),
                conn.target_side,
                painter,
                settings,
                stroke,
                zoom,
            );
        }

        // Draw label if present
        let label = conn.label();
        if !label.is_empty() {
            if let Some((label_pos, midpoint)) = conn.label_position() {
                let label_screen_pos = self.scale_pos(label_pos.x, label_pos.y, zoom);
                let midpoint_screen_pos = self.scale_pos(midpoint.x, midpoint.y, zoom);

                // Draw leader line if enabled and label has been moved from default
                if settings.show_leader_lines && conn.label_offset.is_some() {
                    // Draw dotted line from label to connection midpoint
                    self.draw_dashed_line(
                        painter,
                        label_screen_pos,
                        midpoint_screen_pos,
                        Stroke::new(zoom * 0.5, Color32::GRAY),
                        4.0 * zoom,  // dash length
                        2.0 * zoom,  // gap length
                    );
                }

                // Determine label color based on selection state
                let label_color = if conn.label_selected || conn.selected {
                    Color32::from_rgb(255, 165, 0)  // Orange when selected
                } else {
                    Color32::BLACK
                };

                painter.text(
                    label_screen_pos,
                    egui::Align2::CENTER_BOTTOM,
                    &label,
                    egui::FontId::proportional(10.0 * zoom),
                    label_color,
                );
            }
        }
    }

    /// Draw a dashed line between two points
    fn draw_dashed_line(
        &self,
        painter: &egui::Painter,
        start: Pos2,
        end: Pos2,
        stroke: Stroke,
        dash_length: f32,
        gap_length: f32,
    ) {
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let total_length = (dx * dx + dy * dy).sqrt();

        if total_length < 0.001 {
            return;
        }

        let unit_x = dx / total_length;
        let unit_y = dy / total_length;

        let mut current_pos = 0.0;
        let mut drawing = true;

        while current_pos < total_length {
            let segment_length = if drawing { dash_length } else { gap_length };
            let end_pos = (current_pos + segment_length).min(total_length);

            if drawing {
                let p1 = Pos2::new(
                    start.x + current_pos * unit_x,
                    start.y + current_pos * unit_y,
                );
                let p2 = Pos2::new(
                    start.x + end_pos * unit_x,
                    start.y + end_pos * unit_y,
                );
                painter.line_segment([p1, p2], stroke);
            }

            current_pos = end_pos;
            drawing = !drawing;
        }
    }

    /// Render an arrowhead
    fn render_arrowhead(
        &self,
        point: Pos2,
        side: Side,
        painter: &egui::Painter,
        settings: &jmt_core::DiagramSettings,
        stroke: Stroke,
        zoom: f32,
    ) {
        let w = settings.arrow_width * zoom;
        let h = settings.arrow_height * zoom;

        let (p1, p2) = match side {
            Side::Top => (
                Pos2::new(point.x - w, point.y - h),
                Pos2::new(point.x + w, point.y - h),
            ),
            Side::Bottom => (
                Pos2::new(point.x - w, point.y + h),
                Pos2::new(point.x + w, point.y + h),
            ),
            Side::Left => (
                Pos2::new(point.x - h, point.y - w),
                Pos2::new(point.x - h, point.y + w),
            ),
            Side::Right => (
                Pos2::new(point.x + h, point.y - w),
                Pos2::new(point.x + h, point.y + w),
            ),
            Side::None => return,
        };

        painter.line_segment([p1, point], stroke);
        painter.line_segment([p2, point], stroke);
    }

    // ===== Sequence Diagram Rendering =====

    /// Render a lifeline
    fn render_lifeline(&self, lifeline: &Lifeline, painter: &egui::Painter, zoom: f32) {
        let head_bounds = lifeline.head_bounds();
        let head_rect = self.scale_rect(&head_bounds, zoom);

        // Check if this is an actor lifeline
        if lifeline.stereotype.as_deref() == Some("actor") {
            // Draw stick figure
            self.render_stick_figure(self.scale_pos(lifeline.x, lifeline.y, zoom), painter, zoom);
        } else {
            // Draw head box
            let fill = lifeline.fill_color.map(color_to_egui).unwrap_or(Color32::from_rgb(255, 255, 220));
            painter.rect(head_rect, Rounding::same(2.0 * zoom), fill, Stroke::new(zoom, Color32::BLACK));
        }

        // Draw name
        let name_pos = self.scale_pos(lifeline.x, lifeline.y + lifeline.head_height / 2.0, zoom);
        painter.text(
            name_pos,
            egui::Align2::CENTER_CENTER,
            &lifeline.name,
            egui::FontId::proportional(11.0 * zoom),
            Color32::BLACK,
        );

        // Draw dashed lifeline
        let line_start_y = (lifeline.y + lifeline.head_height) * zoom;
        let line_end_y = lifeline.destruction_y.unwrap_or(lifeline.y + lifeline.head_height + lifeline.line_length) * zoom;

        // Draw dashed line (approximate with short segments)
        let dash_len = 6.0 * zoom;
        let gap_len = 4.0 * zoom;
        let mut y = line_start_y;
        while y < line_end_y {
            let end_y = (y + dash_len).min(line_end_y);
            painter.line_segment(
                [Pos2::new(lifeline.x * zoom, y), Pos2::new(lifeline.x * zoom, end_y)],
                Stroke::new(zoom, Color32::BLACK),
            );
            y += dash_len + gap_len;
        }

        // Draw destruction X if destroyed
        if lifeline.is_destroyed {
            if let Some(dy) = lifeline.destruction_y {
                let size = 8.0 * zoom;
                let dy_z = dy * zoom;
                let x_z = lifeline.x * zoom;
                painter.line_segment(
                    [Pos2::new(x_z - size, dy_z - size), Pos2::new(x_z + size, dy_z + size)],
                    Stroke::new(2.0 * zoom, Color32::BLACK),
                );
                painter.line_segment(
                    [Pos2::new(x_z - size, dy_z + size), Pos2::new(x_z + size, dy_z - size)],
                    Stroke::new(2.0 * zoom, Color32::BLACK),
                );
            }
        }

        // Draw selection indicator
        if lifeline.has_focus {
            let full_rect = Rect::from_min_max(
                Pos2::new((head_bounds.x1 - 2.0) * zoom, (head_bounds.y1 - 2.0) * zoom),
                Pos2::new((head_bounds.x2 + 2.0) * zoom, (head_bounds.y2 + 2.0) * zoom),
            );
            painter.rect_stroke(full_rect, Rounding::same(2.0 * zoom), Stroke::new(2.0 * zoom, Color32::from_rgb(0, 120, 215)));
        }
    }

    /// Render a message
    fn render_message(&self, message: &Message, diagram: &Diagram, painter: &egui::Painter, zoom: f32) {
        let source_x = message.source_id
            .and_then(|id| diagram.find_lifeline(id))
            .map(|l| l.x)
            .unwrap_or(50.0) * zoom;

        let target_x = message.target_id
            .and_then(|id| diagram.find_lifeline(id))
            .map(|l| l.x)
            .unwrap_or(150.0) * zoom;

        let msg_y = message.y * zoom;

        let stroke = if message.selected {
            Stroke::new(2.0 * zoom, Color32::from_rgb(255, 165, 0))
        } else {
            Stroke::new(zoom, Color32::BLACK)
        };

        // Draw the message line
        if message.kind.is_dashed() {
            // Dashed line for return messages
            let dash_len: f32 = 6.0 * zoom;
            let gap_len: f32 = 4.0 * zoom;
            let total_len = (target_x - source_x).abs();
            let direction = if target_x > source_x { 1.0 } else { -1.0 };
            let mut x = source_x;
            let mut remaining = total_len;
            while remaining > 0.0 {
                let seg_len = dash_len.min(remaining);
                painter.line_segment(
                    [Pos2::new(x, msg_y), Pos2::new(x + direction * seg_len, msg_y)],
                    stroke,
                );
                x += direction * (seg_len + gap_len);
                remaining -= dash_len + gap_len;
            }
        } else {
            painter.line_segment(
                [Pos2::new(source_x, msg_y), Pos2::new(target_x, msg_y)],
                stroke,
            );
        }

        // Draw arrowhead
        let arrow_size = 8.0 * zoom;
        if message.kind.has_filled_arrow() {
            // Filled triangle arrowhead
            let direction = if target_x > source_x { -1.0 } else { 1.0 };
            let tip = Pos2::new(target_x, msg_y);
            let left = Pos2::new(target_x + direction * arrow_size, msg_y - arrow_size / 2.0);
            let right = Pos2::new(target_x + direction * arrow_size, msg_y + arrow_size / 2.0);
            painter.add(egui::Shape::convex_polygon(
                vec![tip, left, right],
                Color32::BLACK,
                Stroke::NONE,
            ));
        } else {
            // Open arrowhead
            let direction = if target_x > source_x { -1.0 } else { 1.0 };
            let tip = Pos2::new(target_x, msg_y);
            painter.line_segment(
                [tip, Pos2::new(target_x + direction * arrow_size, msg_y - arrow_size / 2.0)],
                stroke,
            );
            painter.line_segment(
                [tip, Pos2::new(target_x + direction * arrow_size, msg_y + arrow_size / 2.0)],
                stroke,
            );
        }

        // Draw label
        let label = message.full_label();
        if !label.is_empty() {
            let mid_x = (source_x + target_x) / 2.0;
            painter.text(
                Pos2::new(mid_x, msg_y - 10.0 * zoom),
                egui::Align2::CENTER_BOTTOM,
                &label,
                egui::FontId::proportional(10.0 * zoom),
                Color32::BLACK,
            );
        }
    }

    /// Render a combined fragment
    fn render_combined_fragment(&self, fragment: &CombinedFragment, painter: &egui::Painter, zoom: f32) {
        let bounds = &fragment.bounds;
        let rect = self.scale_rect(bounds, zoom);

        // Draw frame
        let fill = Color32::from_rgba_unmultiplied(240, 240, 255, 100);
        painter.rect(rect, Rounding::ZERO, fill, Stroke::new(zoom, Color32::BLACK));

        // Draw keyword pentagon
        let keyword = fragment.kind.display_name();
        let kw_width = (keyword.len() as f32 * 7.0 + 10.0) * zoom;
        let kw_height = 18.0 * zoom;
        let x1 = bounds.x1 * zoom;
        let y1 = bounds.y1 * zoom;
        let pentagon = vec![
            Pos2::new(x1, y1),
            Pos2::new(x1 + kw_width, y1),
            Pos2::new(x1 + kw_width + 8.0 * zoom, y1 + kw_height / 2.0),
            Pos2::new(x1 + kw_width, y1 + kw_height),
            Pos2::new(x1, y1 + kw_height),
        ];
        painter.add(egui::Shape::convex_polygon(
            pentagon,
            Color32::WHITE,
            Stroke::new(zoom, Color32::BLACK),
        ));
        painter.text(
            Pos2::new(x1 + 5.0 * zoom, y1 + kw_height / 2.0),
            egui::Align2::LEFT_CENTER,
            keyword,
            egui::FontId::proportional(10.0 * zoom),
            Color32::BLACK,
        );

        // Draw operand separators
        for operand in &fragment.operands {
            if operand.start_y > bounds.y1 + 20.0 {
                // Dashed separator line
                let sep_y = operand.start_y * zoom;
                let dash_len = 6.0 * zoom;
                let gap_len = 4.0 * zoom;
                let mut x = bounds.x1 * zoom;
                let x2 = bounds.x2 * zoom;
                while x < x2 {
                    let end_x = (x + dash_len).min(x2);
                    painter.line_segment(
                        [Pos2::new(x, sep_y), Pos2::new(end_x, sep_y)],
                        Stroke::new(zoom, Color32::DARK_GRAY),
                    );
                    x += dash_len + gap_len;
                }
            }
            // Draw guard
            if let Some(ref guard) = operand.guard {
                painter.text(
                    Pos2::new(bounds.x1 * zoom + 5.0 * zoom, operand.start_y * zoom + 12.0 * zoom),
                    egui::Align2::LEFT_CENTER,
                    guard,
                    egui::FontId::proportional(9.0 * zoom),
                    Color32::DARK_GRAY,
                );
            }
        }
    }

    // ===== Use Case Diagram Rendering =====

    /// Render an actor (stick figure)
    fn render_actor(&self, actor: &Actor, painter: &egui::Painter, zoom: f32) {
        if actor.use_stick_figure {
            self.render_stick_figure(self.scale_pos(actor.x, actor.y, zoom), painter, zoom);
        } else {
            // Rectangle representation for system actors
            let bounds = actor.bounds();
            let rect = self.scale_rect(&bounds, zoom);
            painter.rect(rect, Rounding::same(4.0 * zoom), Color32::from_rgb(230, 230, 250), Stroke::new(zoom, Color32::BLACK));
        }

        // Draw name below
        let name_y = (actor.y + actor.height - 10.0) * zoom;
        painter.text(
            Pos2::new(actor.x * zoom, name_y),
            egui::Align2::CENTER_TOP,
            &actor.name,
            egui::FontId::proportional(11.0 * zoom),
            Color32::BLACK,
        );

        // Draw selection indicator
        if actor.has_focus {
            let bounds = actor.bounds();
            let rect = Rect::from_min_max(
                Pos2::new((bounds.x1 - 2.0) * zoom, (bounds.y1 - 2.0) * zoom),
                Pos2::new((bounds.x2 + 2.0) * zoom, (bounds.y2 + 2.0) * zoom),
            );
            painter.rect_stroke(rect, Rounding::ZERO, Stroke::new(2.0 * zoom, Color32::from_rgb(0, 120, 215)));
        }
    }

    /// Render a stick figure for actors
    fn render_stick_figure(&self, pos: Pos2, painter: &egui::Painter, zoom: f32) {
        let head_radius = 8.0 * zoom;
        let head_y = pos.y + head_radius;
        let body_top = head_y + head_radius;
        let body_bottom = body_top + 20.0 * zoom;
        let arm_y = body_top + 8.0 * zoom;
        let leg_spread = 12.0 * zoom;
        let leg_bottom = body_bottom + 18.0 * zoom;

        let stroke = Stroke::new(1.5 * zoom, Color32::BLACK);

        // Head
        painter.circle_stroke(Pos2::new(pos.x, head_y), head_radius, stroke);

        // Body
        painter.line_segment([Pos2::new(pos.x, body_top), Pos2::new(pos.x, body_bottom)], stroke);

        // Arms
        painter.line_segment(
            [Pos2::new(pos.x - 15.0 * zoom, arm_y), Pos2::new(pos.x + 15.0 * zoom, arm_y)],
            stroke,
        );

        // Legs
        painter.line_segment(
            [Pos2::new(pos.x, body_bottom), Pos2::new(pos.x - leg_spread, leg_bottom)],
            stroke,
        );
        painter.line_segment(
            [Pos2::new(pos.x, body_bottom), Pos2::new(pos.x + leg_spread, leg_bottom)],
            stroke,
        );
    }

    /// Render a use case (ellipse)
    fn render_use_case(&self, use_case: &UseCase, painter: &egui::Painter, zoom: f32) {
        let bounds = &use_case.bounds;
        let center = self.scale_pos(bounds.center().x, bounds.center().y, zoom);
        let radius = Vec2::new(bounds.width() / 2.0 * zoom, bounds.height() / 2.0 * zoom);

        let fill = use_case.fill_color.map(color_to_egui).unwrap_or(Color32::from_rgb(255, 255, 220));

        // Draw ellipse
        painter.add(egui::Shape::ellipse_filled(center, radius, fill));
        painter.add(egui::Shape::ellipse_stroke(center, radius, Stroke::new(zoom, Color32::BLACK)));

        // Draw name
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            &use_case.name,
            egui::FontId::proportional(11.0 * zoom),
            Color32::BLACK,
        );

        // Draw selection indicator
        if use_case.has_focus {
            painter.add(egui::Shape::ellipse_stroke(
                center,
                Vec2::new(radius.x + 3.0 * zoom, radius.y + 3.0 * zoom),
                Stroke::new(2.0 * zoom, Color32::from_rgb(0, 120, 215)),
            ));
        }
    }

    /// Render a system boundary
    fn render_system_boundary(&self, boundary: &SystemBoundary, painter: &egui::Painter, zoom: f32) {
        let bounds = &boundary.bounds;
        let rect = self.scale_rect(bounds, zoom);

        let fill = boundary.fill_color.map(color_to_egui).unwrap_or(Color32::from_rgba_unmultiplied(245, 245, 245, 100));
        painter.rect(rect, Rounding::same(4.0 * zoom), fill, Stroke::new(zoom, Color32::BLACK));

        // Draw system name at top
        painter.text(
            Pos2::new(bounds.center().x * zoom, (bounds.y1 + 15.0) * zoom),
            egui::Align2::CENTER_CENTER,
            &boundary.name,
            egui::FontId::proportional(12.0 * zoom),
            Color32::BLACK,
        );
    }

    /// Render a use case relationship
    fn render_uc_relationship(&self, rel: &UseCaseRelationship, diagram: &Diagram, painter: &egui::Painter, zoom: f32) {
        // Get source and target positions (already scaled)
        let (source_pos, target_pos) = self.get_uc_relationship_endpoints(rel, diagram, zoom);

        let stroke = if rel.selected {
            Stroke::new(2.0 * zoom, Color32::from_rgb(255, 165, 0))
        } else {
            Stroke::new(zoom, Color32::BLACK)
        };

        // Draw line (dashed for include/extend)
        if rel.kind.is_dashed() {
            let dash_len: f32 = 6.0 * zoom;
            let gap_len: f32 = 4.0 * zoom;
            let dx = target_pos.x - source_pos.x;
            let dy = target_pos.y - source_pos.y;
            let len = (dx * dx + dy * dy).sqrt();
            let ux = dx / len;
            let uy = dy / len;
            let mut t = 0.0;
            while t < len {
                let seg_len = dash_len.min(len - t);
                let start = Pos2::new(source_pos.x + t * ux, source_pos.y + t * uy);
                let end = Pos2::new(source_pos.x + (t + seg_len) * ux, source_pos.y + (t + seg_len) * uy);
                painter.line_segment([start, end], stroke);
                t += dash_len + gap_len;
            }
        } else {
            painter.line_segment([source_pos, target_pos], stroke);
        }

        // Draw arrowhead for relationships with arrows
        if rel.kind.has_arrow() {
            let dx = target_pos.x - source_pos.x;
            let dy = target_pos.y - source_pos.y;
            let len = (dx * dx + dy * dy).sqrt();
            let ux = dx / len;
            let uy = dy / len;
            let arrow_size = 10.0 * zoom;

            if matches!(rel.kind, RelationshipKind::Generalization) {
                // Hollow triangle
                let tip = target_pos;
                let left = Pos2::new(
                    tip.x - ux * arrow_size - uy * arrow_size / 2.0,
                    tip.y - uy * arrow_size + ux * arrow_size / 2.0,
                );
                let right = Pos2::new(
                    tip.x - ux * arrow_size + uy * arrow_size / 2.0,
                    tip.y - uy * arrow_size - ux * arrow_size / 2.0,
                );
                painter.add(egui::Shape::convex_polygon(
                    vec![tip, left, right],
                    Color32::WHITE,
                    Stroke::new(zoom, Color32::BLACK),
                ));
            } else {
                // Open arrowhead
                let tip = target_pos;
                painter.line_segment(
                    [tip, Pos2::new(tip.x - ux * arrow_size - uy * arrow_size / 2.0, tip.y - uy * arrow_size + ux * arrow_size / 2.0)],
                    stroke,
                );
                painter.line_segment(
                    [tip, Pos2::new(tip.x - ux * arrow_size + uy * arrow_size / 2.0, tip.y - uy * arrow_size - ux * arrow_size / 2.0)],
                    stroke,
                );
            }
        }

        // Draw stereotype label
        if let Some(stereotype) = rel.kind.stereotype() {
            let mid = Pos2::new((source_pos.x + target_pos.x) / 2.0, (source_pos.y + target_pos.y) / 2.0 - 10.0 * zoom);
            painter.text(
                mid,
                egui::Align2::CENTER_BOTTOM,
                stereotype,
                egui::FontId::proportional(9.0 * zoom),
                Color32::BLACK,
            );
        }
    }

    /// Get endpoints for a use case relationship
    fn get_uc_relationship_endpoints(&self, rel: &UseCaseRelationship, diagram: &Diagram, zoom: f32) -> (Pos2, Pos2) {
        let source_pos = match rel.source_kind {
            UseCaseElementKind::Actor => {
                diagram.actors.iter()
                    .find(|a| a.id == rel.source_id)
                    .map(|a| Pos2::new(a.x * zoom, (a.y + a.height / 2.0) * zoom))
                    .unwrap_or(Pos2::new(0.0, 0.0))
            }
            UseCaseElementKind::UseCase => {
                diagram.use_cases.iter()
                    .find(|u| u.id == rel.source_id)
                    .map(|u| Pos2::new(u.bounds.center().x * zoom, u.bounds.center().y * zoom))
                    .unwrap_or(Pos2::new(0.0, 0.0))
            }
        };

        let target_pos = match rel.target_kind {
            UseCaseElementKind::Actor => {
                diagram.actors.iter()
                    .find(|a| a.id == rel.target_id)
                    .map(|a| Pos2::new(a.x * zoom, (a.y + a.height / 2.0) * zoom))
                    .unwrap_or(Pos2::new(100.0 * zoom, 100.0 * zoom))
            }
            UseCaseElementKind::UseCase => {
                diagram.use_cases.iter()
                    .find(|u| u.id == rel.target_id)
                    .map(|u| Pos2::new(u.bounds.center().x * zoom, u.bounds.center().y * zoom))
                    .unwrap_or(Pos2::new(100.0 * zoom, 100.0 * zoom))
            }
        };

        (source_pos, target_pos)
    }

    // ===== Activity Diagram Rendering =====

    /// Render a swimlane
    fn render_swimlane(&self, swimlane: &Swimlane, painter: &egui::Painter, zoom: f32) {
        let bounds = &swimlane.bounds;
        let rect = self.scale_rect(bounds, zoom);

        let fill = swimlane.fill_color.map(color_to_egui).unwrap_or(Color32::from_rgba_unmultiplied(250, 250, 250, 200));
        painter.rect(rect, Rounding::ZERO, fill, Stroke::new(zoom, Color32::BLACK));

        // Draw header
        let header_rect = swimlane.header_rect();
        let header_egui = self.scale_rect(&header_rect, zoom);
        painter.rect_filled(header_egui, Rounding::ZERO, Color32::from_rgb(220, 220, 240));
        painter.rect_stroke(header_egui, Rounding::ZERO, Stroke::new(zoom, Color32::BLACK));

        // Draw name
        painter.text(
            header_egui.center(),
            egui::Align2::CENTER_CENTER,
            &swimlane.name,
            egui::FontId::proportional(11.0 * zoom),
            Color32::BLACK,
        );
    }

    /// Render an action
    fn render_action(&self, action: &Action, painter: &egui::Painter, zoom: f32) {
        let bounds = &action.bounds;
        let fill = action.fill_color.map(color_to_egui).unwrap_or(Color32::from_rgb(255, 255, 220));

        match action.kind {
            ActionKind::Action | ActionKind::CallBehavior | ActionKind::CallOperation => {
                // Rounded rectangle
                let rect = self.scale_rect(bounds, zoom);
                let rounding = action.corner_rounding() * zoom;
                painter.rect(rect, Rounding::same(rounding), fill, Stroke::new(zoom, Color32::BLACK));

                // Draw name
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &action.name,
                    egui::FontId::proportional(11.0 * zoom),
                    Color32::BLACK,
                );

                // Draw fork icon for CallBehavior
                if matches!(action.kind, ActionKind::CallBehavior) {
                    let icon_y = (bounds.y2 - 8.0) * zoom;
                    let icon_x = (bounds.x2 - 15.0) * zoom;
                    // Small fork icon
                    painter.line_segment(
                        [Pos2::new(icon_x, icon_y - 5.0 * zoom), Pos2::new(icon_x, icon_y + 2.0 * zoom)],
                        Stroke::new(zoom, Color32::BLACK),
                    );
                    painter.line_segment(
                        [Pos2::new(icon_x - 4.0 * zoom, icon_y - 2.0 * zoom), Pos2::new(icon_x, icon_y - 5.0 * zoom)],
                        Stroke::new(zoom, Color32::BLACK),
                    );
                    painter.line_segment(
                        [Pos2::new(icon_x + 4.0 * zoom, icon_y - 2.0 * zoom), Pos2::new(icon_x, icon_y - 5.0 * zoom)],
                        Stroke::new(zoom, Color32::BLACK),
                    );
                }
            }
            ActionKind::SendSignal | ActionKind::AcceptEvent => {
                // Pentagon shapes
                if let Some(points) = action.shape_points() {
                    let egui_points: Vec<Pos2> = points.iter()
                        .map(|p| Pos2::new(p.x * zoom, p.y * zoom))
                        .collect();
                    painter.add(egui::Shape::convex_polygon(
                        egui_points.clone(),
                        fill,
                        Stroke::new(zoom, Color32::BLACK),
                    ));
                }

                // Draw name
                let center = self.scale_pos(bounds.center().x, bounds.center().y, zoom);
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    &action.name,
                    egui::FontId::proportional(10.0 * zoom),
                    Color32::BLACK,
                );
            }
            ActionKind::AcceptTimeEvent => {
                // Hourglass shape
                let cx = bounds.center().x * zoom;
                let cy = bounds.center().y * zoom;
                let hw = (bounds.width() / 2.0 - 2.0) * zoom;
                let hh = (bounds.height() / 2.0 - 2.0) * zoom;
                let points = vec![
                    Pos2::new(cx - hw, cy - hh),
                    Pos2::new(cx + hw, cy - hh),
                    Pos2::new(cx - hw, cy + hh),
                    Pos2::new(cx + hw, cy + hh),
                ];
                painter.line_segment([points[0], points[2]], Stroke::new(zoom, Color32::BLACK));
                painter.line_segment([points[0], points[3]], Stroke::new(zoom, Color32::BLACK));
                painter.line_segment([points[1], points[2]], Stroke::new(zoom, Color32::BLACK));
                painter.line_segment([points[1], points[3]], Stroke::new(zoom, Color32::BLACK));
            }
        }

        // Draw selection indicator
        if action.has_focus {
            let rect = Rect::from_min_max(
                Pos2::new((bounds.x1 - 2.0) * zoom, (bounds.y1 - 2.0) * zoom),
                Pos2::new((bounds.x2 + 2.0) * zoom, (bounds.y2 + 2.0) * zoom),
            );
            painter.rect_stroke(rect, Rounding::same((action.corner_rounding() + 2.0) * zoom), Stroke::new(2.0 * zoom, Color32::from_rgb(0, 120, 215)));
        }
    }

    /// Render a control flow
    fn render_control_flow(&self, flow: &ControlFlow, diagram: &Diagram, painter: &egui::Painter, zoom: f32) {
        // Get source and target positions (scaled)
        let source_pos = diagram.find_action(flow.source_id)
            .map(|a| Pos2::new(a.bounds.center().x * zoom, a.bounds.y2 * zoom))
            .or_else(|| diagram.find_node(flow.source_id).map(|n| Pos2::new(n.bounds().center().x * zoom, n.bounds().y2 * zoom)))
            .unwrap_or(Pos2::new(0.0, 0.0));

        let target_pos = diagram.find_action(flow.target_id)
            .map(|a| Pos2::new(a.bounds.center().x * zoom, a.bounds.y1 * zoom))
            .or_else(|| diagram.find_node(flow.target_id).map(|n| Pos2::new(n.bounds().center().x * zoom, n.bounds().y1 * zoom)))
            .unwrap_or(Pos2::new(100.0 * zoom, 100.0 * zoom));

        let stroke = if flow.selected {
            Stroke::new(2.0 * zoom, Color32::from_rgb(255, 165, 0))
        } else {
            Stroke::new(zoom, Color32::BLACK)
        };

        // Draw line with waypoints
        let mut points = vec![source_pos];
        for wp in &flow.waypoints {
            points.push(Pos2::new(wp.x * zoom, wp.y * zoom));
        }
        points.push(target_pos);

        for i in 0..points.len() - 1 {
            painter.line_segment([points[i], points[i + 1]], stroke);
        }

        // Draw arrowhead at target
        let last_seg_start = points[points.len() - 2];
        let dx = target_pos.x - last_seg_start.x;
        let dy = target_pos.y - last_seg_start.y;
        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.0 {
            let ux = dx / len;
            let uy = dy / len;
            let arrow_size = 8.0 * zoom;
            painter.line_segment(
                [target_pos, Pos2::new(target_pos.x - ux * arrow_size - uy * arrow_size / 2.0, target_pos.y - uy * arrow_size + ux * arrow_size / 2.0)],
                stroke,
            );
            painter.line_segment(
                [target_pos, Pos2::new(target_pos.x - ux * arrow_size + uy * arrow_size / 2.0, target_pos.y - uy * arrow_size - ux * arrow_size / 2.0)],
                stroke,
            );
        }

        // Draw guard label
        if let Some(label) = flow.label() {
            let mid = Pos2::new((source_pos.x + target_pos.x) / 2.0, (source_pos.y + target_pos.y) / 2.0 - 10.0 * zoom);
            painter.text(
                mid,
                egui::Align2::CENTER_BOTTOM,
                &label,
                egui::FontId::proportional(9.0 * zoom),
                Color32::BLACK,
            );
        }
    }
}

/// Convert core Color to egui Color32
fn color_to_egui(color: Color) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a)
}

impl Default for DiagramCanvas {
    fn default() -> Self {
        Self::new()
    }
}
