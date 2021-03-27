#[macro_use]
mod macros;
pub mod image_texture_conversion;
mod sort_if;
pub mod ui;

pub use sort_if::sort_iter_if;

use bevy_egui::egui;
use egui::{Color32, Label};

const ERROR_COLOR: Color32 = Color32::from_rgb(255, 0, 0);

pub(crate) fn error_label(ui: &mut egui::Ui, msg: impl Into<Label>) {
    ui.colored_label(ERROR_COLOR, msg);
}

macro_rules! expect_world {
    ($ui:ident, $context:ident, $ty:literal) => {
        match unsafe { $context.world() } {
            Some(val) => val,
            None => {
                return $crate::utils::error_label(
                    $ui,
                    format!("'{}' needs exclusive access to the world", $ty),
                );
            }
        }
    };
}
