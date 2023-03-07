//! Easy plugins for showing UI panels.
//!
//! **Pros:** no manual code required
//!
//! **Cons:** not configurable
//!
//! When you want something more custom, you can use these plugins as a starting point.

use std::marker::PhantomData;

use bevy_app::Plugin;
use bevy_asset::Asset;
use bevy_ecs::{prelude::*, query::ReadOnlyWorldQuery};
use bevy_egui::{EguiContext, EguiPlugin};
use bevy_reflect::Reflect;
use bevy_window::PrimaryWindow;
use pretty_type_name::pretty_type_name;

use crate::{bevy_inspector, DefaultInspectorConfigPlugin};

const DEFAULT_SIZE: (f32, f32) = (320., 160.);

/// Plugin displaying a egui window with an entity list, resources and assets
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_inspector_egui::prelude::*;
/// use bevy_inspector_egui::quick::ResourceInspectorPlugin;
///
/// // `InspectorOptions` are completely optional
/// #[derive(Reflect, Resource, Default, InspectorOptions)]
/// #[reflect(Resource, InspectorOptions)]
/// struct Configuration {
///     name: String,
///     #[inspector(min = 0.0, max = 1.0)]
///     option: f32,
/// }
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .init_resource::<Configuration>() // `ResourceInspectorPlugin` won't initialize the resource
///         .register_type::<Configuration>() // you need to register your type to display it
///         .add_plugin(ResourceInspectorPlugin::<Configuration>::default())
///         // also works with built-in resources, as long as they implement `Reflect`
///         .add_plugin(ResourceInspectorPlugin::<Time>::default())
///         .run();
/// }
/// ```
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
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();
    egui::Window::new("World Inspector")
        .default_size(DEFAULT_SIZE)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                bevy_inspector::ui_for_world(world, ui);
                ui.allocate_space(ui.available_size());
            });
        });
}

/// Plugin displaying an egui window for a single resource.
/// Remember to insert the resource and call [`App::register_type`](bevy_app::App::register_type).
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_inspector_egui::prelude::*;
/// use bevy_inspector_egui::quick::ResourceInspectorPlugin;
///
/// // `InspectorOptions` are completely optional
/// #[derive(Reflect, Resource, Default, InspectorOptions)]
/// #[reflect(Resource, InspectorOptions)]
/// struct Configuration {
///     name: String,
///     #[inspector(min = 0.0, max = 1.0)]
///     option: f32,
/// }
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .init_resource::<Configuration>() // `ResourceInspectorPlugin` won't initialize the resource
///         .register_type::<Configuration>() // you need to register your type to display it
///         .add_plugin(ResourceInspectorPlugin::<Configuration>::default())
///         // also works with built-in resources, as long as they implement `Reflect`
///         .add_plugin(ResourceInspectorPlugin::<Time>::default())
///         .run();
/// }
/// ```
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

impl<T: Resource + Reflect> Plugin for ResourceInspectorPlugin<T> {
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

fn inspector_ui<T: Resource + Reflect>(world: &mut World) {
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();
    egui::Window::new(pretty_type_name::<T>())
        .default_size((0., 0.))
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                bevy_inspector::ui_for_resource::<T>(world, ui);

                ui.allocate_space(ui.available_size());
            });
        });
}

/// Plugin displaying an egui window for an app state.
/// Remember to call [`App::add_state`](bevy_app::App::add_state) .
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_inspector_egui::quick::StateInspectorPlugin;
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .insert_resource(ClearColor(Color::BLACK))
///         .add_state(AppState::A)
///         .register_type::<AppState>()
///         .add_plugin(StateInspectorPlugin::<AppState>::default())
///         .run();
/// }
///
/// #[derive(Debug, Clone, Eq, PartialEq, Hash, Reflect)]
/// enum AppState {
///     A,
///     B,
///     C,
/// }
/// ```
pub struct StateInspectorPlugin<T>(PhantomData<fn() -> T>);

impl<T> Default for StateInspectorPlugin<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<T> StateInspectorPlugin<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: States + Reflect> Plugin for StateInspectorPlugin<T> {
    fn build(&self, app: &mut bevy_app::App) {
        if !app.is_plugin_added::<DefaultInspectorConfigPlugin>() {
            app.add_plugin(DefaultInspectorConfigPlugin);
        }
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugin(EguiPlugin);
        }

        app.add_system(state_ui::<T>);
    }
}

fn state_ui<T: States + Reflect>(world: &mut World) {
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();
    egui::Window::new(std::any::type_name::<T>())
        .resizable(false)
        .title_bar(false)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading(pretty_type_name::<T>());
                bevy_inspector::ui_for_state::<T>(world, ui);
            });
        });
}

/// Plugin displaying an egui window for all assets of type `A`.
/// Remember to call [`App::register_asset_reflect`](bevy_asset::AddAsset::register_asset_reflect).
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_inspector_egui::quick::AssetInspectorPlugin;
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugin(AssetInspectorPlugin::<StandardMaterial>::default())
///         .run();
/// }
/// ```
pub struct AssetInspectorPlugin<A>(PhantomData<fn() -> A>);

impl<A> Default for AssetInspectorPlugin<A> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<A> AssetInspectorPlugin<A> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<A: Asset + Reflect> Plugin for AssetInspectorPlugin<A> {
    fn build(&self, app: &mut bevy_app::App) {
        if !app.is_plugin_added::<DefaultInspectorConfigPlugin>() {
            app.add_plugin(DefaultInspectorConfigPlugin);
        }
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugin(EguiPlugin);
        }

        app.add_system(asset_inspector_ui::<A>);
    }
}

fn asset_inspector_ui<A: Asset + Reflect>(world: &mut World) {
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();
    egui::Window::new(pretty_type_name::<A>())
        .default_size(DEFAULT_SIZE)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                bevy_inspector::ui_for_assets::<A>(world, ui);

                ui.allocate_space(ui.available_size());
            });
        });
}

/// Plugin displaying an egui window for all entities matching the filter `F`.
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_inspector_egui::quick::FilterQueryInspectorPlugin;
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugin(FilterQueryInspectorPlugin::<With<Transform>>::default())
///         .run();
/// }
/// ```
pub struct FilterQueryInspectorPlugin<F>(PhantomData<fn() -> F>);

impl<F> Default for FilterQueryInspectorPlugin<F> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<A> FilterQueryInspectorPlugin<A> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<F: 'static> Plugin for FilterQueryInspectorPlugin<F>
where
    F: ReadOnlyWorldQuery,
{
    fn build(&self, app: &mut bevy_app::App) {
        if !app.is_plugin_added::<DefaultInspectorConfigPlugin>() {
            app.add_plugin(DefaultInspectorConfigPlugin);
        }
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugin(EguiPlugin);
        }

        app.add_system(entity_query_ui::<F>);
    }
}

fn entity_query_ui<F: ReadOnlyWorldQuery>(world: &mut World) {
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();
    egui::Window::new(pretty_type_name::<F>())
        .default_size(DEFAULT_SIZE)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                bevy_inspector::ui_for_world_entities_filtered::<F>(world, ui, false);
                ui.allocate_space(ui.available_size());
            });
        });
}
