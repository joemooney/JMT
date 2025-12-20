//! Toolbar panel with graphical icons

use eframe::egui::{self, Color32, Pos2, Rounding, Stroke, Vec2};
use jmt_core::{EditMode, DiagramType};
use crate::app::JmtApp;

/// Size of toolbar icons
const ICON_SIZE: f32 = 20.0;

pub struct Toolbar;

impl Toolbar {
    pub fn show(ui: &mut egui::Ui, app: &mut JmtApp) {
        ui.horizontal(|ui| {
            // Undo/Redo buttons
            let can_undo = app.current_diagram()
                .map(|s| s.diagram.can_undo())
                .unwrap_or(false);
            let can_redo = app.current_diagram()
                .map(|s| s.diagram.can_redo())
                .unwrap_or(false);

            if ui.add_enabled(can_undo, egui::Button::new("⟲ Undo"))
                .on_hover_text("Undo last action (Ctrl+Z)")
                .clicked()
            {
                if let Some(state) = app.current_diagram_mut() {
                    state.diagram.undo();
                }
            }

            if ui.add_enabled(can_redo, egui::Button::new("⟳ Redo"))
                .on_hover_text("Redo last undone action (Ctrl+Shift+Z)")
                .clicked()
            {
                if let Some(state) = app.current_diagram_mut() {
                    state.diagram.redo();
                }
            }

            ui.separator();

            // Selection tools
            ui.label("Select:");
            Self::tool_button(ui, app, EditMode::Arrow, "↖ Arrow", "Select and move nodes");

            ui.separator();

            // Get current diagram type
            let diagram_type = app.current_diagram()
                .map(|s| s.diagram.diagram_type)
                .unwrap_or(DiagramType::StateMachine);

            // Show diagram-specific tools
            match diagram_type {
                DiagramType::StateMachine => Self::show_state_machine_tools(ui, app),
                DiagramType::Sequence => Self::show_sequence_tools(ui, app),
                DiagramType::UseCase => Self::show_use_case_tools(ui, app),
                DiagramType::Activity => Self::show_activity_tools(ui, app),
            }

            ui.separator();

            // Align dropdown
            let has_multiple_selected = app.current_diagram()
                .map(|s| s.diagram.selected_nodes().len() >= 2)
                .unwrap_or(false);

            ui.add_enabled_ui(has_multiple_selected, |ui| {
                egui::menu::menu_button(ui, "⬚ Align", |ui| {
                    ui.set_min_width(150.0);

                    ui.label("Horizontal");
                    if ui.button("⫷ Align Left").clicked() {
                        Self::align_nodes(app, AlignMode::Left);
                        ui.close_menu();
                    }
                    if ui.button("⫿ Align Center").clicked() {
                        Self::align_nodes(app, AlignMode::CenterH);
                        ui.close_menu();
                    }
                    if ui.button("⫸ Align Right").clicked() {
                        Self::align_nodes(app, AlignMode::Right);
                        ui.close_menu();
                    }

                    ui.separator();
                    ui.label("Vertical");
                    if ui.button("⫠ Align Top").clicked() {
                        Self::align_nodes(app, AlignMode::Top);
                        ui.close_menu();
                    }
                    if ui.button("⫟ Align Middle").clicked() {
                        Self::align_nodes(app, AlignMode::CenterV);
                        ui.close_menu();
                    }
                    if ui.button("⫡ Align Bottom").clicked() {
                        Self::align_nodes(app, AlignMode::Bottom);
                        ui.close_menu();
                    }

                    ui.separator();
                    ui.label("Distribute");
                    if ui.button("↔ Distribute Horizontally").clicked() {
                        Self::distribute_nodes(app, DistributeMode::Horizontal);
                        ui.close_menu();
                    }
                    if ui.button("↕ Distribute Vertically").clicked() {
                        Self::distribute_nodes(app, DistributeMode::Vertical);
                        ui.close_menu();
                    }
                });
            });
        });
    }

    fn tool_button(ui: &mut egui::Ui, app: &mut JmtApp, mode: EditMode, label: &str, tooltip: &str) {
        let current_mode = app.current_diagram()
            .map(|_| app.edit_mode == mode)
            .unwrap_or(false);

        let response = ui.selectable_label(current_mode, label);
        if response.on_hover_text(tooltip).clicked() {
            app.set_edit_mode(mode);
        }
    }

    /// Create a tool button with a custom graphical icon
    fn icon_tool_button(
        ui: &mut egui::Ui,
        app: &mut JmtApp,
        mode: EditMode,
        tooltip: &str,
        draw_icon: impl FnOnce(&egui::Painter, egui::Rect),
    ) {
        let current_mode = app.edit_mode == mode;

        let size = Vec2::splat(ICON_SIZE + 8.0);
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

        // Draw button background
        let bg_color = if current_mode {
            Color32::from_rgb(200, 220, 255)
        } else if response.hovered() {
            Color32::from_rgb(230, 230, 230)
        } else {
            Color32::TRANSPARENT
        };

        ui.painter().rect(rect, Rounding::same(4.0), bg_color, Stroke::NONE);

        // Draw the icon
        let icon_rect = rect.shrink(4.0);
        draw_icon(ui.painter(), icon_rect);

        // Handle click
        if response.on_hover_text(tooltip).clicked() {
            app.set_edit_mode(mode);
        }
    }

    fn show_state_machine_tools(ui: &mut egui::Ui, app: &mut JmtApp) {
        ui.label("Add:");

        // State - rounded rectangle
        Self::icon_tool_button(ui, app, EditMode::AddState, "Add a state", |painter, rect| {
            let inner = rect.shrink(2.0);
            painter.rect(inner, Rounding::same(4.0), Color32::from_rgb(255, 255, 204), Stroke::new(1.0, Color32::BLACK));
        });

        // Initial - filled circle
        Self::icon_tool_button(ui, app, EditMode::AddInitial, "Add initial pseudo-state", |painter, rect| {
            painter.circle_filled(rect.center(), rect.width() / 3.0, Color32::BLACK);
        });

        // Final - double circle
        Self::icon_tool_button(ui, app, EditMode::AddFinal, "Add final pseudo-state", |painter, rect| {
            let center = rect.center();
            let r = rect.width() / 3.0;
            painter.circle_stroke(center, r, Stroke::new(1.0, Color32::BLACK));
            painter.circle_filled(center, r * 0.6, Color32::BLACK);
        });

        // Choice - diamond
        Self::icon_tool_button(ui, app, EditMode::AddChoice, "Add choice pseudo-state", |painter, rect| {
            let center = rect.center();
            let s = rect.width() / 3.0;
            let points = vec![
                Pos2::new(center.x, center.y - s),
                Pos2::new(center.x + s, center.y),
                Pos2::new(center.x, center.y + s),
                Pos2::new(center.x - s, center.y),
            ];
            painter.add(egui::Shape::convex_polygon(points, Color32::WHITE, Stroke::new(1.0, Color32::BLACK)));
        });

        // Junction - filled diamond
        Self::icon_tool_button(ui, app, EditMode::AddJunction, "Add junction pseudo-state", |painter, rect| {
            let center = rect.center();
            let s = rect.width() / 3.5;
            let points = vec![
                Pos2::new(center.x, center.y - s),
                Pos2::new(center.x + s, center.y),
                Pos2::new(center.x, center.y + s),
                Pos2::new(center.x - s, center.y),
            ];
            painter.add(egui::Shape::convex_polygon(points, Color32::BLACK, Stroke::new(1.0, Color32::BLACK)));
        });

        // Fork - horizontal bar
        Self::icon_tool_button(ui, app, EditMode::AddFork, "Add fork pseudo-state", |painter, rect| {
            let center = rect.center();
            let w = rect.width() * 0.7;
            let h = 4.0;
            let bar = egui::Rect::from_center_size(center, Vec2::new(w, h));
            painter.rect_filled(bar, Rounding::ZERO, Color32::BLACK);
        });

        // Join - horizontal bar (same as fork)
        Self::icon_tool_button(ui, app, EditMode::AddJoin, "Add join pseudo-state", |painter, rect| {
            let center = rect.center();
            let w = rect.width() * 0.7;
            let h = 4.0;
            let bar = egui::Rect::from_center_size(center, Vec2::new(w, h));
            painter.rect_filled(bar, Rounding::ZERO, Color32::BLACK);
        });

        ui.separator();
        ui.label("Connect:");

        // Transition - arrow
        Self::icon_tool_button(ui, app, EditMode::Connect, "Create a transition", |painter, rect| {
            let left = Pos2::new(rect.left() + 4.0, rect.center().y);
            let right = Pos2::new(rect.right() - 4.0, rect.center().y);
            painter.line_segment([left, right], Stroke::new(1.5, Color32::BLACK));
            // Arrowhead
            let arrow_size = 5.0;
            painter.line_segment([right, Pos2::new(right.x - arrow_size, right.y - arrow_size)], Stroke::new(1.5, Color32::BLACK));
            painter.line_segment([right, Pos2::new(right.x - arrow_size, right.y + arrow_size)], Stroke::new(1.5, Color32::BLACK));
        });
    }

    fn show_sequence_tools(ui: &mut egui::Ui, app: &mut JmtApp) {
        ui.label("Add:");

        // Lifeline - rectangle head with dashed line
        Self::icon_tool_button(ui, app, EditMode::AddLifeline, "Add a lifeline", |painter, rect| {
            let head_rect = egui::Rect::from_min_size(
                Pos2::new(rect.center().x - 6.0, rect.top() + 2.0),
                Vec2::new(12.0, 8.0),
            );
            painter.rect(head_rect, Rounding::same(1.0), Color32::WHITE, Stroke::new(1.0, Color32::BLACK));
            // Dashed line
            let line_top = head_rect.bottom();
            let line_bottom = rect.bottom() - 2.0;
            let mut y = line_top;
            while y < line_bottom {
                let end_y = (y + 3.0).min(line_bottom);
                painter.line_segment(
                    [Pos2::new(rect.center().x, y), Pos2::new(rect.center().x, end_y)],
                    Stroke::new(1.0, Color32::BLACK),
                );
                y += 5.0;
            }
        });

        // Activation - filled rectangle
        Self::icon_tool_button(ui, app, EditMode::AddActivation, "Add an activation box", |painter, rect| {
            let act_rect = egui::Rect::from_center_size(rect.center(), Vec2::new(8.0, 14.0));
            painter.rect(act_rect, Rounding::ZERO, Color32::WHITE, Stroke::new(1.0, Color32::BLACK));
        });

        // Fragment - box with pentagon
        Self::icon_tool_button(ui, app, EditMode::AddFragment, "Add a combined fragment", |painter, rect| {
            let inner = rect.shrink(3.0);
            painter.rect(inner, Rounding::ZERO, Color32::from_rgba_unmultiplied(240, 240, 255, 200), Stroke::new(1.0, Color32::BLACK));
            // Pentagon tag
            let tag = vec![
                Pos2::new(inner.left(), inner.top()),
                Pos2::new(inner.left() + 10.0, inner.top()),
                Pos2::new(inner.left() + 12.0, inner.top() + 4.0),
                Pos2::new(inner.left() + 10.0, inner.top() + 8.0),
                Pos2::new(inner.left(), inner.top() + 8.0),
            ];
            painter.add(egui::Shape::convex_polygon(tag, Color32::WHITE, Stroke::new(1.0, Color32::BLACK)));
        });

        ui.separator();
        ui.label("Messages:");

        // Sync message - solid arrow
        Self::icon_tool_button(ui, app, EditMode::AddSyncMessage, "Add synchronous message", |painter, rect| {
            let left = Pos2::new(rect.left() + 3.0, rect.center().y);
            let right = Pos2::new(rect.right() - 3.0, rect.center().y);
            painter.line_segment([left, right], Stroke::new(1.5, Color32::BLACK));
            // Filled arrowhead
            let arrow = vec![
                right,
                Pos2::new(right.x - 6.0, right.y - 4.0),
                Pos2::new(right.x - 6.0, right.y + 4.0),
            ];
            painter.add(egui::Shape::convex_polygon(arrow, Color32::BLACK, Stroke::NONE));
        });

        // Async message - open arrow
        Self::icon_tool_button(ui, app, EditMode::AddAsyncMessage, "Add asynchronous message", |painter, rect| {
            let left = Pos2::new(rect.left() + 3.0, rect.center().y);
            let right = Pos2::new(rect.right() - 3.0, rect.center().y);
            painter.line_segment([left, right], Stroke::new(1.5, Color32::BLACK));
            // Open arrowhead
            painter.line_segment([right, Pos2::new(right.x - 5.0, right.y - 4.0)], Stroke::new(1.5, Color32::BLACK));
            painter.line_segment([right, Pos2::new(right.x - 5.0, right.y + 4.0)], Stroke::new(1.5, Color32::BLACK));
        });

        // Return message - dashed with open arrow
        Self::icon_tool_button(ui, app, EditMode::AddReturnMessage, "Add return message", |painter, rect| {
            let left = Pos2::new(rect.left() + 3.0, rect.center().y);
            let right = Pos2::new(rect.right() - 3.0, rect.center().y);
            // Dashed line
            let mut x = left.x;
            while x < right.x - 6.0 {
                let end_x = (x + 4.0).min(right.x - 6.0);
                painter.line_segment(
                    [Pos2::new(x, rect.center().y), Pos2::new(end_x, rect.center().y)],
                    Stroke::new(1.0, Color32::BLACK),
                );
                x += 6.0;
            }
            // Arrow pointing left
            painter.line_segment([left, Pos2::new(left.x + 5.0, left.y - 4.0)], Stroke::new(1.5, Color32::BLACK));
            painter.line_segment([left, Pos2::new(left.x + 5.0, left.y + 4.0)], Stroke::new(1.5, Color32::BLACK));
        });

        // Self message - loop back arrow
        Self::icon_tool_button(ui, app, EditMode::AddSelfMessage, "Add self message", |painter, rect| {
            let start = Pos2::new(rect.center().x, rect.top() + 4.0);
            let mid_right = Pos2::new(rect.right() - 4.0, rect.top() + 4.0);
            let mid_right_bottom = Pos2::new(rect.right() - 4.0, rect.bottom() - 4.0);
            let end = Pos2::new(rect.center().x, rect.bottom() - 4.0);
            let stroke = Stroke::new(1.0, Color32::BLACK);
            painter.line_segment([start, mid_right], stroke);
            painter.line_segment([mid_right, mid_right_bottom], stroke);
            painter.line_segment([mid_right_bottom, end], stroke);
            // Arrow
            painter.line_segment([end, Pos2::new(end.x - 4.0, end.y - 4.0)], stroke);
            painter.line_segment([end, Pos2::new(end.x + 4.0, end.y - 4.0)], stroke);
        });
    }

    fn show_use_case_tools(ui: &mut egui::Ui, app: &mut JmtApp) {
        ui.label("Add:");

        // Actor - stick figure
        Self::icon_tool_button(ui, app, EditMode::AddActor, "Add an actor", |painter, rect| {
            let center_x = rect.center().x;
            let head_y = rect.top() + 4.0;
            let head_r = 3.0;
            let body_top = head_y + head_r + 1.0;
            let body_bottom = body_top + 6.0;
            let arm_y = body_top + 2.0;
            let leg_bottom = rect.bottom() - 2.0;

            let stroke = Stroke::new(1.0, Color32::BLACK);
            // Head
            painter.circle_stroke(Pos2::new(center_x, head_y), head_r, stroke);
            // Body
            painter.line_segment([Pos2::new(center_x, body_top), Pos2::new(center_x, body_bottom)], stroke);
            // Arms
            painter.line_segment([Pos2::new(center_x - 5.0, arm_y), Pos2::new(center_x + 5.0, arm_y)], stroke);
            // Legs
            painter.line_segment([Pos2::new(center_x, body_bottom), Pos2::new(center_x - 4.0, leg_bottom)], stroke);
            painter.line_segment([Pos2::new(center_x, body_bottom), Pos2::new(center_x + 4.0, leg_bottom)], stroke);
        });

        // Use Case - ellipse
        Self::icon_tool_button(ui, app, EditMode::AddUseCase, "Add a use case", |painter, rect| {
            let center = rect.center();
            let radius = Vec2::new(rect.width() / 2.5, rect.height() / 3.0);
            painter.add(egui::Shape::ellipse_filled(center, radius, Color32::from_rgb(255, 255, 220)));
            painter.add(egui::Shape::ellipse_stroke(center, radius, Stroke::new(1.0, Color32::BLACK)));
        });

        // System Boundary - rectangle with header
        Self::icon_tool_button(ui, app, EditMode::AddSystemBoundary, "Add system boundary", |painter, rect| {
            let inner = rect.shrink(3.0);
            painter.rect(inner, Rounding::same(2.0), Color32::from_rgba_unmultiplied(245, 245, 245, 200), Stroke::new(1.0, Color32::BLACK));
            // Header line
            painter.line_segment(
                [Pos2::new(inner.left(), inner.top() + 6.0), Pos2::new(inner.right(), inner.top() + 6.0)],
                Stroke::new(0.5, Color32::GRAY),
            );
        });

        ui.separator();
        ui.label("Connect:");

        // Association - solid line
        Self::icon_tool_button(ui, app, EditMode::AddAssociation, "Add association", |painter, rect| {
            let left = Pos2::new(rect.left() + 4.0, rect.center().y);
            let right = Pos2::new(rect.right() - 4.0, rect.center().y);
            painter.line_segment([left, right], Stroke::new(1.5, Color32::BLACK));
        });

        // Include - dashed arrow with <<include>>
        Self::icon_tool_button(ui, app, EditMode::AddInclude, "Add include relationship", |painter, rect| {
            let left = Pos2::new(rect.left() + 3.0, rect.center().y);
            let right = Pos2::new(rect.right() - 3.0, rect.center().y);
            // Dashed line
            let mut x = left.x;
            while x < right.x - 5.0 {
                let end_x = (x + 3.0).min(right.x - 5.0);
                painter.line_segment(
                    [Pos2::new(x, rect.center().y), Pos2::new(end_x, rect.center().y)],
                    Stroke::new(1.0, Color32::BLACK),
                );
                x += 5.0;
            }
            // Arrow
            painter.line_segment([right, Pos2::new(right.x - 4.0, right.y - 3.0)], Stroke::new(1.0, Color32::BLACK));
            painter.line_segment([right, Pos2::new(right.x - 4.0, right.y + 3.0)], Stroke::new(1.0, Color32::BLACK));
        });

        // Extend - dashed arrow (reversed)
        Self::icon_tool_button(ui, app, EditMode::AddExtend, "Add extend relationship", |painter, rect| {
            let left = Pos2::new(rect.left() + 3.0, rect.center().y);
            let right = Pos2::new(rect.right() - 3.0, rect.center().y);
            // Dashed line
            let mut x = left.x + 5.0;
            while x < right.x {
                let end_x = (x + 3.0).min(right.x);
                painter.line_segment(
                    [Pos2::new(x, rect.center().y), Pos2::new(end_x, rect.center().y)],
                    Stroke::new(1.0, Color32::BLACK),
                );
                x += 5.0;
            }
            // Arrow pointing left
            painter.line_segment([left, Pos2::new(left.x + 4.0, left.y - 3.0)], Stroke::new(1.0, Color32::BLACK));
            painter.line_segment([left, Pos2::new(left.x + 4.0, left.y + 3.0)], Stroke::new(1.0, Color32::BLACK));
        });

        // Generalization - hollow triangle arrow
        Self::icon_tool_button(ui, app, EditMode::AddGeneralization, "Add generalization", |painter, rect| {
            let left = Pos2::new(rect.left() + 3.0, rect.center().y);
            let right = Pos2::new(rect.right() - 3.0, rect.center().y);
            painter.line_segment([left, Pos2::new(right.x - 6.0, right.y)], Stroke::new(1.5, Color32::BLACK));
            // Hollow triangle
            let triangle = vec![
                right,
                Pos2::new(right.x - 6.0, right.y - 4.0),
                Pos2::new(right.x - 6.0, right.y + 4.0),
            ];
            painter.add(egui::Shape::convex_polygon(triangle, Color32::WHITE, Stroke::new(1.0, Color32::BLACK)));
        });
    }

    fn show_activity_tools(ui: &mut egui::Ui, app: &mut JmtApp) {
        ui.label("Add:");

        // Action - rounded rectangle
        Self::icon_tool_button(ui, app, EditMode::AddAction, "Add an action", |painter, rect| {
            let inner = rect.shrink(3.0);
            painter.rect(inner, Rounding::same(5.0), Color32::from_rgb(200, 230, 255), Stroke::new(1.0, Color32::BLACK));
        });

        // Initial - filled circle
        Self::icon_tool_button(ui, app, EditMode::AddInitial, "Add initial node", |painter, rect| {
            painter.circle_filled(rect.center(), rect.width() / 3.0, Color32::BLACK);
        });

        // Final - bullseye (double circle)
        Self::icon_tool_button(ui, app, EditMode::AddFinal, "Add final node", |painter, rect| {
            let center = rect.center();
            let r = rect.width() / 3.0;
            painter.circle_stroke(center, r, Stroke::new(1.5, Color32::BLACK));
            painter.circle_filled(center, r * 0.5, Color32::BLACK);
        });

        // Decision - diamond
        Self::icon_tool_button(ui, app, EditMode::AddDecision, "Add decision/merge node", |painter, rect| {
            let center = rect.center();
            let s = rect.width() / 3.0;
            let points = vec![
                Pos2::new(center.x, center.y - s),
                Pos2::new(center.x + s, center.y),
                Pos2::new(center.x, center.y + s),
                Pos2::new(center.x - s, center.y),
            ];
            painter.add(egui::Shape::convex_polygon(points, Color32::WHITE, Stroke::new(1.0, Color32::BLACK)));
        });

        // Fork - horizontal bar
        Self::icon_tool_button(ui, app, EditMode::AddFork, "Add fork bar", |painter, rect| {
            let center = rect.center();
            let w = rect.width() * 0.7;
            let bar = egui::Rect::from_center_size(center, Vec2::new(w, 4.0));
            painter.rect_filled(bar, Rounding::ZERO, Color32::BLACK);
        });

        // Join - horizontal bar (same as fork)
        Self::icon_tool_button(ui, app, EditMode::AddJoin, "Add join bar", |painter, rect| {
            let center = rect.center();
            let w = rect.width() * 0.7;
            let bar = egui::Rect::from_center_size(center, Vec2::new(w, 4.0));
            painter.rect_filled(bar, Rounding::ZERO, Color32::BLACK);
        });

        ui.separator();
        ui.label("Signals:");

        // Send Signal - pentagon pointing right
        Self::icon_tool_button(ui, app, EditMode::AddSendSignal, "Add send signal action", |painter, rect| {
            let inner = rect.shrink(3.0);
            let points = vec![
                Pos2::new(inner.left(), inner.top()),
                Pos2::new(inner.right() - 4.0, inner.top()),
                Pos2::new(inner.right(), inner.center().y),
                Pos2::new(inner.right() - 4.0, inner.bottom()),
                Pos2::new(inner.left(), inner.bottom()),
            ];
            painter.add(egui::Shape::convex_polygon(points, Color32::from_rgb(255, 230, 200), Stroke::new(1.0, Color32::BLACK)));
        });

        // Accept Event - concave pentagon (notch on left)
        Self::icon_tool_button(ui, app, EditMode::AddAcceptEvent, "Add accept event action", |painter, rect| {
            let inner = rect.shrink(3.0);
            let points = vec![
                Pos2::new(inner.left(), inner.top()),
                Pos2::new(inner.right(), inner.top()),
                Pos2::new(inner.right(), inner.bottom()),
                Pos2::new(inner.left(), inner.bottom()),
                Pos2::new(inner.left() + 4.0, inner.center().y),
            ];
            painter.add(egui::Shape::convex_polygon(points, Color32::from_rgb(200, 255, 200), Stroke::new(1.0, Color32::BLACK)));
        });

        // Time Event - hourglass
        Self::icon_tool_button(ui, app, EditMode::AddTimeEvent, "Add time event action", |painter, rect| {
            let center = rect.center();
            let hw = 5.0;
            let hh = 7.0;
            let points = vec![
                Pos2::new(center.x - hw, center.y - hh),
                Pos2::new(center.x + hw, center.y - hh),
                Pos2::new(center.x - hw, center.y + hh),
                Pos2::new(center.x + hw, center.y + hh),
            ];
            painter.line_segment([points[0], points[2]], Stroke::new(1.0, Color32::BLACK));
            painter.line_segment([points[1], points[3]], Stroke::new(1.0, Color32::BLACK));
            painter.line_segment([points[0], points[1]], Stroke::new(1.0, Color32::BLACK));
            painter.line_segment([points[2], points[3]], Stroke::new(1.0, Color32::BLACK));
        });

        ui.separator();
        ui.label("Objects:");

        // Object Node - rectangle
        Self::icon_tool_button(ui, app, EditMode::AddObjectNode, "Add object node", |painter, rect| {
            let inner = rect.shrink(3.0);
            painter.rect(inner, Rounding::ZERO, Color32::WHITE, Stroke::new(1.0, Color32::BLACK));
        });

        // Data Store - rectangle with double lines
        Self::icon_tool_button(ui, app, EditMode::AddDataStore, "Add data store", |painter, rect| {
            let inner = rect.shrink(3.0);
            painter.rect(inner, Rounding::ZERO, Color32::WHITE, Stroke::new(1.0, Color32::BLACK));
            // Double lines on sides
            painter.line_segment(
                [Pos2::new(inner.left() + 2.0, inner.top()), Pos2::new(inner.left() + 2.0, inner.bottom())],
                Stroke::new(1.0, Color32::BLACK),
            );
            painter.line_segment(
                [Pos2::new(inner.right() - 2.0, inner.top()), Pos2::new(inner.right() - 2.0, inner.bottom())],
                Stroke::new(1.0, Color32::BLACK),
            );
        });

        // Swimlane - vertical partition
        Self::icon_tool_button(ui, app, EditMode::AddSwimlane, "Add swimlane", |painter, rect| {
            let inner = rect.shrink(2.0);
            painter.rect(inner, Rounding::ZERO, Color32::from_rgba_unmultiplied(230, 230, 255, 200), Stroke::new(1.0, Color32::BLACK));
            // Vertical divider
            painter.line_segment(
                [Pos2::new(inner.center().x, inner.top()), Pos2::new(inner.center().x, inner.bottom())],
                Stroke::new(1.0, Color32::GRAY),
            );
            // Header area
            painter.line_segment(
                [Pos2::new(inner.left(), inner.top() + 5.0), Pos2::new(inner.right(), inner.top() + 5.0)],
                Stroke::new(0.5, Color32::GRAY),
            );
        });

        ui.separator();
        ui.label("Connect:");

        // Control Flow - arrow
        Self::icon_tool_button(ui, app, EditMode::Connect, "Create control flow", |painter, rect| {
            let left = Pos2::new(rect.left() + 4.0, rect.center().y);
            let right = Pos2::new(rect.right() - 4.0, rect.center().y);
            painter.line_segment([left, right], Stroke::new(1.5, Color32::BLACK));
            // Arrowhead
            painter.line_segment([right, Pos2::new(right.x - 5.0, right.y - 4.0)], Stroke::new(1.5, Color32::BLACK));
            painter.line_segment([right, Pos2::new(right.x - 5.0, right.y + 4.0)], Stroke::new(1.5, Color32::BLACK));
        });
    }

    fn align_nodes(app: &mut JmtApp, mode: AlignMode) {
        if let Some(state) = app.current_diagram_mut() {
            let selected_ids = state.diagram.selected_nodes();
            if selected_ids.len() < 2 {
                return;
            }

            state.diagram.push_undo();

            // Collect bounds of selected nodes
            let bounds: Vec<_> = selected_ids.iter()
                .filter_map(|id| state.diagram.find_node(*id))
                .map(|n| n.bounds().clone())
                .collect();

            if bounds.is_empty() {
                return;
            }

            // Calculate alignment target
            let target = match mode {
                AlignMode::Left => bounds.iter().map(|b| b.x1).fold(f32::MAX, f32::min),
                AlignMode::Right => bounds.iter().map(|b| b.x2).fold(f32::MIN, f32::max),
                AlignMode::CenterH => {
                    let min_x = bounds.iter().map(|b| b.x1).fold(f32::MAX, f32::min);
                    let max_x = bounds.iter().map(|b| b.x2).fold(f32::MIN, f32::max);
                    (min_x + max_x) / 2.0
                }
                AlignMode::Top => bounds.iter().map(|b| b.y1).fold(f32::MAX, f32::min),
                AlignMode::Bottom => bounds.iter().map(|b| b.y2).fold(f32::MIN, f32::max),
                AlignMode::CenterV => {
                    let min_y = bounds.iter().map(|b| b.y1).fold(f32::MAX, f32::min);
                    let max_y = bounds.iter().map(|b| b.y2).fold(f32::MIN, f32::max);
                    (min_y + max_y) / 2.0
                }
            };

            // Apply alignment
            for id in selected_ids {
                if let Some(node) = state.diagram.find_node_mut(id) {
                    let bounds = node.bounds();
                    let offset = match mode {
                        AlignMode::Left => target - bounds.x1,
                        AlignMode::Right => target - bounds.x2,
                        AlignMode::CenterH => target - bounds.center().x,
                        AlignMode::Top => target - bounds.y1,
                        AlignMode::Bottom => target - bounds.y2,
                        AlignMode::CenterV => target - bounds.center().y,
                    };

                    match mode {
                        AlignMode::Left | AlignMode::Right | AlignMode::CenterH => {
                            node.translate(offset, 0.0);
                        }
                        AlignMode::Top | AlignMode::Bottom | AlignMode::CenterV => {
                            node.translate(0.0, offset);
                        }
                    }
                }
            }

            state.diagram.recalculate_connections();
            state.modified = true;
        }
    }

    fn distribute_nodes(app: &mut JmtApp, mode: DistributeMode) {
        if let Some(state) = app.current_diagram_mut() {
            let selected_ids = state.diagram.selected_nodes();
            if selected_ids.len() < 3 {
                return; // Need at least 3 nodes to distribute
            }

            state.diagram.push_undo();

            // Collect node IDs with their center positions
            let mut nodes_with_pos: Vec<_> = selected_ids.iter()
                .filter_map(|id| {
                    state.diagram.find_node(*id).map(|n| {
                        let center = n.bounds().center();
                        (*id, center.x, center.y)
                    })
                })
                .collect();

            if nodes_with_pos.len() < 3 {
                return;
            }

            match mode {
                DistributeMode::Horizontal => {
                    // Sort by x position
                    nodes_with_pos.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

                    let first_x = nodes_with_pos.first().unwrap().1;
                    let last_x = nodes_with_pos.last().unwrap().1;
                    let count = nodes_with_pos.len();
                    let spacing = (last_x - first_x) / (count - 1) as f32;

                    for (i, (id, current_x, _)) in nodes_with_pos.iter().enumerate() {
                        let target_x = first_x + spacing * i as f32;
                        let offset = target_x - current_x;
                        if let Some(node) = state.diagram.find_node_mut(*id) {
                            node.translate(offset, 0.0);
                        }
                    }
                }
                DistributeMode::Vertical => {
                    // Sort by y position
                    nodes_with_pos.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

                    let first_y = nodes_with_pos.first().unwrap().2;
                    let last_y = nodes_with_pos.last().unwrap().2;
                    let count = nodes_with_pos.len();
                    let spacing = (last_y - first_y) / (count - 1) as f32;

                    for (i, (id, _, current_y)) in nodes_with_pos.iter().enumerate() {
                        let target_y = first_y + spacing * i as f32;
                        let offset = target_y - current_y;
                        if let Some(node) = state.diagram.find_node_mut(*id) {
                            node.translate(0.0, offset);
                        }
                    }
                }
            }

            state.diagram.recalculate_connections();
            state.modified = true;
        }
    }
}

#[derive(Copy, Clone)]
enum AlignMode {
    Left,
    Right,
    CenterH,
    Top,
    Bottom,
    CenterV,
}

#[derive(Copy, Clone)]
enum DistributeMode {
    Horizontal,
    Vertical,
}
