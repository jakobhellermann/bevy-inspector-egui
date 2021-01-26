use bevy_egui::egui;
use egui::{Color32, Label};

const ERROR_COLOR: Color32 = Color32::from_rgb(255, 0, 0);

pub(crate) fn error_label(ui: &mut egui::Ui, msg: impl Into<Label>) {
    ui.colored_label(ERROR_COLOR, msg);
}

#[allow(unused)]
macro_rules! impl_for_simple_enum {
    ($name:ident with $($variant:ident),* ) => {
        impl $crate::Inspectable for $name {
            type Attributes = ();


            fn ui(&mut self, ui: &mut $crate::egui::Ui, _: Self::Attributes, _: &$crate::Context) {
                use $crate::egui;

                let id = ui.make_persistent_id(stringify!(#id));
                egui::combo_box(ui, id, format!("{:?}", self), |ui| {
                    $(
                        ui.selectable_value(self, $name::$variant, format!("{:?}", $name::$variant));
                    )*
                });
            }
        }
    }
}

macro_rules! expect_context {
    ($ui:ident, $context:ident.$field:ident, $ty:literal) => {
        match $context.$field {
            Some(val) => val,
            None => {
                let msg = format!(
                    "'{}' needs unique access via InspectorPlugin::thread_local",
                    $ty
                );
                return utils::error_label($ui, msg);
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
                return utils::error_label($ui, msg);
            }
        }
    };
}
