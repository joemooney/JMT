//! Diagram rendering using egui Painter

use eframe::egui::{self, Color32, Pos2, Rect, Rounding, Stroke, Vec2};
use jmt_core::{Diagram, Node, Connection};
use jmt_core::node::{PseudoStateKind, Side};
use jmt_core::geometry::Color;

/// Canvas for rendering diagrams
pub struct DiagramCanvas {
    // Future: scroll offset, zoom, etc.
}

impl DiagramCanvas {
    pub fn new() -> Self {
        Self {}
    }

    /// Render the diagram to the canvas
    pub fn render(&self, diagram: &Diagram, painter: &egui::Painter, _rect: Rect) {
        // Render root state boundary (optional - can be hidden)
        // self.render_root_state(&diagram.root_state, painter);

        // Render all nodes
        for node in diagram.nodes() {
            self.render_node(node, painter, &diagram.settings);
        }

        // Render all connections
        for conn in diagram.connections() {
            self.render_connection(conn, painter, &diagram.settings);
        }
    }

    /// Render a node
    fn render_node(&self, node: &Node, painter: &egui::Painter, settings: &jmt_core::DiagramSettings) {
        match node {
            Node::State(state) => self.render_state(state, painter, settings),
            Node::Pseudo(pseudo) => self.render_pseudo_state(pseudo, painter, settings),
        }
    }

    /// Render a state node
    fn render_state(&self, state: &jmt_core::node::State, painter: &egui::Painter, settings: &jmt_core::DiagramSettings) {
        let bounds = &state.bounds;
        let rect = Rect::from_min_max(
            Pos2::new(bounds.x1, bounds.y1),
            Pos2::new(bounds.x2, bounds.y2),
        );

        // Fill color
        let fill_color = state.fill_color.unwrap_or(settings.state_color);
        let fill = color_to_egui(fill_color);

        // Draw rounded rectangle
        let rounding = Rounding::same(settings.corner_rounding);
        painter.rect(rect, rounding, fill, Stroke::new(1.0, Color32::BLACK));

        // Draw state name
        let text_pos = Pos2::new(rect.center().x, rect.min.y + 12.0);
        painter.text(
            text_pos,
            egui::Align2::CENTER_TOP,
            &state.name,
            egui::FontId::proportional(12.0),
            Color32::BLACK,
        );

        // Draw activities if present
        if state.has_activities() {
            // Draw separator line
            let line_y = rect.min.y + 24.0;
            painter.line_segment(
                [Pos2::new(rect.min.x, line_y), Pos2::new(rect.max.x, line_y)],
                Stroke::new(1.0, Color32::BLACK),
            );

            // Draw activities text
            let mut y = line_y + 4.0;
            if !state.entry_activity.is_empty() {
                painter.text(
                    Pos2::new(rect.min.x + 4.0, y),
                    egui::Align2::LEFT_TOP,
                    format!("entry / {}", state.entry_activity),
                    egui::FontId::proportional(10.0),
                    Color32::BLACK,
                );
                y += 12.0;
            }
            if !state.exit_activity.is_empty() {
                painter.text(
                    Pos2::new(rect.min.x + 4.0, y),
                    egui::Align2::LEFT_TOP,
                    format!("exit / {}", state.exit_activity),
                    egui::FontId::proportional(10.0),
                    Color32::BLACK,
                );
                y += 12.0;
            }
            if !state.do_activity.is_empty() {
                painter.text(
                    Pos2::new(rect.min.x + 4.0, y),
                    egui::Align2::LEFT_TOP,
                    format!("do / {}", state.do_activity),
                    egui::FontId::proportional(10.0),
                    Color32::BLACK,
                );
            }
        }

        // Draw focus corners if selected
        if state.has_focus {
            self.render_focus_corners(rect, painter, settings.corner_size);
        }
    }

    /// Render a pseudo-state node
    fn render_pseudo_state(&self, pseudo: &jmt_core::node::PseudoState, painter: &egui::Painter, settings: &jmt_core::DiagramSettings) {
        let bounds = &pseudo.bounds;
        let center = Pos2::new(bounds.center().x, bounds.center().y);

        match pseudo.kind {
            PseudoStateKind::Initial => {
                // Filled black circle
                let radius = bounds.width().min(bounds.height()) / 2.0 - 2.0;
                painter.circle_filled(center, radius, Color32::BLACK);
            }
            PseudoStateKind::Final => {
                // Double circle (outer ring + inner filled)
                let outer_radius = bounds.width().min(bounds.height()) / 2.0 - 2.0;
                let inner_radius = outer_radius - 4.0;
                painter.circle_stroke(center, outer_radius, Stroke::new(1.0, Color32::BLACK));
                painter.circle_filled(center, inner_radius, Color32::BLACK);
            }
            PseudoStateKind::Choice | PseudoStateKind::Junction => {
                // Diamond shape
                let half_w = bounds.width() / 2.0;
                let half_h = bounds.height() / 2.0;
                let points = vec![
                    Pos2::new(center.x, center.y - half_h),  // top
                    Pos2::new(center.x + half_w, center.y),  // right
                    Pos2::new(center.x, center.y + half_h),  // bottom
                    Pos2::new(center.x - half_w, center.y),  // left
                    Pos2::new(center.x, center.y - half_h),  // close
                ];
                painter.add(egui::Shape::line(points, Stroke::new(1.0, Color32::BLACK)));
            }
            PseudoStateKind::Fork | PseudoStateKind::Join => {
                // Thick horizontal or vertical bar
                let rect = Rect::from_min_max(
                    Pos2::new(bounds.x1, bounds.y1),
                    Pos2::new(bounds.x2, bounds.y2),
                );
                painter.rect_filled(rect, Rounding::ZERO, Color32::BLACK);
            }
        }

        // Draw focus corners if selected
        if pseudo.has_focus {
            let rect = Rect::from_min_max(
                Pos2::new(bounds.x1, bounds.y1),
                Pos2::new(bounds.x2, bounds.y2),
            );
            self.render_focus_corners(rect, painter, settings.pseudo_corner_size);
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
    fn render_connection(&self, conn: &Connection, painter: &egui::Painter, settings: &jmt_core::DiagramSettings) {
        let stroke = if conn.selected {
            Stroke::new(2.0, Color32::from_rgb(255, 165, 0))  // Orange
        } else {
            Stroke::new(1.0, Color32::BLACK)
        };

        // Draw line segments
        for segment in &conn.segments {
            painter.line_segment(
                [
                    Pos2::new(segment.start.x, segment.start.y),
                    Pos2::new(segment.end.x, segment.end.y),
                ],
                stroke,
            );
        }

        // Draw arrowhead at target
        if let Some(end_point) = conn.end_point() {
            self.render_arrowhead(
                Pos2::new(end_point.x, end_point.y),
                conn.target_side,
                painter,
                settings,
                stroke,
            );
        }

        // Draw label if present
        let label = conn.label();
        if !label.is_empty() {
            if let (Some(start), Some(end)) = (conn.start_point(), conn.end_point()) {
                let mid = Pos2::new(
                    (start.x + end.x) / 2.0,
                    (start.y + end.y) / 2.0 - 10.0,
                );
                painter.text(
                    mid,
                    egui::Align2::CENTER_BOTTOM,
                    &label,
                    egui::FontId::proportional(10.0),
                    Color32::BLACK,
                );
            }
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
    ) {
        let w = settings.arrow_width;
        let h = settings.arrow_height;

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
