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
///
/// To be able to edit custom components in inspector, they need to be registered first with
/// [`crate::InspectableRegistry`], to do that they need to implement [`crate::Inspectable`].
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_inspector_egui::{Inspectable, InspectableRegistry};
///
/// #[derive(Inspectable)]
/// pub struct MyComponent {
///     foo: f32,
///     bar: usize
/// }
///
/// pub struct MyPlugin;
///
/// impl Plugin for MyPlugin {
///     fn build(&self, app: &mut AppBuilder) {
///         let mut registry = app
///             .world_mut()
///             .get_resource_or_insert_with(InspectableRegistry::default);
///
///         registry.register::<MyComponent>();
///     }
/// }
/// ```
///
/// Components can be registered in `main` function aswell, just use your [`bevy::app::AppBuilder`]
/// instance to do so.

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

    let egui_context = world.get_resource::<EguiContext>().expect("EguiContext");
    let ctx = {
        let params = world.get_resource::<WorldInspectorParams>().unwrap();
        if !params.enabled {
            return;
        }
        let ctx = match egui_context.try_ctx_for_window(params.window) {
            Some(ctx) => ctx,
            None => return,
        };
        ctx
    };
    let world: &mut World = unsafe { &mut *world_ptr };
    let mut params = world.get_resource_mut::<WorldInspectorParams>().unwrap();

    let mut is_open = true;
    egui::Window::new("World")
        .open(&mut is_open)
        .scroll(true)
        .show(ctx, |ui| {
            crate::plugin::default_settings(ui);
            let world: &mut World = unsafe { &mut *world_ptr };
            let mut ui_context = WorldUIContext::new(world, Some(egui_context.ctx()));
            ui_context.world_ui::<F>(ui, &mut params);
        });

    if !is_open {
        world
            .get_resource_mut::<WorldInspectorParams>()
            .unwrap()
            .enabled = false;
    }
}
