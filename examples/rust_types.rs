use bevy::prelude::*;
use bevy_inspector_egui::{egui, Inspectable, InspectorPlugin};

#[derive(Inspectable, Default)]
struct Data {
    #[inspectable(min = 10.0, max = 70.0, suffix = "pt")]
    font_size: f32,
    #[inspectable(label = "Display Square")]
    show_square: bool,
    color: Color,
    #[inspectable(visual, min = Vec2::new(-200., -200.), max = Vec2::new(200., 200.))]
    position: Vec2,
    #[inspectable(min = 42.0, max = 100.0)] // attributes get passed to each child
    list: [f32; 2],
    custom_enum: CustomEnum,
    vector: Vec<String>,
    #[inspectable(min = Vec3::ZERO, max = Vec3::splat(128.0))]
    vec3: Vec3,
    text: String,
    #[inspectable(wrapper = change_bg_color)]
    transform: Transform,
    #[inspectable(read_only)]
    disabled: f32,
    #[inspectable(collapse)]
    noise_settings: NoiseSettings,
    #[allow(unused)]
    #[inspectable(ignore)]
    non_inspectable: NonInspectable,
}

fn change_bg_color(
    ui: &mut egui::Ui,
    mut content: impl FnMut(&mut egui::Ui),
) {
    ui.scope(|ui| {
        let bg_color = egui::Color32::from_rgb(41, 80, 80);
        ui.style_mut().visuals.widgets.inactive.bg_fill = bg_color;
        ui.style_mut().visuals.widgets.active.bg_fill = bg_color;
        ui.style_mut().visuals.widgets.hovered.bg_fill = bg_color;
        ui.style_mut().visuals.widgets.noninteractive.bg_fill = bg_color;
        content(ui);
    });
}

#[derive(Inspectable)]
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

#[derive(Inspectable, Default)]
struct NoiseSettings {
    #[inspectable(max = 8)]
    octaves: u8,
    frequency: f32,
    lacunarity: f32,
    persistence: f32,
}

#[derive(Default)]
struct NonInspectable;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<Data>::new())
        .run();
}
