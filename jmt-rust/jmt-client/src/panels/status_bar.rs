//! Status bar panel

use eframe::egui;

pub struct StatusBar;

impl StatusBar {
    pub fn show(ui: &mut egui::Ui, message: &str, hover_info: Option<&str>, cursor_pos: Option<(f32, f32)>) {
        ui.horizontal(|ui| {
            // Main status message
            ui.label(message);

            // Spacer to push hover info to the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Cursor position (rightmost)
                if let Some((x, y)) = cursor_pos {
                    ui.label(format!("({:.0}, {:.0})", x, y));
                    ui.separator();
                }

                // Hover info
                if let Some(info) = hover_info {
                    ui.label(info);
                }
            });
        });
    }
}
