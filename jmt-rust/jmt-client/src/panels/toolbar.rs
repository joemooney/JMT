//! Toolbar panel

use eframe::egui;
use jmt_core::EditMode;
use crate::app::JmtApp;

pub struct Toolbar;

impl Toolbar {
    pub fn show(ui: &mut egui::Ui, app: &mut JmtApp) {
        ui.horizontal(|ui| {
            // Selection tools
            ui.label("Select:");
            Self::tool_button(ui, app, EditMode::Arrow, "Arrow", "Select and move nodes");

            ui.separator();

            // Node creation tools
            ui.label("Add:");
            Self::tool_button(ui, app, EditMode::AddState, "State", "Add a state");
            Self::tool_button(ui, app, EditMode::AddInitial, "Initial", "Add initial pseudo-state");
            Self::tool_button(ui, app, EditMode::AddFinal, "Final", "Add final pseudo-state");
            Self::tool_button(ui, app, EditMode::AddChoice, "Choice", "Add choice pseudo-state");
            Self::tool_button(ui, app, EditMode::AddJunction, "Junction", "Add junction pseudo-state");
            Self::tool_button(ui, app, EditMode::AddFork, "Fork", "Add fork pseudo-state");
            Self::tool_button(ui, app, EditMode::AddJoin, "Join", "Add join pseudo-state");

            ui.separator();

            // Connection tool
            ui.label("Connect:");
            Self::tool_button(ui, app, EditMode::Connect, "Transition", "Create a transition between nodes");

            ui.separator();

            // Alignment tools
            ui.label("Align:");
            if ui.button("Left").on_hover_text("Align selected nodes to the left").clicked() {
                Self::align_nodes(app, AlignMode::Left);
            }
            if ui.button("Center").on_hover_text("Center selected nodes horizontally").clicked() {
                Self::align_nodes(app, AlignMode::CenterH);
            }
            if ui.button("Right").on_hover_text("Align selected nodes to the right").clicked() {
                Self::align_nodes(app, AlignMode::Right);
            }
            if ui.button("Top").on_hover_text("Align selected nodes to the top").clicked() {
                Self::align_nodes(app, AlignMode::Top);
            }
            if ui.button("Middle").on_hover_text("Center selected nodes vertically").clicked() {
                Self::align_nodes(app, AlignMode::CenterV);
            }
            if ui.button("Bottom").on_hover_text("Align selected nodes to the bottom").clicked() {
                Self::align_nodes(app, AlignMode::Bottom);
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
