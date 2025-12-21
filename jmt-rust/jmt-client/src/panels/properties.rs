//! Properties panel for editing selected elements

use eframe::egui;
use jmt_core::Node;
use crate::app::JmtApp;

pub struct PropertiesPanel;

impl PropertiesPanel {
    pub fn show(ui: &mut egui::Ui, app: &mut JmtApp) {
        ui.heading("Properties");
        ui.separator();

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
                    Self::show_node_properties(ui, node, &mut state.modified, region_info);
                }
            } else if let Some(conn_id) = selected_conn {
                // Show connection properties
                if let Some(conn) = state.diagram.find_connection_mut(conn_id) {
                    Self::show_connection_properties(ui, conn, &mut state.modified);
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
    }

    fn show_node_properties(
        ui: &mut egui::Ui,
        node: &mut Node,
        modified: &mut bool,
        region_info: Option<(String, Option<String>)>,  // (region_name, parent_state_name)
    ) {
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
        }

        // Show bounds (read-only)
        ui.separator();
        ui.label("Bounds:");
        let bounds = node.bounds();
        ui.label(format!("  X: {:.0} - {:.0}", bounds.x1, bounds.x2));
        ui.label(format!("  Y: {:.0} - {:.0}", bounds.y1, bounds.y2));
        ui.label(format!("  Size: {:.0} x {:.0}", bounds.width(), bounds.height()));
    }

    fn show_connection_properties(ui: &mut egui::Ui, conn: &mut jmt_core::Connection, modified: &mut bool) {
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
    }

    fn show_diagram_properties(ui: &mut egui::Ui, state: &mut crate::app::DiagramState) {
        ui.label("Diagram");

        ui.horizontal(|ui| {
            ui.label("Name:");
            if ui.text_edit_singleline(&mut state.diagram.settings.name).changed() {
                state.modified = true;
            }
        });

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
