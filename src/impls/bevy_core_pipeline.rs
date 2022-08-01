use bevy::prelude::ClearColor;
use bevy::prelude::Color;
use bevy_egui::egui;

use crate::Context;
use crate::Inspectable;

impl Inspectable for ClearColor {
    type Attributes = <Color as Inspectable>::Attributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &mut Context) -> bool {
        self.0.ui(ui, options, context)
    }
}
