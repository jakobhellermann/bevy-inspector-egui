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
/// fn setup(commands: &mut Commands) {
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
        if app.resources().get::<EguiContext>().is_none() {
            app.add_plugin(EguiPlugin);
        }

        let resources = app.resources_mut();
        resources.get_or_insert_with(WorldInspectorParams::default);
        resources.get_or_insert_with(InspectableRegistry::default);

        app.add_system(world_inspector_ui.exclusive_system());
    }
}

fn world_inspector_ui(world: &mut World, resources: &mut Resources) {
    let params = &*resources.get::<WorldInspectorParams>().unwrap();

    let egui_context = resources.get::<EguiContext>().expect("EguiContext");
    let ctx = &egui_context.ctx;

    egui::Window::new("World").scroll(true).show(ctx, |ui| {
        let ui_context = WorldUIContext::new(ctx, world, resources);
        ui_context.ui(ui, params);
    });
}
