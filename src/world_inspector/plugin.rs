use std::marker::PhantomData;

use bevy::{
    ecs::query::{FilterFetch, WorldQuery},
    prelude::*,
};
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
///         .add_plugin(WorldInspectorPlugin::new())
///         .add_startup_system(setup.system())
///         .run();
/// }
///
/// fn setup(mut commands: Commands) {
///   // setup your scene
///   // adding `Name` components will make the inspector more readable
/// }
/// ```
pub struct WorldInspectorPlugin<F = ()>(PhantomData<fn() -> F>);
impl Default for WorldInspectorPlugin {
    fn default() -> Self {
        WorldInspectorPlugin::new()
    }
}

impl WorldInspectorPlugin {
    /// Create new `WorldInpsectorPlugin`
    pub fn new() -> Self {
        WorldInspectorPlugin(PhantomData)
    }

    /// Constrain the world inspector to only show entities matching the query filter `F`
    ///
    /// ```rust,no_run
    /// # use bevy::prelude::*;
    /// # use bevy_inspector_egui::WorldInspectorPlugin;
    /// struct Show;
    ///
    /// App::build()
    ///   .add_plugin(WorldInspectorPlugin::new().filter::<With<Show>>())
    ///   .run();
    /// ```
    pub fn filter<F>(self) -> WorldInspectorPlugin<F> {
        WorldInspectorPlugin(PhantomData)
    }
}

impl<F> Plugin for WorldInspectorPlugin<F>
where
    F: WorldQuery + 'static,
    F::Fetch: FilterFetch,
{
    fn build(&self, app: &mut AppBuilder) {
        if !app.world_mut().contains_resource::<EguiContext>() {
            app.add_plugin(EguiPlugin);
        }

        let world = app.world_mut();
        world.get_resource_or_insert_with(WorldInspectorParams::default);
        world.get_resource_or_insert_with(InspectableRegistry::default);

        app.add_system(world_inspector_ui::<F>.exclusive_system());
    }
}

fn world_inspector_ui<F>(world: &mut World)
where
    F: WorldQuery,
    F::Fetch: FilterFetch,
{
    let world_ptr = world as *mut _;

    let params = world.get_resource::<WorldInspectorParams>().unwrap();
    if !params.enabled {
        return;
    }

    let egui_context = world.get_resource::<EguiContext>().expect("EguiContext");
    let mut is_open = true;
    egui::Window::new("World")
        .open(&mut is_open)
        .scroll(true)
        .show(egui_context.ctx(), |ui| {
            crate::plugin::default_settings(ui);
            let world: &mut World = unsafe { &mut *world_ptr };
            let mut ui_context = WorldUIContext::new(Some(egui_context.ctx()), world);
            ui_context.world_ui::<F>(ui, &params);
        });

    if !is_open {
        world
            .get_resource_mut::<WorldInspectorParams>()
            .unwrap()
            .enabled = false;
    }
}
