use bevy::{platform::collections::HashMap, prelude::*};
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext};
use bevy_inspector_egui::{
    DefaultInspectorConfigPlugin, bevy_egui::EguiContext,
    inspector_options::std_options::NumberDisplay, prelude::*,
};

#[derive(Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
struct Config {
    // `f32` uses `NumberOptions<f32>`
    #[inspector(min = 10.0, max = 70.0, display = NumberDisplay::Slider)]
    font_size: f32,
    #[inspector(min = -1.0, speed = 0.001)] // you can specify inner options for `Option<T>`
    option: Option<f32>,
    #[inspector(min = 10, max = 20)] // same for Vec<T>
    vec: Vec<u32>,
    hash_map: HashMap<u32, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            font_size: 0.,
            option: None,
            vec: Vec::default(),
            hash_map: [(0, "foo".to_owned()), (1, "bar".to_owned())]
                .into_iter()
                .collect(),
        }
    }
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
        .add_plugins(EguiPlugin::default())
        // types need to be registered
        .init_resource::<UiData>()
        .register_type::<Config>()
        .register_type::<Shape>()
        .register_type::<UiData>()
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, ui_example)
        .run();
}

fn setup(mut commands: Commands, mut ui_data: ResMut<UiData>) {
    let entity = commands.spawn(Mesh3d::default()).id();
    ui_data.entity = Some(entity);
}

fn ui_example(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single(world)
    else {
        return;
    };
    let mut ctx = egui_context.clone();
    egui::Window::new("UI").show(ctx.get_mut(), |ui| {
        egui::ScrollArea::both().show(ui, |ui| {
            bevy_inspector_egui::bevy_inspector::ui_for_resource::<UiData>(world, ui);
        });
    });
}
