//! Toolbar panel with graphical icons

use eframe::egui::{self, Color32, Pos2, Rounding, Stroke, Vec2};
use jmt_core::{EditMode, DiagramType, NodeId};
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

            // Lasso selection - freeform lasso icon
            Self::icon_tool_button(ui, app, EditMode::Lasso, "Lasso select - draw around elements to select", |painter, rect, stroke_color| {
                // Draw a lasso loop shape
                let cx = rect.center().x;
                let cy = rect.center().y;
                let r = rect.width() / 2.5;

                // Draw a curved lasso path
                let points: Vec<Pos2> = (0..=12).map(|i| {
                    let t = i as f32 / 12.0;
                    let angle = t * std::f32::consts::PI * 1.8 - std::f32::consts::PI * 0.4;
                    let wobble = (t * std::f32::consts::PI * 4.0).sin() * 0.15;
                    let radius = r * (1.0 + wobble);
                    Pos2::new(cx + radius * angle.cos(), cy + radius * angle.sin())
                }).collect();

                painter.add(egui::Shape::line(points, Stroke::new(1.5, stroke_color)));

                // Add a small circle at the end (lasso tip)
                let end_x = cx + r * 0.9;
                let end_y = cy - r * 0.6;
                painter.circle_filled(Pos2::new(end_x, end_y), 2.0, stroke_color);
            });

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
                    if ui.button("⫿ Align Vertically").clicked() {
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
                    if ui.button("⫟ Align Horizontally").clicked() {
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

            ui.separator();

            // Zoom controls
            ui.label("Zoom:");
            if ui.button("−").on_hover_text("Zoom out (Ctrl+Scroll down)").clicked() {
                app.zoom_out();
            }

            // Show current zoom level as a clickable button to reset
            let zoom_text = format!("{:.0}%", app.zoom_level * 100.0);
            if ui.button(&zoom_text).on_hover_text("Reset zoom to 100%").clicked() {
                app.reset_zoom();
            }

            if ui.button("+").on_hover_text("Zoom in (Ctrl+Scroll up)").clicked() {
                app.zoom_in();
            }
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
    /// The draw_icon closure receives (painter, rect, stroke_color) where stroke_color
    /// is theme-aware (dark in light mode, light in dark mode)
    fn icon_tool_button(
        ui: &mut egui::Ui,
        app: &mut JmtApp,
        mode: EditMode,
        tooltip: &str,
        draw_icon: impl FnOnce(&egui::Painter, egui::Rect, Color32),
    ) {
        let current_mode = app.edit_mode == mode;

        // Get theme-aware colors
        let is_dark_mode = ui.visuals().dark_mode;
        let stroke_color = if is_dark_mode {
            Color32::from_rgb(220, 220, 220) // Light color for dark mode
        } else {
            Color32::BLACK // Dark color for light mode
        };

        let size = Vec2::splat(ICON_SIZE + 8.0);
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

        // Draw button background (theme-aware)
        let bg_color = if current_mode {
            if is_dark_mode {
                Color32::from_rgb(60, 80, 120) // Darker blue for dark mode
            } else {
                Color32::from_rgb(200, 220, 255) // Light blue for light mode
            }
        } else if response.hovered() {
            if is_dark_mode {
                Color32::from_rgb(70, 70, 70) // Dark grey for dark mode hover
            } else {
                Color32::from_rgb(230, 230, 230) // Light grey for light mode hover
            }
        } else {
            Color32::TRANSPARENT
        };

        ui.painter().rect(rect, Rounding::same(4.0), bg_color, Stroke::NONE);

        // Draw the icon with theme-aware color
        let icon_rect = rect.shrink(4.0);
        draw_icon(ui.painter(), icon_rect, stroke_color);

        // Handle click
        if response.on_hover_text(tooltip).clicked() {
            app.set_edit_mode(mode);
        }
    }

    fn show_state_machine_tools(ui: &mut egui::Ui, app: &mut JmtApp) {
        ui.label("Add:");

        // State - rounded rectangle
        Self::icon_tool_button(ui, app, EditMode::AddState, "Add a state", |painter, rect, stroke_color| {
            let inner = rect.shrink(2.0);
            painter.rect(inner, Rounding::same(4.0), Color32::from_rgb(255, 255, 204), Stroke::new(1.0, stroke_color));
        });

        // Initial - filled circle
        Self::icon_tool_button(ui, app, EditMode::AddInitial, "Add initial pseudo-state", |painter, rect, stroke_color| {
            painter.circle_filled(rect.center(), rect.width() / 3.0, stroke_color);
        });

        // Final - double circle
        Self::icon_tool_button(ui, app, EditMode::AddFinal, "Add final pseudo-state", |painter, rect, stroke_color| {
            let center = rect.center();
            let r = rect.width() / 3.0;
            painter.circle_stroke(center, r, Stroke::new(1.0, stroke_color));
            painter.circle_filled(center, r * 0.6, stroke_color);
        });

        // Choice - diamond
        Self::icon_tool_button(ui, app, EditMode::AddChoice, "Add choice pseudo-state", |painter, rect, stroke_color| {
            let center = rect.center();
            let s = rect.width() / 3.0;
            let points = vec![
                Pos2::new(center.x, center.y - s),
                Pos2::new(center.x + s, center.y),
                Pos2::new(center.x, center.y + s),
                Pos2::new(center.x - s, center.y),
            ];
            painter.add(egui::Shape::convex_polygon(points, Color32::TRANSPARENT, Stroke::new(1.0, stroke_color)));
        });

        // Junction - filled diamond
        Self::icon_tool_button(ui, app, EditMode::AddJunction, "Add junction pseudo-state", |painter, rect, stroke_color| {
            let center = rect.center();
            let s = rect.width() / 3.5;
            let points = vec![
                Pos2::new(center.x, center.y - s),
                Pos2::new(center.x + s, center.y),
                Pos2::new(center.x, center.y + s),
                Pos2::new(center.x - s, center.y),
            ];
            painter.add(egui::Shape::convex_polygon(points, stroke_color, Stroke::new(1.0, stroke_color)));
        });

        // Fork - horizontal bar
        Self::icon_tool_button(ui, app, EditMode::AddFork, "Add fork pseudo-state", |painter, rect, stroke_color| {
            let center = rect.center();
            let w = rect.width() * 0.7;
            let h = 4.0;
            let bar = egui::Rect::from_center_size(center, Vec2::new(w, h));
            painter.rect_filled(bar, Rounding::ZERO, stroke_color);
        });

        // Join - horizontal bar (same as fork)
        Self::icon_tool_button(ui, app, EditMode::AddJoin, "Add join pseudo-state", |painter, rect, stroke_color| {
            let center = rect.center();
            let w = rect.width() * 0.7;
            let h = 4.0;
            let bar = egui::Rect::from_center_size(center, Vec2::new(w, h));
            painter.rect_filled(bar, Rounding::ZERO, stroke_color);
        });

        ui.separator();
        ui.label("Connect:");

        // Transition - arrow
        Self::icon_tool_button(ui, app, EditMode::Connect, "Create a transition", |painter, rect, stroke_color| {
            let left = Pos2::new(rect.left() + 4.0, rect.center().y);
            let right = Pos2::new(rect.right() - 4.0, rect.center().y);
            painter.line_segment([left, right], Stroke::new(1.5, stroke_color));
            // Arrowhead
            let arrow_size = 5.0;
            painter.line_segment([right, Pos2::new(right.x - arrow_size, right.y - arrow_size)], Stroke::new(1.5, stroke_color));
            painter.line_segment([right, Pos2::new(right.x - arrow_size, right.y + arrow_size)], Stroke::new(1.5, stroke_color));
        });

        ui.separator();

        // Add Region button - only enabled when a state is selected
        Self::add_region_button(ui, app);
    }

    /// Add Region button - adds a region to the selected state
    fn add_region_button(ui: &mut egui::Ui, app: &mut JmtApp) {
        // Check if exactly one state is selected
        let selected_state_id = app.current_diagram().and_then(|s| {
            let selected = s.diagram.selected_nodes();
            if selected.len() == 1 {
                let id = selected[0];
                // Check if it's a State (not a PseudoState)
                if let Some(jmt_core::Node::State(_)) = s.diagram.find_node(id) {
                    return Some(id);
                }
            }
            None
        });

        let is_enabled = selected_state_id.is_some();
        let is_dark_mode = ui.visuals().dark_mode;
        let stroke_color = if is_dark_mode {
            if is_enabled {
                Color32::from_rgb(220, 220, 220)
            } else {
                Color32::from_rgb(100, 100, 100)
            }
        } else {
            if is_enabled {
                Color32::BLACK
            } else {
                Color32::from_rgb(180, 180, 180)
            }
        };

        let size = Vec2::splat(ICON_SIZE + 8.0);
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

        // Draw button background
        let bg_color = if !is_enabled {
            Color32::TRANSPARENT
        } else if response.hovered() {
            if is_dark_mode {
                Color32::from_rgb(70, 70, 70)
            } else {
                Color32::from_rgb(230, 230, 230)
            }
        } else {
            Color32::TRANSPARENT
        };

        ui.painter().rect(rect, Rounding::same(4.0), bg_color, Stroke::NONE);

        // Draw icon: state with dashed region separator
        let icon_rect = rect.shrink(4.0);
        let inner = icon_rect.shrink(2.0);

        // State rectangle
        ui.painter().rect(inner, Rounding::same(3.0), Color32::from_rgb(255, 255, 204), Stroke::new(1.0, stroke_color));

        // Dashed horizontal line (region separator)
        let y = inner.center().y;
        let mut x = inner.left() + 2.0;
        while x < inner.right() - 2.0 {
            let end_x = (x + 3.0).min(inner.right() - 2.0);
            ui.painter().line_segment(
                [Pos2::new(x, y), Pos2::new(end_x, y)],
                Stroke::new(1.0, stroke_color),
            );
            x += 5.0;
        }

        // Handle click
        let tooltip = if is_enabled {
            "Add region to selected state"
        } else {
            "Select a state to add a region"
        };

        if response.on_hover_text(tooltip).clicked() && is_enabled {
            if let Some(state_id) = selected_state_id {
                if let Some(diagram_state) = app.current_diagram_mut() {
                    diagram_state.diagram.push_undo();
                    if let Some(jmt_core::Node::State(state)) = diagram_state.diagram.find_node_mut(state_id) {
                        state.add_region("Region");
                    }
                    diagram_state.modified = true;
                }
            }
        }
    }

    fn show_sequence_tools(ui: &mut egui::Ui, app: &mut JmtApp) {
        ui.label("Add:");

        // Lifeline - rectangle head with dashed line
        Self::icon_tool_button(ui, app, EditMode::AddLifeline, "Add a lifeline", |painter, rect, stroke_color| {
            let head_rect = egui::Rect::from_min_size(
                Pos2::new(rect.center().x - 6.0, rect.top() + 2.0),
                Vec2::new(12.0, 8.0),
            );
            painter.rect(head_rect, Rounding::same(1.0), Color32::from_rgb(255, 255, 240), Stroke::new(1.0, stroke_color));
            // Dashed line
            let line_top = head_rect.bottom();
            let line_bottom = rect.bottom() - 2.0;
            let mut y = line_top;
            while y < line_bottom {
                let end_y = (y + 3.0).min(line_bottom);
                painter.line_segment(
                    [Pos2::new(rect.center().x, y), Pos2::new(rect.center().x, end_y)],
                    Stroke::new(1.0, stroke_color),
                );
                y += 5.0;
            }
        });

        // Activation - filled rectangle
        Self::icon_tool_button(ui, app, EditMode::AddActivation, "Add an activation box", |painter, rect, stroke_color| {
            let act_rect = egui::Rect::from_center_size(rect.center(), Vec2::new(8.0, 14.0));
            painter.rect(act_rect, Rounding::ZERO, Color32::from_rgb(255, 255, 240), Stroke::new(1.0, stroke_color));
        });

        // Fragment - box with pentagon
        Self::icon_tool_button(ui, app, EditMode::AddFragment, "Add a combined fragment", |painter, rect, stroke_color| {
            let inner = rect.shrink(3.0);
            painter.rect(inner, Rounding::ZERO, Color32::from_rgba_unmultiplied(240, 240, 255, 200), Stroke::new(1.0, stroke_color));
            // Pentagon tag
            let tag = vec![
                Pos2::new(inner.left(), inner.top()),
                Pos2::new(inner.left() + 10.0, inner.top()),
                Pos2::new(inner.left() + 12.0, inner.top() + 4.0),
                Pos2::new(inner.left() + 10.0, inner.top() + 8.0),
                Pos2::new(inner.left(), inner.top() + 8.0),
            ];
            painter.add(egui::Shape::convex_polygon(tag, Color32::from_rgb(255, 255, 240), Stroke::new(1.0, stroke_color)));
        });

        ui.separator();
        ui.label("Messages:");

        // Sync message - solid arrow
        Self::icon_tool_button(ui, app, EditMode::AddSyncMessage, "Add synchronous message", |painter, rect, stroke_color| {
            let left = Pos2::new(rect.left() + 3.0, rect.center().y);
            let right = Pos2::new(rect.right() - 3.0, rect.center().y);
            painter.line_segment([left, right], Stroke::new(1.5, stroke_color));
            // Filled arrowhead
            let arrow = vec![
                right,
                Pos2::new(right.x - 6.0, right.y - 4.0),
                Pos2::new(right.x - 6.0, right.y + 4.0),
            ];
            painter.add(egui::Shape::convex_polygon(arrow, stroke_color, Stroke::NONE));
        });

        // Async message - open arrow
        Self::icon_tool_button(ui, app, EditMode::AddAsyncMessage, "Add asynchronous message", |painter, rect, stroke_color| {
            let left = Pos2::new(rect.left() + 3.0, rect.center().y);
            let right = Pos2::new(rect.right() - 3.0, rect.center().y);
            painter.line_segment([left, right], Stroke::new(1.5, stroke_color));
            // Open arrowhead
            painter.line_segment([right, Pos2::new(right.x - 5.0, right.y - 4.0)], Stroke::new(1.5, stroke_color));
            painter.line_segment([right, Pos2::new(right.x - 5.0, right.y + 4.0)], Stroke::new(1.5, stroke_color));
        });

        // Return message - dashed with open arrow
        Self::icon_tool_button(ui, app, EditMode::AddReturnMessage, "Add return message", |painter, rect, stroke_color| {
            let left = Pos2::new(rect.left() + 3.0, rect.center().y);
            let right = Pos2::new(rect.right() - 3.0, rect.center().y);
            // Dashed line
            let mut x = left.x;
            while x < right.x - 6.0 {
                let end_x = (x + 4.0).min(right.x - 6.0);
                painter.line_segment(
                    [Pos2::new(x, rect.center().y), Pos2::new(end_x, rect.center().y)],
                    Stroke::new(1.0, stroke_color),
                );
                x += 6.0;
            }
            // Arrow pointing left
            painter.line_segment([left, Pos2::new(left.x + 5.0, left.y - 4.0)], Stroke::new(1.5, stroke_color));
            painter.line_segment([left, Pos2::new(left.x + 5.0, left.y + 4.0)], Stroke::new(1.5, stroke_color));
        });

        // Self message - loop back arrow
        Self::icon_tool_button(ui, app, EditMode::AddSelfMessage, "Add self message", |painter, rect, stroke_color| {
            let start = Pos2::new(rect.center().x, rect.top() + 4.0);
            let mid_right = Pos2::new(rect.right() - 4.0, rect.top() + 4.0);
            let mid_right_bottom = Pos2::new(rect.right() - 4.0, rect.bottom() - 4.0);
            let end = Pos2::new(rect.center().x, rect.bottom() - 4.0);
            let stroke = Stroke::new(1.0, stroke_color);
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
        Self::icon_tool_button(ui, app, EditMode::AddActor, "Add an actor", |painter, rect, stroke_color| {
            let center_x = rect.center().x;
            let head_y = rect.top() + 4.0;
            let head_r = 3.0;
            let body_top = head_y + head_r + 1.0;
            let body_bottom = body_top + 6.0;
            let arm_y = body_top + 2.0;
            let leg_bottom = rect.bottom() - 2.0;

            let stroke = Stroke::new(1.0, stroke_color);
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
        Self::icon_tool_button(ui, app, EditMode::AddUseCase, "Add a use case", |painter, rect, stroke_color| {
            let center = rect.center();
            let radius = Vec2::new(rect.width() / 2.5, rect.height() / 3.0);
            painter.add(egui::Shape::ellipse_filled(center, radius, Color32::from_rgb(255, 255, 220)));
            painter.add(egui::Shape::ellipse_stroke(center, radius, Stroke::new(1.0, stroke_color)));
        });

        // System Boundary - rectangle with header
        Self::icon_tool_button(ui, app, EditMode::AddSystemBoundary, "Add system boundary", |painter, rect, stroke_color| {
            let inner = rect.shrink(3.0);
            painter.rect(inner, Rounding::same(2.0), Color32::from_rgba_unmultiplied(245, 245, 245, 200), Stroke::new(1.0, stroke_color));
            // Header line
            painter.line_segment(
                [Pos2::new(inner.left(), inner.top() + 6.0), Pos2::new(inner.right(), inner.top() + 6.0)],
                Stroke::new(0.5, Color32::GRAY),
            );
        });

        ui.separator();
        ui.label("Connect:");

        // Association - solid line
        Self::icon_tool_button(ui, app, EditMode::AddAssociation, "Add association", |painter, rect, stroke_color| {
            let left = Pos2::new(rect.left() + 4.0, rect.center().y);
            let right = Pos2::new(rect.right() - 4.0, rect.center().y);
            painter.line_segment([left, right], Stroke::new(1.5, stroke_color));
        });

        // Include - dashed arrow with <<include>>
        Self::icon_tool_button(ui, app, EditMode::AddInclude, "Add include relationship", |painter, rect, stroke_color| {
            let left = Pos2::new(rect.left() + 3.0, rect.center().y);
            let right = Pos2::new(rect.right() - 3.0, rect.center().y);
            // Dashed line
            let mut x = left.x;
            while x < right.x - 5.0 {
                let end_x = (x + 3.0).min(right.x - 5.0);
                painter.line_segment(
                    [Pos2::new(x, rect.center().y), Pos2::new(end_x, rect.center().y)],
                    Stroke::new(1.0, stroke_color),
                );
                x += 5.0;
            }
            // Arrow
            painter.line_segment([right, Pos2::new(right.x - 4.0, right.y - 3.0)], Stroke::new(1.0, stroke_color));
            painter.line_segment([right, Pos2::new(right.x - 4.0, right.y + 3.0)], Stroke::new(1.0, stroke_color));
        });

        // Extend - dashed arrow (reversed)
        Self::icon_tool_button(ui, app, EditMode::AddExtend, "Add extend relationship", |painter, rect, stroke_color| {
            let left = Pos2::new(rect.left() + 3.0, rect.center().y);
            let right = Pos2::new(rect.right() - 3.0, rect.center().y);
            // Dashed line
            let mut x = left.x + 5.0;
            while x < right.x {
                let end_x = (x + 3.0).min(right.x);
                painter.line_segment(
                    [Pos2::new(x, rect.center().y), Pos2::new(end_x, rect.center().y)],
                    Stroke::new(1.0, stroke_color),
                );
                x += 5.0;
            }
            // Arrow pointing left
            painter.line_segment([left, Pos2::new(left.x + 4.0, left.y - 3.0)], Stroke::new(1.0, stroke_color));
            painter.line_segment([left, Pos2::new(left.x + 4.0, left.y + 3.0)], Stroke::new(1.0, stroke_color));
        });

        // Generalization - hollow triangle arrow
        Self::icon_tool_button(ui, app, EditMode::AddGeneralization, "Add generalization", |painter, rect, stroke_color| {
            let left = Pos2::new(rect.left() + 3.0, rect.center().y);
            let right = Pos2::new(rect.right() - 3.0, rect.center().y);
            painter.line_segment([left, Pos2::new(right.x - 6.0, right.y)], Stroke::new(1.5, stroke_color));
            // Hollow triangle
            let triangle = vec![
                right,
                Pos2::new(right.x - 6.0, right.y - 4.0),
                Pos2::new(right.x - 6.0, right.y + 4.0),
            ];
            painter.add(egui::Shape::convex_polygon(triangle, Color32::from_rgb(255, 255, 240), Stroke::new(1.0, stroke_color)));
        });
    }

    fn show_activity_tools(ui: &mut egui::Ui, app: &mut JmtApp) {
        ui.label("Add:");

        // Action - rounded rectangle
        Self::icon_tool_button(ui, app, EditMode::AddAction, "Add an action", |painter, rect, stroke_color| {
            let inner = rect.shrink(3.0);
            painter.rect(inner, Rounding::same(5.0), Color32::from_rgb(200, 230, 255), Stroke::new(1.0, stroke_color));
        });

        // Initial - filled circle
        Self::icon_tool_button(ui, app, EditMode::AddInitial, "Add initial node", |painter, rect, stroke_color| {
            painter.circle_filled(rect.center(), rect.width() / 3.0, stroke_color);
        });

        // Final - bullseye (double circle)
        Self::icon_tool_button(ui, app, EditMode::AddFinal, "Add final node", |painter, rect, stroke_color| {
            let center = rect.center();
            let r = rect.width() / 3.0;
            painter.circle_stroke(center, r, Stroke::new(1.5, stroke_color));
            painter.circle_filled(center, r * 0.5, stroke_color);
        });

        // Decision - diamond
        Self::icon_tool_button(ui, app, EditMode::AddDecision, "Add decision/merge node", |painter, rect, stroke_color| {
            let center = rect.center();
            let s = rect.width() / 3.0;
            let points = vec![
                Pos2::new(center.x, center.y - s),
                Pos2::new(center.x + s, center.y),
                Pos2::new(center.x, center.y + s),
                Pos2::new(center.x - s, center.y),
            ];
            painter.add(egui::Shape::convex_polygon(points, Color32::TRANSPARENT, Stroke::new(1.0, stroke_color)));
        });

        // Fork - horizontal bar
        Self::icon_tool_button(ui, app, EditMode::AddFork, "Add fork bar", |painter, rect, stroke_color| {
            let center = rect.center();
            let w = rect.width() * 0.7;
            let bar = egui::Rect::from_center_size(center, Vec2::new(w, 4.0));
            painter.rect_filled(bar, Rounding::ZERO, stroke_color);
        });

        // Join - horizontal bar (same as fork)
        Self::icon_tool_button(ui, app, EditMode::AddJoin, "Add join bar", |painter, rect, stroke_color| {
            let center = rect.center();
            let w = rect.width() * 0.7;
            let bar = egui::Rect::from_center_size(center, Vec2::new(w, 4.0));
            painter.rect_filled(bar, Rounding::ZERO, stroke_color);
        });

        ui.separator();
        ui.label("Signals:");

        // Send Signal - pentagon pointing right
        Self::icon_tool_button(ui, app, EditMode::AddSendSignal, "Add send signal action", |painter, rect, stroke_color| {
            let inner = rect.shrink(3.0);
            let points = vec![
                Pos2::new(inner.left(), inner.top()),
                Pos2::new(inner.right() - 4.0, inner.top()),
                Pos2::new(inner.right(), inner.center().y),
                Pos2::new(inner.right() - 4.0, inner.bottom()),
                Pos2::new(inner.left(), inner.bottom()),
            ];
            painter.add(egui::Shape::convex_polygon(points, Color32::from_rgb(255, 230, 200), Stroke::new(1.0, stroke_color)));
        });

        // Accept Event - concave pentagon (notch on left)
        Self::icon_tool_button(ui, app, EditMode::AddAcceptEvent, "Add accept event action", |painter, rect, stroke_color| {
            let inner = rect.shrink(3.0);
            let points = vec![
                Pos2::new(inner.left(), inner.top()),
                Pos2::new(inner.right(), inner.top()),
                Pos2::new(inner.right(), inner.bottom()),
                Pos2::new(inner.left(), inner.bottom()),
                Pos2::new(inner.left() + 4.0, inner.center().y),
            ];
            painter.add(egui::Shape::convex_polygon(points, Color32::from_rgb(200, 255, 200), Stroke::new(1.0, stroke_color)));
        });

        // Time Event - hourglass
        Self::icon_tool_button(ui, app, EditMode::AddTimeEvent, "Add time event action", |painter, rect, stroke_color| {
            let center = rect.center();
            let hw = 5.0;
            let hh = 7.0;
            let points = vec![
                Pos2::new(center.x - hw, center.y - hh),
                Pos2::new(center.x + hw, center.y - hh),
                Pos2::new(center.x - hw, center.y + hh),
                Pos2::new(center.x + hw, center.y + hh),
            ];
            painter.line_segment([points[0], points[2]], Stroke::new(1.0, stroke_color));
            painter.line_segment([points[1], points[3]], Stroke::new(1.0, stroke_color));
            painter.line_segment([points[0], points[1]], Stroke::new(1.0, stroke_color));
            painter.line_segment([points[2], points[3]], Stroke::new(1.0, stroke_color));
        });

        ui.separator();
        ui.label("Objects:");

        // Object Node - rectangle
        Self::icon_tool_button(ui, app, EditMode::AddObjectNode, "Add object node", |painter, rect, stroke_color| {
            let inner = rect.shrink(3.0);
            painter.rect(inner, Rounding::ZERO, Color32::from_rgb(255, 255, 240), Stroke::new(1.0, stroke_color));
        });

        // Data Store - rectangle with double lines
        Self::icon_tool_button(ui, app, EditMode::AddDataStore, "Add data store", |painter, rect, stroke_color| {
            let inner = rect.shrink(3.0);
            painter.rect(inner, Rounding::ZERO, Color32::from_rgb(255, 255, 240), Stroke::new(1.0, stroke_color));
            // Double lines on sides
            painter.line_segment(
                [Pos2::new(inner.left() + 2.0, inner.top()), Pos2::new(inner.left() + 2.0, inner.bottom())],
                Stroke::new(1.0, stroke_color),
            );
            painter.line_segment(
                [Pos2::new(inner.right() - 2.0, inner.top()), Pos2::new(inner.right() - 2.0, inner.bottom())],
                Stroke::new(1.0, stroke_color),
            );
        });

        // Swimlane - vertical partition
        Self::icon_tool_button(ui, app, EditMode::AddSwimlane, "Add swimlane", |painter, rect, stroke_color| {
            let inner = rect.shrink(2.0);
            painter.rect(inner, Rounding::ZERO, Color32::from_rgba_unmultiplied(230, 230, 255, 200), Stroke::new(1.0, stroke_color));
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
        Self::icon_tool_button(ui, app, EditMode::Connect, "Create control flow", |painter, rect, stroke_color| {
            let left = Pos2::new(rect.left() + 4.0, rect.center().y);
            let right = Pos2::new(rect.right() - 4.0, rect.center().y);
            painter.line_segment([left, right], Stroke::new(1.5, stroke_color));
            // Arrowhead
            painter.line_segment([right, Pos2::new(right.x - 5.0, right.y - 4.0)], Stroke::new(1.5, stroke_color));
            painter.line_segment([right, Pos2::new(right.x - 5.0, right.y + 4.0)], Stroke::new(1.5, stroke_color));
        });
    }

    fn align_nodes(app: &mut JmtApp, mode: AlignMode) {
        const MIN_SEPARATION: f32 = 20.0; // Minimum gap between nodes

        if let Some(state) = app.current_diagram_mut() {
            let selected_ids = state.diagram.selected_nodes_in_order();
            if selected_ids.len() < 2 {
                return;
            }

            // Check if user explicitly ordered via Ctrl+Click
            let use_selection_order = state.diagram.has_explicit_selection_order();

            state.diagram.push_undo();

            // Determine which nodes to align and which are parents whose children move with them
            // Rule 1: If selection is exactly one parent + all its children, align only children
            // Rule 2: Otherwise, exclude descendants of selected parents from alignment
            //         (they move with their parent via translate_node_with_children)

            let selected_set: std::collections::HashSet<_> = selected_ids.iter().copied().collect();

            // Find which selected nodes are parents (have selected descendants)
            let mut parent_ids: Vec<NodeId> = Vec::new();
            let mut child_ids_of_parents: std::collections::HashSet<NodeId> = std::collections::HashSet::new();

            for &id in &selected_ids {
                let descendants = state.diagram.get_all_descendants(id);
                let selected_descendants: Vec<_> = descendants.iter()
                    .filter(|d| selected_set.contains(d))
                    .copied()
                    .collect();

                if !selected_descendants.is_empty() {
                    parent_ids.push(id);
                    child_ids_of_parents.extend(selected_descendants);
                }
            }

            // Check if this is exactly a parent + all its children (Rule 1)
            let align_only_children = if parent_ids.len() == 1 {
                let parent = parent_ids[0];
                let all_children = state.diagram.get_children_of_node(parent);
                let all_children_set: std::collections::HashSet<_> = all_children.iter().copied().collect();

                // Check if all selected non-parent nodes are exactly the children of this parent
                let non_parent_selected: std::collections::HashSet<_> = selected_ids.iter()
                    .filter(|id| **id != parent)
                    .copied()
                    .collect();

                non_parent_selected == all_children_set && !all_children.is_empty()
            } else {
                false
            };

            // Determine which nodes actually participate in alignment
            let nodes_to_align: Vec<NodeId> = if align_only_children {
                // Only align the children, not the parent
                selected_ids.iter()
                    .filter(|id| **id != parent_ids[0])
                    .copied()
                    .collect()
            } else {
                // Exclude descendants of selected parents (they move with parent)
                selected_ids.iter()
                    .filter(|id| !child_ids_of_parents.contains(id))
                    .copied()
                    .collect()
            };

            if nodes_to_align.len() < 2 {
                return;
            }

            // Collect node IDs with bounds for alignment calculation
            let mut nodes_with_bounds: Vec<_> = nodes_to_align.iter()
                .filter_map(|id| {
                    state.diagram.find_node(*id).map(|n| (*id, n.bounds().clone()))
                })
                .collect();

            if nodes_with_bounds.len() < 2 {
                return;
            }

            let is_horizontal_align = matches!(mode, AlignMode::Left | AlignMode::Right | AlignMode::CenterH);

            // Sort by position if NOT using explicit selection order (marquee/lasso)
            if !use_selection_order {
                if is_horizontal_align {
                    // Sort by Y for horizontal alignment (vertical spreading)
                    nodes_with_bounds.sort_by(|a, b| a.1.y1.partial_cmp(&b.1.y1).unwrap());
                } else {
                    // Sort by X for vertical alignment (horizontal spreading)
                    nodes_with_bounds.sort_by(|a, b| a.1.x1.partial_cmp(&b.1.x1).unwrap());
                }
            }

            // Collect just bounds for alignment target calculation
            let bounds: Vec<_> = nodes_with_bounds.iter().map(|(_, b)| b.clone()).collect();

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

            // Apply alignment - use translate_node_with_children for parents (unless align_only_children)
            for (id, _) in nodes_with_bounds.iter() {
                if let Some(node) = state.diagram.find_node(*id) {
                    let bounds = node.bounds().clone();
                    let offset = match mode {
                        AlignMode::Left => target - bounds.x1,
                        AlignMode::Right => target - bounds.x2,
                        AlignMode::CenterH => target - bounds.center().x,
                        AlignMode::Top => target - bounds.y1,
                        AlignMode::Bottom => target - bounds.y2,
                        AlignMode::CenterV => target - bounds.center().y,
                    };

                    let (dx, dy) = match mode {
                        AlignMode::Left | AlignMode::Right | AlignMode::CenterH => (offset, 0.0),
                        AlignMode::Top | AlignMode::Bottom | AlignMode::CenterV => (0.0, offset),
                    };

                    // If this is a parent and we're NOT in align_only_children mode,
                    // move with children
                    if !align_only_children && parent_ids.contains(id) {
                        state.diagram.translate_node_with_children(*id, dx, dy);
                    } else {
                        if let Some(node) = state.diagram.find_node_mut(*id) {
                            node.translate(dx, dy);
                        }
                    }
                }
            }

            // Re-collect bounds after alignment
            let mut nodes_with_bounds: Vec<_> = nodes_with_bounds.iter()
                .filter_map(|(id, _)| {
                    state.diagram.find_node(*id).map(|n| (*id, n.bounds().clone()))
                })
                .collect();

            // Check for overlaps and spread if needed
            for i in 1..nodes_with_bounds.len() {
                let (_prev_id, prev_bounds) = nodes_with_bounds[i - 1].clone();
                let (curr_id, curr_bounds) = nodes_with_bounds[i].clone();

                if is_horizontal_align {
                    // Check vertical overlap
                    let min_y = prev_bounds.y2 + MIN_SEPARATION;
                    if curr_bounds.y1 < min_y {
                        let offset = min_y - curr_bounds.y1;
                        // Move with children if this is a parent
                        if !align_only_children && parent_ids.contains(&curr_id) {
                            state.diagram.translate_node_with_children(curr_id, 0.0, offset);
                        } else if let Some(node) = state.diagram.find_node_mut(curr_id) {
                            node.translate(0.0, offset);
                        }
                        nodes_with_bounds[i].1.y1 += offset;
                        nodes_with_bounds[i].1.y2 += offset;
                    }
                } else {
                    // Check horizontal overlap
                    let min_x = prev_bounds.x2 + MIN_SEPARATION;
                    if curr_bounds.x1 < min_x {
                        let offset = min_x - curr_bounds.x1;
                        // Move with children if this is a parent
                        if !align_only_children && parent_ids.contains(&curr_id) {
                            state.diagram.translate_node_with_children(curr_id, offset, 0.0);
                        } else if let Some(node) = state.diagram.find_node_mut(curr_id) {
                            node.translate(offset, 0.0);
                        }
                        nodes_with_bounds[i].1.x1 += offset;
                        nodes_with_bounds[i].1.x2 += offset;
                    }
                }
            }

            // Expand parent states to contain children (preserve parentage)
            state.diagram.expand_parents_to_contain_children();
            state.diagram.recalculate_connections();
            state.modified = true;
        }
    }

    fn distribute_nodes(app: &mut JmtApp, mode: DistributeMode) {
        const MIN_SEPARATION: f32 = 20.0; // Minimum gap between nodes

        if let Some(state) = app.current_diagram_mut() {
            let selected_ids = state.diagram.selected_nodes_in_order();
            if selected_ids.len() < 3 {
                return; // Need at least 3 nodes to distribute
            }

            // Check if user explicitly ordered via Ctrl+Click
            let use_selection_order = state.diagram.has_explicit_selection_order();

            state.diagram.push_undo();

            // Determine which nodes to distribute and which are parents whose children move with them
            // Rule 1: If selection is exactly one parent + all its children, distribute only children
            // Rule 2: Otherwise, exclude descendants of selected parents from distribution
            //         (they move with their parent via translate_node_with_children)

            let selected_set: std::collections::HashSet<_> = selected_ids.iter().copied().collect();

            // Find which selected nodes are parents (have selected descendants)
            let mut parent_ids: Vec<NodeId> = Vec::new();
            let mut child_ids_of_parents: std::collections::HashSet<NodeId> = std::collections::HashSet::new();

            for &id in &selected_ids {
                let descendants = state.diagram.get_all_descendants(id);
                let selected_descendants: Vec<_> = descendants.iter()
                    .filter(|d| selected_set.contains(d))
                    .copied()
                    .collect();

                if !selected_descendants.is_empty() {
                    parent_ids.push(id);
                    child_ids_of_parents.extend(selected_descendants);
                }
            }

            // Check if this is exactly a parent + all its children (Rule 1)
            let distribute_only_children = if parent_ids.len() == 1 {
                let parent = parent_ids[0];
                let all_children = state.diagram.get_children_of_node(parent);
                let all_children_set: std::collections::HashSet<_> = all_children.iter().copied().collect();

                // Check if all selected non-parent nodes are exactly the children of this parent
                let non_parent_selected: std::collections::HashSet<_> = selected_ids.iter()
                    .filter(|id| **id != parent)
                    .copied()
                    .collect();

                non_parent_selected == all_children_set && all_children.len() >= 2
            } else {
                false
            };

            // Determine which nodes actually participate in distribution
            let nodes_to_distribute: Vec<NodeId> = if distribute_only_children {
                // Only distribute the children, not the parent
                selected_ids.iter()
                    .filter(|id| **id != parent_ids[0])
                    .copied()
                    .collect()
            } else {
                // Exclude descendants of selected parents (they move with parent)
                selected_ids.iter()
                    .filter(|id| !child_ids_of_parents.contains(id))
                    .copied()
                    .collect()
            };

            if nodes_to_distribute.len() < 3 {
                return;
            }

            // Collect node IDs with their bounds and center positions
            let mut nodes_with_info: Vec<_> = nodes_to_distribute.iter()
                .filter_map(|id| {
                    state.diagram.find_node(*id).map(|n| {
                        let bounds = n.bounds().clone();
                        let center = bounds.center();
                        (*id, bounds, center.x, center.y)
                    })
                })
                .collect();

            if nodes_with_info.len() < 3 {
                return;
            }

            // Sort by position if NOT using explicit selection order (marquee/lasso)
            if !use_selection_order {
                match mode {
                    DistributeMode::Horizontal => {
                        nodes_with_info.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
                    }
                    DistributeMode::Vertical => {
                        nodes_with_info.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap());
                    }
                }
            }

            match mode {
                DistributeMode::Horizontal => {
                    // First node stays in place, each subsequent node is placed
                    // with MIN_SEPARATION gap from the previous node's right edge
                    let mut prev_right = nodes_with_info.first().unwrap().1.x2;

                    for (i, (id, bounds, _, _)) in nodes_with_info.iter().enumerate() {
                        if i == 0 {
                            // First node stays in place
                            continue;
                        }
                        // Place this node's left edge at prev_right + MIN_SEPARATION
                        let target_left = prev_right + MIN_SEPARATION;
                        let current_left = bounds.x1;
                        let offset = target_left - current_left;

                        // Move with children if this is a parent
                        if !distribute_only_children && parent_ids.contains(id) {
                            state.diagram.translate_node_with_children(*id, offset, 0.0);
                            // Update prev_right from the translated node
                            if let Some(node) = state.diagram.find_node(*id) {
                                prev_right = node.bounds().x2;
                            }
                        } else if let Some(node) = state.diagram.find_node_mut(*id) {
                            node.translate(offset, 0.0);
                            prev_right = node.bounds().x2;
                        }
                    }
                }
                DistributeMode::Vertical => {
                    // First node stays in place, each subsequent node is placed
                    // with MIN_SEPARATION gap from the previous node's bottom edge
                    let mut prev_bottom = nodes_with_info.first().unwrap().1.y2;

                    for (i, (id, bounds, _, _)) in nodes_with_info.iter().enumerate() {
                        if i == 0 {
                            // First node stays in place
                            continue;
                        }
                        // Place this node's top edge at prev_bottom + MIN_SEPARATION
                        let target_top = prev_bottom + MIN_SEPARATION;
                        let current_top = bounds.y1;
                        let offset = target_top - current_top;

                        // Move with children if this is a parent
                        if !distribute_only_children && parent_ids.contains(id) {
                            state.diagram.translate_node_with_children(*id, 0.0, offset);
                            // Update prev_bottom from the translated node
                            if let Some(node) = state.diagram.find_node(*id) {
                                prev_bottom = node.bounds().y2;
                            }
                        } else if let Some(node) = state.diagram.find_node_mut(*id) {
                            node.translate(0.0, offset);
                            prev_bottom = node.bounds().y2;
                        }
                    }
                }
            }

            // Expand parent states to contain children (preserve parentage)
            state.diagram.expand_parents_to_contain_children();
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
