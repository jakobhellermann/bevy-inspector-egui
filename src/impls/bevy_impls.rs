use crate::{options::NumberAttributes, Inspectable, Options};
use bevy::prelude::*;
use bevy_egui::egui;
use egui::Grid;

impl Inspectable for Quat {
    type FieldOptions = NumberAttributes<[f32; 4]>;

    fn ui(&mut self, ui: &mut egui::Ui, options: Options<Self::FieldOptions>) {
        let options = options.map(|custom| custom.map(|arr| Vec4::from(*arr)));
        let mut vec4 = Vec4::from(*self);
        vec4.ui(ui, options);
        *self = vec4.into();
    }
}

impl Inspectable for Transform {
    type FieldOptions = ();

    fn ui(&mut self, ui: &mut bevy_egui::egui::Ui, _options: crate::Options<Self::FieldOptions>) {
        let id = "asdf";
        Grid::new(id).show(ui, |ui| {
            ui.label("Translation");
            self.translation.ui(ui, Default::default());
            ui.end_row();

            ui.label("Rotation");
            self.rotation.ui(ui, Default::default());
            ui.end_row();

            ui.label("Scale");
            self.scale.ui(ui, Default::default());
            ui.end_row();
        });
    }
}
