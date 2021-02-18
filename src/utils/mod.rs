#[macro_use]
mod macros;
pub mod ui;

use bevy_egui::egui;
use egui::{Color32, Label};

const ERROR_COLOR: Color32 = Color32::from_rgb(255, 0, 0);

pub(crate) fn error_label(ui: &mut egui::Ui, msg: impl Into<Label>) {
    ui.colored_label(ERROR_COLOR, msg);
}

macro_rules! expect_context {
    ($ui:ident, $context:ident.$field:ident, $ty:literal) => {
        match $context.$field {
            Some(val) => val,
            None => {
                let msg = format!(
                    "'{}' needs exclusive access to the {}",
                    $ty,
                    stringify!($field)
                );
                return $crate::utils::error_label($ui, msg);
            }
        }
    };
}
macro_rules! expect_resource {
    ($ui:ident, $resources:ident, $method:ident $ty:ty) => {
        match $resources.$method::<$ty>() {
            Some(res) => res,
            None => {
                let msg = format!("No {} resource found", std::any::type_name::<$ty>());
                return $crate::utils::error_label($ui, msg);
            }
        }
    };
}
