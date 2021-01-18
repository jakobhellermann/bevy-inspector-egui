use crate::{InspectableWidget, Options};
use bevy_egui::egui;
use egui::widgets;

pub struct NumberAttributes {
    pub min: f32,
    pub max: f32,
}
impl Default for NumberAttributes {
    fn default() -> Self {
        NumberAttributes { min: 0.0, max: 1.0 }
    }
}

macro_rules! impl_for_num {
    ($ty:ident) => {
        impl InspectableWidget for $ty {
            type FieldOptions = NumberAttributes;

            fn ui(&mut self, ui: &mut egui::Ui, options: Options<Self::FieldOptions>) {
                let widget = widgets::DragValue::$ty(self)
                    .range(options.custom.min..=options.custom.max);
                ui.add(widget);
            }
        }
    };

    ($($ty:ident),*) => {
        $(impl_for_num!($ty);)*
    }
}

impl_for_num!(f32, f64, u8, i32);

impl InspectableWidget for String {
    type FieldOptions = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Options<Self::FieldOptions>) {
        let widget = widgets::TextEdit::singleline(self);
        ui.add(widget);
    }
}

impl InspectableWidget for bool {
    type FieldOptions = ();
    fn ui(&mut self, ui: &mut egui::Ui, options: Options<Self::FieldOptions>) {
        ui.checkbox(self, options.label);
    }
}
