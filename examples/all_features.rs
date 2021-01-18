use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};

#[derive(Inspectable, Debug, Default)]
struct Data {
    #[inspectable(min = 10.0, max = 70.0)]
    font_size: f32,
    text: String,
    #[inspectable(label = "Display Square")]
    show_square: bool,
    color: Color,
    #[inspectable(min = Vec2::new(-200., -200.), max = Vec2::new(200., 200.))]
    position: Vec2,
    #[inspectable(min = 42.0, max = 100.0, speed = 2.0)] // attributes get passed to each child
    list: [f32; 2],
    custom_enum: CustomEnum,
    #[inspectable(collapse)]
    noise_settings: NoiseSettings,
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

#[derive(Inspectable, Debug, Default)]
struct NoiseSettings {
    octaves: u8,
    frequency: f32,
    lacunarity: f32,
    persistence: f32,
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
