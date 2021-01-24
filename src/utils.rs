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
