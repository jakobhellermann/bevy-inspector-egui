use crate::{options::NumberAttributes, Inspectable};
use bevy::prelude::*;
use bevy_egui::egui;
use egui::Grid;

impl Inspectable for Quat {
    type Attributes = NumberAttributes<[f32; 4]>;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes) {
        let options = options.map(|arr| Vec4::from(*arr));
        let mut vec4 = Vec4::from(*self);
        vec4.ui(ui, options);
        *self = vec4.into();
    }
}

impl Inspectable for Transform {
    type Attributes = ();

    fn ui(&mut self, ui: &mut bevy_egui::egui::Ui, _options: Self::Attributes) {
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
