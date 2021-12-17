#[macro_use]
mod macros;
pub mod image_texture_conversion;
pub mod ui;

use bevy_egui::egui::{self, RichText};
use egui::Color32;

const ERROR_COLOR: Color32 = Color32::from_rgb(255, 0, 0);

pub(crate) fn error_label(ui: &mut egui::Ui, msg: impl Into<RichText>) {
    ui.colored_label(ERROR_COLOR, msg);
}
pub(crate) fn error_label_needs_world(ui: &mut egui::Ui, ty: &str) -> bool {
    error_label(ui, format!("'{}' needs exclusive access to the world", ty));
    false
}
