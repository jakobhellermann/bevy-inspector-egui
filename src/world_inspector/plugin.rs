use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};

use super::{WorldInspectorParams, WorldUIContext};
use crate::InspectableRegistry;

/// Plugin for displaying an inspector window of all entites in the world and their components.
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_inspector_egui::WorldInspectorPlugin;
///
/// fn main() {
///     App::build()
///         .add_plugins(DefaultPlugins)
///         .add_plugin(WorldInspectorPlugin)
///         .add_startup_system(setup.system())
///         .run();
/// }
///
/// fn setup(mut commands: Commands) {
///   // setup your scene
///   // adding `Name` components will make the inspector more readable
/// }
/// ```
#[derive(Default)]
pub struct WorldInspectorPlugin;

impl WorldInspectorPlugin {
    /// Create new `WorldInpsectorPlugin`
    pub fn new() -> Self {
        WorldInspectorPlugin
    }
}

impl Plugin for WorldInspectorPlugin {
    fn build(&self, app: &mut AppBuilder) {
        if !app.world_mut().contains_resource::<EguiContext>() {
            app.add_plugin(EguiPlugin);
        }

        let world = app.world_mut();
        world.get_resource_or_insert_with(WorldInspectorParams::default);
        world.get_resource_or_insert_with(InspectableRegistry::default);

        app.add_system(world_inspector_ui.exclusive_system());
    }
}

fn world_inspector_ui(world: &mut World) {
    let mut params = world.get_resource_mut::<WorldInspectorParams>().unwrap();
    let params = std::mem::replace(&mut *params, WorldInspectorParams::empty());
    if !params.enabled {
        return;
    }

    let world_ptr = world as *mut _;

    let egui_context = world.get_resource::<EguiContext>().expect("EguiContext");
    let ctx = &egui_context.ctx;

    egui::Window::new("World").scroll(true).show(ctx, |ui| {
        crate::plugin::default_settings(ui);
        let world: &mut World = unsafe { &mut *world_ptr };
        let mut ui_context = WorldUIContext::new(ctx, world);
        ui_context.world_ui(ui, &params);
    });

    *world.get_resource_mut::<WorldInspectorParams>().unwrap() = params;
}
