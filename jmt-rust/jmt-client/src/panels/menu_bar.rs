//! Menu bar panel

use eframe::egui;
use crate::app::JmtApp;

pub struct MenuBar;

impl MenuBar {
    pub fn show(ui: &mut egui::Ui, app: &mut JmtApp) {
        egui::menu::bar(ui, |ui| {
            // File menu
            ui.menu_button("File", |ui| {
                if ui.button("New").clicked() {
                    app.new_diagram();
                    ui.close_menu();
                }

                if ui.button("Open...").clicked() {
                    // TODO: Implement file open via server
                    ui.close_menu();
                }

                if ui.button("Save").clicked() {
                    // TODO: Implement file save via server
                    ui.close_menu();
                }

                if ui.button("Save As...").clicked() {
                    // TODO: Implement save as via server
                    ui.close_menu();
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

            // View menu
            ui.menu_button("View", |ui| {
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
