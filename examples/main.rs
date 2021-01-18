use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};

#[derive(Inspectable, Debug, Default)]
struct Data {
    #[inspectable(min = 10.0, max = 70.0)]
    font_size: f32,
    text: String,
    show_square: bool,
    color: Color,
    // #[inspectable(min = Vec2::new(-200., -200.), max = Vec2::new(200., 200.))]
    // position: Vec2,
    // list: [Vec2; 3],
    custom_enum: CustomEnum,
}

#[derive(Inspectable, Debug, PartialEq)]
enum CustomEnum {
    A,
    B,
    C,
}
impl Default for CustomEnum {
    fn default() -> Self {
        CustomEnum::A
    }
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
