use crate::{options::NumberAttributes, Inspectable, Options};
use bevy::prelude::*;
use bevy_egui::egui;

impl Inspectable for Quat {
    type FieldOptions = NumberAttributes<[f32; 4]>;

    fn ui(&mut self, ui: &mut egui::Ui, options: Options<Self::FieldOptions>) {
        let options = options.map(|custom| custom.map(|arr| Vec4::from(*arr)));
        let mut vec4 = Vec4::from(*self);
        vec4.ui(ui, options);
        *self = vec4.into();
    }
}
