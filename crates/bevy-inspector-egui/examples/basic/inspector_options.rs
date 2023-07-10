use bevy::prelude::*;
use bevy_egui::EguiContext;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::{prelude::*, DefaultInspectorConfigPlugin};
use bevy_pbr::PbrBundle;
use bevy_window::PrimaryWindow;

#[derive(Reflect, Default, InspectorOptions)]
#[reflect(InspectorOptions)]
struct Config {
    // `f32` uses `NumberOptions<f32>`
    #[inspector(min = 10.0, max = 70.0, display = NumberDisplay::Slider)]
    font_size: f32,
    #[inspector(min = -1.0, speed = 0.001)] // you can specify inner options for `Option<T>`
    option: Option<f32>,
    #[inspector(min = 10, max = 20)] // same for Vec<T>
    vec: Vec<u32>,
}

// Enums can be have `InspectorOptions` as well.
// Note that in order to switch to another enum variant, all its fields need to have [`ReflectDefault`] type data.
#[derive(Default, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
enum Shape {
    Box {
        size: Vec3,
    },
    Icosphere {
        #[inspector(min = 1)]
        subdivisions: usize,
        #[inspector(min = 0.1)]
        radius: f32,
    },
    Capsule {
        radius: f32,
        rings: usize,
        depth: f32,
        latitudes: usize,
        longitudes: usize,
    },
    Line(Vec2, Vec2),
    #[default]
    UnitSphere,
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
struct UiData {
    config: Config,
    shape: Shape,
    entity: Option<Entity>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultInspectorConfigPlugin)
        .add_plugins(bevy_egui::EguiPlugin)
        // types need to be registered
        .init_resource::<UiData>()
        .register_type::<Config>()
        .register_type::<Shape>()
        .register_type::<UiData>()
        .add_systems(Startup, setup)
        .add_systems(Update, ui_example)
        .run();
}

fn setup(mut commands: Commands, mut ui_data: ResMut<UiData>) {
    let entity = commands.spawn(PbrBundle::default()).id();
    ui_data.entity = Some(entity);
}

fn ui_example(world: &mut World) {
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();

    egui::Window::new("UI").show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            bevy_inspector_egui::bevy_inspector::ui_for_resource::<UiData>(world, ui);
        });
    });
}
