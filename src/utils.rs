use bevy_egui::egui;
use egui::{Color32, Label};

const ERROR_COLOR: Color32 = Color32::from_rgb(255, 0, 0);

pub(crate) fn error_label(ui: &mut egui::Ui, msg: impl Into<Label>) {
    ui.colored_label(ERROR_COLOR, msg);
}
