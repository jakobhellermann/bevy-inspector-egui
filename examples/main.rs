use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, InspectableWidget, InspectorPlugin, Options};

#[derive(Inspectable, Debug, Default)]
struct Data {
    font_size: f32,
    // #[inspectable(min = 10.0, max = 70.0)]
    text: String,
    show_square: bool,
    // text_color: TextColor,
    // color: Color,
    // #[inspectable(min = Vec2::new(-200., -200.), max = Vec2::new(200., 200.))]
    // position: Vec2,
    // list: [Vec2; 3],
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<Data>::new())
        .add_system(data.system())
        .run();
}

// TODO: make ChangedRes work
fn data(data: Res<Data>) {
    dbg!(data);
}
