//! Toolbar panel

use eframe::egui;
use jmt_core::{EditMode, DiagramType};
use crate::app::JmtApp;

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

    fn show_state_machine_tools(ui: &mut egui::Ui, app: &mut JmtApp) {
        ui.label("Add:");
        Self::tool_button(ui, app, EditMode::AddState, "▢ State", "Add a state");
        Self::tool_button(ui, app, EditMode::AddInitial, "● Initial", "Add initial pseudo-state");
        Self::tool_button(ui, app, EditMode::AddFinal, "◉ Final", "Add final pseudo-state");
        Self::tool_button(ui, app, EditMode::AddChoice, "◇ Choice", "Add choice pseudo-state");
        Self::tool_button(ui, app, EditMode::AddJunction, "◆ Junction", "Add junction pseudo-state");
        Self::tool_button(ui, app, EditMode::AddFork, "┳ Fork", "Add fork pseudo-state");
        Self::tool_button(ui, app, EditMode::AddJoin, "┻ Join", "Add join pseudo-state");

        ui.separator();
        ui.label("Connect:");
        Self::tool_button(ui, app, EditMode::Connect, "→ Transition", "Create a transition between nodes");
    }

    fn show_sequence_tools(ui: &mut egui::Ui, app: &mut JmtApp) {
        ui.label("Add:");
        Self::tool_button(ui, app, EditMode::AddLifeline, "⎸ Lifeline", "Add a lifeline");
        Self::tool_button(ui, app, EditMode::AddActivation, "▮ Activation", "Add an activation box");
        Self::tool_button(ui, app, EditMode::AddFragment, "⊡ Fragment", "Add a combined fragment");

        ui.separator();
        ui.label("Messages:");
        Self::tool_button(ui, app, EditMode::AddSyncMessage, "→ Sync", "Add synchronous message");
        Self::tool_button(ui, app, EditMode::AddAsyncMessage, "⇢ Async", "Add asynchronous message");
        Self::tool_button(ui, app, EditMode::AddReturnMessage, "⇠ Return", "Add return message");
        Self::tool_button(ui, app, EditMode::AddSelfMessage, "↺ Self", "Add self message");
    }

    fn show_use_case_tools(ui: &mut egui::Ui, app: &mut JmtApp) {
        ui.label("Add:");
        Self::tool_button(ui, app, EditMode::AddActor, "☺ Actor", "Add an actor");
        Self::tool_button(ui, app, EditMode::AddUseCase, "○ Use Case", "Add a use case");
        Self::tool_button(ui, app, EditMode::AddSystemBoundary, "▭ System", "Add system boundary");

        ui.separator();
        ui.label("Connect:");
        Self::tool_button(ui, app, EditMode::AddAssociation, "─ Association", "Add association");
        Self::tool_button(ui, app, EditMode::AddInclude, "⟵ Include", "Add include relationship");
        Self::tool_button(ui, app, EditMode::AddExtend, "⟶ Extend", "Add extend relationship");
        Self::tool_button(ui, app, EditMode::AddGeneralization, "▷ Generalize", "Add generalization");
    }

    fn show_activity_tools(ui: &mut egui::Ui, app: &mut JmtApp) {
        ui.label("Add:");
        Self::tool_button(ui, app, EditMode::AddAction, "▢ Action", "Add an action");
        Self::tool_button(ui, app, EditMode::AddInitial, "● Initial", "Add initial node");
        Self::tool_button(ui, app, EditMode::AddFinal, "◉ Final", "Add final node");
        Self::tool_button(ui, app, EditMode::AddDecision, "◇ Decision", "Add decision/merge node");
        Self::tool_button(ui, app, EditMode::AddFork, "┳ Fork", "Add fork bar");
        Self::tool_button(ui, app, EditMode::AddJoin, "┻ Join", "Add join bar");

        ui.separator();
        ui.label("Signals:");
        Self::tool_button(ui, app, EditMode::AddSendSignal, "▷ Send", "Add send signal action");
        Self::tool_button(ui, app, EditMode::AddAcceptEvent, "◁ Accept", "Add accept event action");
        Self::tool_button(ui, app, EditMode::AddTimeEvent, "⏱ Time", "Add time event action");

        ui.separator();
        ui.label("Objects:");
        Self::tool_button(ui, app, EditMode::AddObjectNode, "▤ Object", "Add object node");
        Self::tool_button(ui, app, EditMode::AddDataStore, "▥ Data Store", "Add data store");
        Self::tool_button(ui, app, EditMode::AddSwimlane, "⎮ Swimlane", "Add swimlane");

        ui.separator();
        ui.label("Connect:");
        Self::tool_button(ui, app, EditMode::Connect, "→ Flow", "Create control flow");
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
