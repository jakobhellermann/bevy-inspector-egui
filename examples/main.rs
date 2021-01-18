use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_inspector_egui::{InspectableWidget, Options};

#[derive(Debug, Default)]
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

impl InspectableWidget for Data {
    type FieldOptions = ();

    fn ui(&mut self, ui: &mut egui::Ui, options: Options<Self::FieldOptions>) {
        let grid = egui::Grid::new("id").striped(true);
        grid.show(ui, |ui| {
            ui.label("font_size:");
            let custom_options = <f32 as InspectableWidget>::FieldOptions::default();
            let options = Options::new("font_size", custom_options);
            <f32 as InspectableWidget>::ui(&mut self.font_size, ui, options);
            ui.end_row();

            ui.label("text:");
            let custom_options = <String as InspectableWidget>::FieldOptions::default();
            let options = Options::new("text", custom_options);
            <String as InspectableWidget>::ui(&mut self.text, ui, options);
            ui.end_row();

            ui.label("show_square:");
            let custom_options = <bool as InspectableWidget>::FieldOptions::default();
            let options = Options::new("show_square", custom_options);
            <bool as InspectableWidget>::ui(&mut self.show_square, ui, options);
            ui.end_row();
        });
    }
}

impl Data {
    fn inspector_window(&mut self, ctx: &mut egui::CtxRef) {
        egui::Window::new("Inspector")
            .resizable(false)
            .show(ctx, |ui| {
                self.ui(ui, Options::default("test"));
            });
    }
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
    data.inspector_window(&mut egui_context.ctx);
}

fn data(data: ChangedRes<Data>) {
    dbg!(data);
}
