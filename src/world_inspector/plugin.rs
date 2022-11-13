use std::marker::PhantomData;

use bevy::{ecs::query::ReadOnlyWorldQuery, prelude::*};
use bevy_egui::{egui, EguiContext, EguiPlugin};

use super::{WorldInspectorParams, WorldUIContext};
use crate::InspectableRegistry;

/// Plugin for displaying an inspector window of all entites in the world and their components.
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_inspector_egui::WorldInspectorPlugin;
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugin(WorldInspectorPlugin::new())
///         .add_startup_system(setup)
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
///     fn build(&self, app: &mut App) {
///         let mut registry = app
///             .world
///             .get_resource_or_insert_with(InspectableRegistry::default);
///
///         registry.register::<MyComponent>();
///     }
/// }
/// ```
///
/// Components can be registered in `main` function aswell, just use your [`bevy::app::App`]
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
    /// #[derive(Component)]
    /// struct Show;
    ///
    /// App::new()
    ///   .add_plugin(WorldInspectorPlugin::new().filter::<With<Show>>())
    ///   .run();
    /// ```
    pub fn filter<F>(self) -> WorldInspectorPlugin<F> {
        WorldInspectorPlugin(PhantomData)
    }
}

impl<F> Plugin for WorldInspectorPlugin<F>
where
    F: ReadOnlyWorldQuery + 'static,
{
    fn build(&self, app: &mut App) {
        if !app.world.contains_resource::<EguiContext>() {
            app.add_plugin(EguiPlugin);
        }

        let world = &mut app.world;
        world.get_resource_or_insert_with(WorldInspectorParams::default);
        world.get_resource_or_insert_with(InspectableRegistry::default);

        app.add_system(world_inspector_ui::<F>);
    }
}

fn world_inspector_ui<F>(world: &mut World)
where
    F: ReadOnlyWorldQuery,
{
    let world_ptr = world as *mut _;

    let window_id = {
        let params = world.get_resource::<WorldInspectorParams>().unwrap();
        if !params.enabled {
            return;
        }
        params.window
    };

    let mut egui_context = world
        .get_resource_mut::<EguiContext>()
        .expect("EguiContext");
    let ctx = {
        match egui_context.try_ctx_for_window_mut(window_id) {
            Some(ctx) => ctx,
            _ => return,
        }
    };
    let world: &mut World = unsafe { &mut *world_ptr };
    let mut params = world.get_resource_mut::<WorldInspectorParams>().unwrap();

    let mut is_open = true;
    egui::Window::new("World")
        .open(&mut is_open)
        .vscroll(true)
        .show(ctx, |ui| {
            crate::plugin::default_settings(ui);
            let world: &mut World = unsafe { &mut *world_ptr };
            let mut ui_context = WorldUIContext::new(world, Some(ctx));
            ui_context.world_ui::<F>(ui, &mut params);
        });

    if !is_open {
        world
            .get_resource_mut::<WorldInspectorParams>()
            .unwrap()
            .enabled = false;
    }
}
