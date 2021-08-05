use bevy::prelude::*;
use bevy_inspector_egui::widgets::InNewWindow;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};

#[derive(Inspectable, Default)]
struct SomeComplexType {
    very_long_field_name: Color,
}

#[derive(Inspectable, Default)]
struct Inspector {
    a: f32,
    #[inspectable(title = "Complex Type", resizable)]
    window: InNewWindow<SomeComplexType>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<Inspector>::new())
        .run();
}
