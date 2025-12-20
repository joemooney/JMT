//! Status bar panel

use eframe::egui;

pub struct StatusBar;

impl StatusBar {
    pub fn show(ui: &mut egui::Ui, message: &str) {
        ui.horizontal(|ui| {
            ui.label(message);
        });
    }
}
