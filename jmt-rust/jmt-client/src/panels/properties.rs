//! Properties panel for editing selected elements

use eframe::egui;
use jmt_core::{Node, NodeId, TitleStyle};
use crate::app::JmtApp;

/// Actions that the properties panel can request
#[derive(Debug, Clone)]
pub enum PropertiesAction {
    /// Open a sub-statemachine for the given state
    OpenSubStateMachine(NodeId),
    /// Create a new embedded sub-statemachine for the given state
    CreateSubStateMachine(NodeId),
    /// Expand a state to fit its children (when show_expanded is enabled)
    ExpandStateToFitChildren(NodeId),
}

pub struct PropertiesPanel;

impl PropertiesPanel {
    pub fn show(ui: &mut egui::Ui, app: &mut JmtApp) {
        ui.heading("Properties");
        ui.separator();

        let mut action: Option<PropertiesAction> = None;

        if let Some(state) = app.current_diagram_mut() {
            let selected_nodes = state.diagram.selected_nodes();
            let selected_conn = state.diagram.selected_connection();

            if selected_nodes.len() == 1 {
                // Show node properties
                let node_id = selected_nodes[0];

                // Get region info before getting mutable reference
                let region_info = state.diagram.find_node(node_id)
                    .and_then(|n| n.parent_region_id())
                    .map(|rid| {
                        let name = state.diagram.find_region_name(rid)
                            .unwrap_or_else(|| "Unknown".to_string());
                        let parent_state = state.diagram.find_region_parent_state(rid)
                            .map(|s| s.name.clone());
                        (name, parent_state)
                    });

                if let Some(node) = state.diagram.find_node_mut(node_id) {
                    action = Self::show_node_properties(ui, node, &mut state.modified, region_info);
                }
            } else if let Some(conn_id) = selected_conn {
                // Show connection properties
                let mut needs_recalculate = false;
                if let Some(conn) = state.diagram.find_connection_mut(conn_id) {
                    needs_recalculate = Self::show_connection_properties(ui, conn, &mut state.modified);
                }

                if needs_recalculate {
                    state.diagram.recalculate_connections();
                }
            } else if selected_nodes.len() > 1 {
                // Multiple nodes selected
                ui.label(format!("{} nodes selected", selected_nodes.len()));
                ui.label("Select a single node to edit properties");
            } else {
                // Nothing selected - show diagram properties
                Self::show_diagram_properties(ui, state);
            }
        } else {
            ui.label("No diagram open");
        }

        // Handle actions from properties panel
        if let Some(action) = action {
            match action {
                PropertiesAction::OpenSubStateMachine(node_id) => {
                    app.open_substatemachine(node_id);
                }
                PropertiesAction::CreateSubStateMachine(node_id) => {
                    app.create_substatemachine(node_id);
                }
                PropertiesAction::ExpandStateToFitChildren(node_id) => {
                    if let Some(state) = app.current_diagram_mut() {
                        state.diagram.push_undo();
                        state.diagram.expand_state_to_fit_children(node_id);
                        state.diagram.recalculate_connections();
                        state.modified = true;
                    }
                }
            }
        }
    }

    fn show_node_properties(
        ui: &mut egui::Ui,
        node: &mut Node,
        modified: &mut bool,
        region_info: Option<(String, Option<String>)>,  // (region_name, parent_state_name)
    ) -> Option<PropertiesAction> {
        let mut action: Option<PropertiesAction> = None;
        let node_id = node.id();
        ui.label(format!("Type: {}", node.node_type().display_name()));

        ui.horizontal(|ui| {
            ui.label("Name:");
            let mut name = node.name().to_string();
            if ui.text_edit_singleline(&mut name).changed() {
                node.set_name(name);
                *modified = true;
            }
        });

        // Show containing region
        if let Some((region_name, parent_state)) = region_info {
            ui.horizontal(|ui| {
                ui.label("In Region:");
                if let Some(state_name) = parent_state {
                    if state_name == "Root" {
                        ui.label(format!("{} (diagram)", region_name));
                    } else {
                        ui.label(format!("{} (in {})", region_name, state_name));
                    }
                } else {
                    ui.label(&region_name);
                }
            });
        } else {
            ui.horizontal(|ui| {
                ui.label("In Region:");
                ui.label("(unassigned)");
            });
        }

        // Show state-specific properties
        if let Some(state) = node.as_state_mut() {
            ui.separator();

            // Show activities checkbox
            ui.horizontal(|ui| {
                let mut show = state.show_activities.unwrap_or(true);
                let label = if state.show_activities.is_some() {
                    "Show Activities"
                } else {
                    "Show Activities (using diagram default)"
                };
                if ui.checkbox(&mut show, label).changed() {
                    state.show_activities = Some(show);
                    *modified = true;
                }
            });

            // Reset to diagram default button
            if state.show_activities.is_some() {
                if ui.small_button("Use diagram default").clicked() {
                    state.show_activities = None;
                    *modified = true;
                }
            }

            ui.separator();
            ui.label("Activities:");

            ui.horizontal(|ui| {
                ui.label("Entry:");
            });
            if ui.text_edit_multiline(&mut state.entry_activity).changed() {
                *modified = true;
            }

            ui.horizontal(|ui| {
                ui.label("Exit:");
            });
            if ui.text_edit_multiline(&mut state.exit_activity).changed() {
                *modified = true;
            }

            ui.horizontal(|ui| {
                ui.label("Do:");
            });
            if ui.text_edit_multiline(&mut state.do_activity).changed() {
                *modified = true;
            }

            ui.separator();

            // Auto-fit button
            if ui.button("⬚ Fit to Content").on_hover_text("Resize state to fit its content").clicked() {
                let show_activities = state.show_activities.unwrap_or(true);
                state.resize_to_fit(show_activities);
                *modified = true;
            }

            ui.separator();

            ui.label(format!("Regions: {}", state.regions.len()));
            if ui.button("Add Region").clicked() {
                state.add_region("Region");
                *modified = true;
            }

            ui.separator();

            // Sub-Statemachine Section
            ui.label("Sub-Statemachine:");

            // Title field
            ui.horizontal(|ui| {
                ui.label("Title:");
                if ui.text_edit_singleline(&mut state.title).changed() {
                    *modified = true;
                }
            });

            // Show expanded checkbox (only if has sub-statemachine)
            if state.substatemachine_path.is_some() {
                let was_expanded = state.show_expanded;
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut state.show_expanded, "Show Expanded")
                        .on_hover_text("Show sub-statemachine contents inline instead of icon")
                        .changed()
                    {
                        *modified = true;
                    }
                });
                // If show_expanded was just enabled, expand state to fit children
                if state.show_expanded && !was_expanded {
                    action = Some(PropertiesAction::ExpandStateToFitChildren(node_id));
                }

                // Storage mode
                ui.horizontal(|ui| {
                    ui.label("Storage:");
                    let is_external = state.is_external_substatemachine();

                    if ui.selectable_label(!is_external, "Embedded")
                        .on_hover_text("Store sub-statemachine in this file")
                        .clicked()
                    {
                        state.substatemachine_path = Some(String::new());
                        *modified = true;
                    }
                    if ui.selectable_label(is_external, "External File")
                        .on_hover_text("Store sub-statemachine in separate .jmt file")
                        .clicked()
                    {
                        // Default to name-based file
                        let safe_name = state.name.replace(' ', "_");
                        state.substatemachine_path = Some(format!("{}_sub.jmt", safe_name));
                        *modified = true;
                    }
                });

                // Show file path if external
                let is_external = state.substatemachine_path.as_ref()
                    .map(|p| !p.is_empty())
                    .unwrap_or(false);
                if is_external {
                    let mut path_str = state.substatemachine_path.clone().unwrap_or_default();
                    let original_path = path_str.clone();
                    ui.horizontal(|ui| {
                        ui.label("Path:");
                        ui.text_edit_singleline(&mut path_str);
                    });
                    // Update after horizontal to avoid borrow issues
                    if path_str != original_path {
                        state.substatemachine_path = Some(path_str);
                        *modified = true;
                    }
                }

                // Open button
                if ui.button("📂 Open SubStateMachine")
                    .on_hover_text("Open sub-statemachine in a new tab")
                    .clicked()
                {
                    action = Some(PropertiesAction::OpenSubStateMachine(node_id));
                }

                // Remove sub-statemachine button
                if ui.button("🗑 Remove SubStateMachine")
                    .on_hover_text("Remove sub-statemachine reference")
                    .clicked()
                {
                    state.substatemachine_path = None;
                    state.show_expanded = false;
                    *modified = true;
                }
            } else {
                // Create button
                if ui.button("➕ Create SubStateMachine")
                    .on_hover_text("Create a sub-statemachine for this state")
                    .clicked()
                {
                    action = Some(PropertiesAction::CreateSubStateMachine(node_id));
                }
            }
        }

        // Show bounds (read-only)
        ui.separator();
        ui.label("Bounds:");
        let bounds = node.bounds();
        ui.label(format!("  X: {:.0} - {:.0}", bounds.x1, bounds.x2));
        ui.label(format!("  Y: {:.0} - {:.0}", bounds.y1, bounds.y2));
        ui.label(format!("  Size: {:.0} x {:.0}", bounds.width(), bounds.height()));

        action
    }

    /// Show connection properties. Returns true if segments need recalculation.
    fn show_connection_properties(ui: &mut egui::Ui, conn: &mut jmt_core::Connection, modified: &mut bool) -> bool {
        let mut needs_recalculate = false;

        ui.label("Transition");

        ui.horizontal(|ui| {
            ui.label("Name:");
            if ui.text_edit_singleline(&mut conn.name).changed() {
                *modified = true;
            }
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Event:");
        });
        if ui.text_edit_singleline(&mut conn.event).changed() {
            *modified = true;
        }

        ui.horizontal(|ui| {
            ui.label("Guard:");
        });
        if ui.text_edit_singleline(&mut conn.guard).changed() {
            *modified = true;
        }

        ui.horizontal(|ui| {
            ui.label("Action:");
        });
        if ui.text_edit_multiline(&mut conn.action).changed() {
            *modified = true;
        }

        ui.separator();

        // Show label preview
        let label = conn.label();
        if !label.is_empty() {
            ui.label(format!("Label: {}", label));
        }

        ui.separator();

        // Text adjoined checkbox
        let mut text_adjoined = conn.text_adjoined;
        if ui.checkbox(&mut text_adjoined, "Text Adjoined")
            .on_hover_text("Keep label attached to connection without leader line")
            .changed()
        {
            conn.set_text_adjoined(text_adjoined);
            *modified = true;
        }

        ui.separator();

        // Show pivot points info
        let num_pivots = conn.pivot_points.len();
        if num_pivots > 0 {
            ui.label(format!("Pivot Points: {}", num_pivots));

            // Segment curve toggles
            let num_segments = num_pivots + 1;
            ui.label("Segments:");

            // Ensure segment_curves has correct length
            while conn.segment_curves.len() < num_segments {
                conn.segment_curves.push(false);
            }
            conn.segment_curves.truncate(num_segments);

            for i in 0..num_segments {
                let label = if num_segments == 1 {
                    "Direct".to_string()
                } else if i == 0 {
                    "Start → P1".to_string()
                } else if i == num_segments - 1 {
                    format!("P{} → End", i)
                } else {
                    format!("P{} → P{}", i, i + 1)
                };

                ui.horizontal(|ui| {
                    let was_curved = conn.segment_curves[i];
                    if ui.checkbox(&mut conn.segment_curves[i], "").changed() {
                        *modified = true;
                        if was_curved != conn.segment_curves[i] {
                            needs_recalculate = true;
                        }
                    }
                    let style = if conn.segment_curves[i] { "Curved" } else { "Straight" };
                    ui.label(format!("{}: {}", label, style));
                });
            }

            ui.separator();

            // Clear pivot points button
            if ui.button("Clear Pivot Points").clicked() {
                conn.pivot_points.clear();
                conn.segment_curves.clear();
                *modified = true;
                needs_recalculate = true;
            }
        } else {
            ui.label("Pivot Points: 0");
            ui.label("Double-click on connection to add pivot points");
        }

        needs_recalculate
    }

    fn show_diagram_properties(ui: &mut egui::Ui, state: &mut crate::app::DiagramState) {
        ui.label("Diagram");

        ui.horizontal(|ui| {
            ui.label("Name:");
            if ui.text_edit_singleline(&mut state.diagram.settings.name).changed() {
                state.modified = true;
            }
        });

        ui.separator();

        // Title settings
        ui.label("Title:");
        ui.horizontal(|ui| {
            ui.label("Text:");
            if ui.text_edit_singleline(&mut state.diagram.title).changed() {
                state.modified = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Style:");
            egui::ComboBox::from_id_salt("title_style")
                .selected_text(state.diagram.title_style.display_name())
                .show_ui(ui, |ui| {
                    for style in TitleStyle::all() {
                        if ui.selectable_value(
                            &mut state.diagram.title_style,
                            *style,
                            style.display_name(),
                        ).clicked() {
                            state.modified = true;
                        }
                    }
                });
        });

        ui.separator();

        if let Some(path) = &state.diagram.settings.file_path {
            ui.label(format!("Path: {}", path));
        } else {
            ui.label("Path: (not saved)");
        }

        ui.separator();

        ui.label("Statistics:");
        ui.label(format!("  Nodes: {}", state.diagram.nodes().len()));
        ui.label(format!("  Connections: {}", state.diagram.connections().len()));
    }
}
