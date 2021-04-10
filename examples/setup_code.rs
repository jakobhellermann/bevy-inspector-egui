use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};

#[derive(Default)]
struct ExampleWidget;
impl Inspectable for ExampleWidget {
    type Attributes = ();

    fn ui(
        &mut self,
        _: &mut bevy_inspector_egui::egui::Ui,
        _: Self::Attributes,
        _: &bevy_inspector_egui::Context,
    ) -> bool {
        false
    }

    fn setup(_: &mut AppBuilder) {
        eprintln!("Running setup code...");

        // app.init_resource::<WhateverYouNeed>();
    }
}

#[derive(Inspectable, Default)]
struct Data {
    _widget: ExampleWidget,
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<Data>::new());
}
