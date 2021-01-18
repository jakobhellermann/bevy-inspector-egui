mod impls;
mod plugin;

#[doc(hidden)]
pub use bevy_egui::egui;

pub use bevy_inspector_egui_derive::Inspectable;
pub use plugin::InspectorPlugin;

pub mod options {
    pub use crate::impls::*;
}

#[non_exhaustive]
#[derive(Default)]
pub struct Options<T> {
    pub custom: T,
}
impl<T: Default> Options<T> {
    pub fn new(custom: T) -> Self {
        Options { custom }
    }
}

pub trait InspectableWidget {
    type FieldOptions: Default;

    fn ui(&mut self, ui: &mut egui::Ui, options: Options<Self::FieldOptions>);
}
