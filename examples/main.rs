use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_system(ui_example.system())
        .run();
}

fn ui_example(mut egui_context: ResMut<EguiContext>) {
    let ctx = &mut egui_context.ctx;
    egui::Window::new("Hello").resizable(false).show(ctx, |ui| {
        ui.label("world");
    });
}
