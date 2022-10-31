use bevy::prelude::*;
use bevy_inspector_egui::{prelude::*, DefaultInspectorConfigPlugin};
use std::any::TypeId;

#[derive(Reflect, Default, InspectorOptions)]
#[reflect(InspectorOptions)]
struct Config {
    // `f32` uses `NumberOptions<f32>`
    #[inspector(min = 10.0, max = 70.0)]
    font_size: f32,
    option: Option<f32>,
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
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(DefaultInspectorConfigPlugin)
        .add_plugin(bevy_egui::EguiPlugin)
        // types need to be registered
        .init_resource::<UiData>()
        .register_type::<Config>()
        .register_type::<Shape>()
        .register_type::<UiData>()
        .add_system(ui_example)
        .run();
}

fn ui_example(world: &mut World) {
    let egui_context = world
        .resource_mut::<bevy_egui::EguiContext>()
        .ctx_mut()
        .clone();
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = type_registry.read();

    egui::Window::new("UI").show(&egui_context, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            bevy_inspector_egui::bevy_ecs_inspector::ui_for_resource(
                world,
                TypeId::of::<UiData>(),
                ui,
                &type_registry,
            );
        });
    });
}
