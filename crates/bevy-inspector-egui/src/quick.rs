//! Easy plugins for showing UI panels.
//!
//! **Pros:** no manual code required
//!
//! **Cons:** not configurable
//!
//! When you want something more custom, you can use these plugins as a starting point.

use std::marker::PhantomData;

use bevy_app::{AppTypeRegistry, Plugin};
use bevy_ecs::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_reflect::Reflect;
use pretty_type_name::pretty_type_name;

use crate::{bevy_ecs_inspector, DefaultInspectorConfigPlugin};

const DEFAULT_SIZE: (f32, f32) = (320., 160.);

/// Plugin displaying a egui window with an entity list, resources and assets
pub struct WorldInspectorPlugin;

impl Plugin for WorldInspectorPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        if !app.is_plugin_added::<DefaultInspectorConfigPlugin>() {
            app.add_plugin(DefaultInspectorConfigPlugin);
        }
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugin(EguiPlugin);
        }

        app.add_system(world_inspector_ui);
    }
}

fn world_inspector_ui(world: &mut World) {
    let egui_context = world
        .resource_mut::<bevy_egui::EguiContext>()
        .ctx_mut()
        .clone();
    egui::Window::new("World Inspector")
        .default_size(DEFAULT_SIZE)
        .show(&egui_context, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                bevy_ecs_inspector::ui_for_world(world, ui);
                ui.allocate_space(ui.available_size());
            });
        });
}

/// Plugin displaying a egui window for a single resource.
/// Remember to call insert the resource and call [`App::register_type`](bevy_app::App::register_type).
pub struct ResourceInspectorPlugin<T>(PhantomData<fn() -> T>);

impl<T> Default for ResourceInspectorPlugin<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<T> ResourceInspectorPlugin<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Reflect> Plugin for ResourceInspectorPlugin<T> {
    fn build(&self, app: &mut bevy_app::App) {
        if !app.is_plugin_added::<DefaultInspectorConfigPlugin>() {
            app.add_plugin(DefaultInspectorConfigPlugin);
        }
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugin(EguiPlugin);
        }

        app.add_system(inspector_ui::<T>);
    }
}

fn inspector_ui<T: Reflect>(world: &mut World) {
    let egui_context = world
        .resource_mut::<bevy_egui::EguiContext>()
        .ctx_mut()
        .clone();
    egui::Window::new(pretty_type_name::<T>())
        .default_size((0., 0.))
        .show(&egui_context, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let type_registry = world.resource::<AppTypeRegistry>().clone();
                let type_registry = type_registry.read();
                bevy_ecs_inspector::ui_for_resource(
                    world,
                    std::any::TypeId::of::<T>(),
                    ui,
                    &type_registry,
                );

                ui.allocate_space(ui.available_size());
            });
        });
}
