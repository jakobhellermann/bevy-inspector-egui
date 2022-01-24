use bevy::prelude::*;
use bevy_inspector_egui::plugin::InspectorWindows;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};

#[derive(Default, Inspectable)]
struct Data {
    field: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<Data>::new())
        .add_system(toggle_inspector)
        .run();
}

fn toggle_inspector(
    input: ResMut<Input<KeyCode>>,
    mut inspector_windows: ResMut<InspectorWindows>,
) {
    if input.just_pressed(KeyCode::Space) {
        let mut inspector_window_data = inspector_windows.window_data_mut::<Data>();
        inspector_window_data.visible = !inspector_window_data.visible;
    }
}
