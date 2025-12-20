//! Menu bar panel

use eframe::egui;
use jmt_core::DiagramType;
use crate::app::JmtApp;

pub struct MenuBar;

impl MenuBar {
    pub fn show(ui: &mut egui::Ui, app: &mut JmtApp) {
        egui::menu::bar(ui, |ui| {
            // File menu
            ui.menu_button("File", |ui| {
                ui.menu_button("New", |ui| {
                    if ui.button("State Machine Diagram").clicked() {
                        app.new_diagram_of_type(DiagramType::StateMachine);
                        ui.close_menu();
                    }
                    if ui.button("Sequence Diagram").clicked() {
                        app.new_diagram_of_type(DiagramType::Sequence);
                        ui.close_menu();
                    }
                    if ui.button("Use Case Diagram").clicked() {
                        app.new_diagram_of_type(DiagramType::UseCase);
                        ui.close_menu();
                    }
                    if ui.button("Activity Diagram").clicked() {
                        app.new_diagram_of_type(DiagramType::Activity);
                        ui.close_menu();
                    }
                });

                #[cfg(not(target_arch = "wasm32"))]
                if ui.button("Open...").clicked() {
                    app.open();
                    ui.close_menu();
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                    let has_diagram = app.current_diagram().is_some();
                    if ui.add_enabled(has_diagram, egui::Button::new("Save")).clicked() {
                        app.save();
                        ui.close_menu();
                    }

                    if ui.add_enabled(has_diagram, egui::Button::new("Save As...")).clicked() {
                        app.save_as();
                        ui.close_menu();
                    }
                }

                ui.separator();

                if ui.button("Close").clicked() {
                    app.close_diagram();
                    ui.close_menu();
                }
            });

            // Edit menu
            ui.menu_button("Edit", |ui| {
                let can_undo = app.current_diagram()
                    .map(|d| d.diagram.can_undo())
                    .unwrap_or(false);
                let can_redo = app.current_diagram()
                    .map(|d| d.diagram.can_redo())
                    .unwrap_or(false);

                if ui.add_enabled(can_undo, egui::Button::new("Undo")).clicked() {
                    if let Some(state) = app.current_diagram_mut() {
                        state.diagram.undo();
                    }
                    ui.close_menu();
                }

                if ui.add_enabled(can_redo, egui::Button::new("Redo")).clicked() {
                    if let Some(state) = app.current_diagram_mut() {
                        state.diagram.redo();
                    }
                    ui.close_menu();
                }

                ui.separator();

                if ui.button("Delete").clicked() {
                    if let Some(state) = app.current_diagram_mut() {
                        state.diagram.push_undo();
                        state.diagram.delete_selected();
                        state.modified = true;
                    }
                    ui.close_menu();
                }
            });

            // Convert menu
            #[cfg(not(target_arch = "wasm32"))]
            ui.menu_button("Convert", |ui| {
                let has_diagram = app.current_diagram().is_some();

                if ui.add_enabled(has_diagram, egui::Button::new("Export as PNG...")).clicked() {
                    app.export_png(false);
                    ui.close_menu();
                }

                if ui.add_enabled(has_diagram, egui::Button::new("Export as PNG (Autocrop)...")).clicked() {
                    app.export_png(true);
                    ui.close_menu();
                }
            });

            // View menu
            ui.menu_button("View", |ui| {
                // Show activities toggle
                if let Some(state) = app.current_diagram_mut() {
                    let mut show = state.diagram.settings.show_activities;
                    if ui.checkbox(&mut show, "Show Activities").on_hover_text("Show/hide activities in states by default").changed() {
                        state.diagram.settings.show_activities = show;
                        state.modified = true;
                    }
                }

                ui.separator();

                if ui.button("Zoom In").clicked() {
                    // TODO: Implement zoom
                    ui.close_menu();
                }

                if ui.button("Zoom Out").clicked() {
                    // TODO: Implement zoom
                    ui.close_menu();
                }

                if ui.button("Fit to Window").clicked() {
                    // TODO: Implement fit to window
                    ui.close_menu();
                }
            });

            // Help menu
            ui.menu_button("Help", |ui| {
                if ui.button("About").clicked() {
                    // TODO: Show about dialog
                    ui.close_menu();
                }
            });
        });
    }
}
