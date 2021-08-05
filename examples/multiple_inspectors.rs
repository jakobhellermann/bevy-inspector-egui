use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};

#[derive(Inspectable, Default)]
struct UiData {
    font_size: f32,
    color: Color,
}
#[derive(Inspectable, Default)]
struct TransformData {
    transform: Transform,
    font_size: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<UiData>::new())
        .add_plugin(InspectorPlugin::<TransformData>::new())
        .run();
}
