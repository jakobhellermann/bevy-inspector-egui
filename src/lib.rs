mod impls;

use bevy_egui::egui;

pub struct Options<T> {
    label: &'static str,
    custom: T,
}
impl<T: Default> Options<T> {
    pub fn default(label: &'static str) -> Self {
        Options {
            label,
            custom: T::default(),
        }
    }
    pub fn new(label: &'static str, custom: T) -> Self {
        Options { label, custom }
    }
}

pub trait InspectableWidget {
    type FieldOptions: Default;

    fn ui(&mut self, ui: &mut egui::Ui, options: Options<Self::FieldOptions>);
}
