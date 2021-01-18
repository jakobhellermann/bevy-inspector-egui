use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_inspector_egui::{Inspectable, InspectableWidget, Options};

#[derive(Debug, Default, Inspectable)]
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
        .add_plugin(EguiPlugin)
        .add_system(ui_example.system())
        .init_resource::<Data>()
        .add_system(data.system())
        .run();
}

fn ui_example(mut egui_context: ResMut<EguiContext>, mut data: ResMut<Data>) {
    let ctx = &mut egui_context.ctx;

    egui::Window::new("Inspector")
        .resizable(false)
        .show(ctx, |ui| {
            data.ui(ui, Options::default());
        });
}

fn data(data: ChangedRes<Data>) {
    dbg!(data);
}
