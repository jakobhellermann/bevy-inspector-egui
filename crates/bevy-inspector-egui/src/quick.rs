//! Easy plugins for showing UI panels.
//!
//! **Pros:** no manual code required
//!
//! **Cons:** not configurable
//!
//! When you want something more custom, you can use these plugins as a starting point.

use std::{marker::PhantomData, sync::Mutex};

use crate::{bevy_inspector::Filter, utils::pretty_type_name};
use bevy_app::Plugin;
use bevy_asset::Asset;
use bevy_ecs::{prelude::*, query::QueryFilter, schedule::BoxedCondition};
use bevy_egui::{EguiContext, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext};
use bevy_reflect::Reflect;
use bevy_state::state::FreelyMutableState;

use crate::{DefaultInspectorConfigPlugin, bevy_inspector};

const DEFAULT_SIZE: (f32, f32) = (320., 160.);

//TODO: Do we need this with EguiPrimaryContextPass?
// #[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
// struct Inspect;

// struct InspectSchedulePlugin;
// impl Plugin for InspectSchedulePlugin {
//     fn build(&self, app: &mut App) {
//         app.init_schedule(Inspect);

//         app.world_mut()
//             .resource_mut::<MainScheduleOrder>()
//             .insert_after(Update, Inspect);
//     }
// }

/// Plugin displaying a egui window with an entity list, resources and assets
///
/// You can use [`WorldInspectorPlugin::run_if`] to control when the window is shown, for example
/// in combination with `input_toggle_active`.
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_inspector_egui::{bevy_egui::EguiPlugin, prelude::*};
/// use bevy_inspector_egui::quick::WorldInspectorPlugin;
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugins(EguiPlugin::default())
///         .add_plugins(WorldInspectorPlugin::new())
///         .run();
/// }
/// ```
#[derive(Default)]
pub struct WorldInspectorPlugin {
    condition: Mutex<Option<BoxedCondition>>,
}

impl WorldInspectorPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    /// Only show the UI of the specified condition is active
    pub fn run_if<M>(mut self, condition: impl SystemCondition<M>) -> Self {
        let condition_system = IntoSystem::into_system(condition);
        self.condition = Mutex::new(Some(Box::new(condition_system) as BoxedCondition));
        self
    }
}

impl Plugin for WorldInspectorPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        check_plugins(app, "WorldInspectorPlugin");

        if !app.is_plugin_added::<DefaultInspectorConfigPlugin>() {
            app.add_plugins(DefaultInspectorConfigPlugin);
        }

        let condition = self.condition.lock().unwrap().take();
        let mut system = world_inspector_ui.into_configs();
        if let Some(condition) = condition {
            system.run_if_dyn(condition);
        }
        app.add_systems(EguiPrimaryContextPass, system);
    }
}

fn world_inspector_ui(world: &mut World) {
    let egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single(world);

    let Ok(egui_context) = egui_context else {
        return;
    };
    let mut egui_context = egui_context.clone();

    egui::Window::new("World Inspector")
        .default_size(DEFAULT_SIZE)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                bevy_inspector::ui_for_world(world, ui);
                ui.allocate_space(ui.available_size());
            });
        });
}

/// Plugin displaying an egui window for a single resource.
/// Remember to insert the resource and call [`App::register_type`](bevy_app::App::register_type).
///
/// You can use [`ResourceInspectorPlugin::run_if`] to control when the window is shown, for example
/// in combination with `input_toggle_active`.
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
///         .add_plugins(ResourceInspectorPlugin::<Configuration>::default())
///         // also works with built-in resources, as long as they implement `Reflect`
///         .add_plugins(ResourceInspectorPlugin::<Time>::default())
///         .run();
/// }
/// ```
pub struct ResourceInspectorPlugin<T> {
    condition: Mutex<Option<BoxedCondition>>,
    marker: PhantomData<fn() -> T>,
}

impl<T> Default for ResourceInspectorPlugin<T> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
            condition: Mutex::new(None),
        }
    }
}

impl<T> ResourceInspectorPlugin<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Only show the UI of the specified condition is active
    pub fn run_if<M>(mut self, condition: impl SystemCondition<M>) -> Self {
        let condition_system = IntoSystem::into_system(condition);
        self.condition = Mutex::new(Some(Box::new(condition_system) as BoxedCondition));
        self
    }
}

impl<T: Resource + Reflect> Plugin for ResourceInspectorPlugin<T> {
    fn build(&self, app: &mut bevy_app::App) {
        check_plugins(app, "ResourceInspectorPlugin");

        if !app.is_plugin_added::<DefaultInspectorConfigPlugin>() {
            app.add_plugins(DefaultInspectorConfigPlugin);
        }

        let condition = self.condition.lock().unwrap().take();
        let mut system = inspector_ui::<T>.into_configs();
        if let Some(condition) = condition {
            system.run_if_dyn(condition);
        }
        app.add_systems(EguiPrimaryContextPass, system);
    }
}

fn inspector_ui<T: Resource + Reflect>(world: &mut World) {
    let egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single(world);

    let Ok(egui_context) = egui_context else {
        return;
    };
    let mut egui_context = egui_context.clone();

    egui::Window::new(pretty_type_name::<T>())
        .default_size((0., 0.))
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                bevy_inspector::ui_for_resource::<T>(world, ui);

                ui.allocate_space(ui.available_size());
            });
        });
}

/// Plugin displaying an egui window for an app state.
/// Remember to call [`App::add_state`](bevy_app::App::init_state).
///
/// You can use [`StateInspectorPlugin::run_if`] to control when the window is shown, for example
/// in combination with `input_toggle_active`.
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_inspector_egui::quick::StateInspectorPlugin;
/// use bevy_state::{app::AppExtStates, prelude::States, state::FreelyMutableState};
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .insert_resource(ClearColor(Color::BLACK))
///         .init_state::<AppState>()
///         .register_type::<AppState>()
///         .add_plugins(StateInspectorPlugin::<AppState>::default())
///         .run();
/// }
///
/// #[derive(Default, States, Debug, Clone, Eq, PartialEq, Hash, Reflect)]
/// enum AppState {
///     #[default]
///     A,
///     B,
///     C,
/// }
/// ```
pub struct StateInspectorPlugin<T> {
    condition: Mutex<Option<BoxedCondition>>,
    marker: PhantomData<fn() -> T>,
}

impl<T> Default for StateInspectorPlugin<T> {
    fn default() -> Self {
        StateInspectorPlugin {
            condition: Mutex::new(None),
            marker: PhantomData,
        }
    }
}
impl<T> StateInspectorPlugin<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Only show the UI of the specified condition is active
    pub fn run_if<M>(mut self, condition: impl SystemCondition<M>) -> Self {
        let condition_system = IntoSystem::into_system(condition);
        self.condition = Mutex::new(Some(Box::new(condition_system) as BoxedCondition));
        self
    }
}

impl<T: FreelyMutableState + Reflect> Plugin for StateInspectorPlugin<T> {
    fn build(&self, app: &mut bevy_app::App) {
        check_plugins(app, "StateInspectorPlugin");

        if !app.is_plugin_added::<DefaultInspectorConfigPlugin>() {
            app.add_plugins(DefaultInspectorConfigPlugin);
        }

        let condition = self.condition.lock().unwrap().take();
        let mut system = state_ui::<T>.into_configs();
        if let Some(condition) = condition {
            system.run_if_dyn(condition);
        }
        app.add_systems(EguiPrimaryContextPass, system);
    }
}

fn state_ui<T: FreelyMutableState + Reflect>(world: &mut World) {
    let egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single(world);

    let Ok(egui_context) = egui_context else {
        return;
    };
    let mut egui_context = egui_context.clone();

    egui::Window::new(std::any::type_name::<T>())
        .resizable(false)
        .title_bar(false)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.heading(pretty_type_name::<T>());
                bevy_inspector::ui_for_state::<T>(world, ui);
            });
        });
}

/// Plugin displaying an egui window for all assets of type `A`.
/// Remember to call [`App::register_asset_reflect`](bevy_asset::AssetApp::register_asset_reflect).
///
/// You can use [`AssetInspectorPlugin::run_if`] to control when the window is shown, for example
/// in combination with `input_toggle_active`.
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_inspector_egui::quick::AssetInspectorPlugin;
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugins(AssetInspectorPlugin::<StandardMaterial>::default())
///         .run();
/// }
/// ```
pub struct AssetInspectorPlugin<A> {
    condition: Mutex<Option<BoxedCondition>>,
    marker: PhantomData<fn() -> A>,
}

impl<A> Default for AssetInspectorPlugin<A> {
    fn default() -> Self {
        Self {
            condition: Mutex::new(None),
            marker: PhantomData,
        }
    }
}
impl<A> AssetInspectorPlugin<A> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Only show the UI of the specified condition is active
    pub fn run_if<M>(mut self, condition: impl SystemCondition<M>) -> Self {
        let condition_system = IntoSystem::into_system(condition);
        self.condition = Mutex::new(Some(Box::new(condition_system) as BoxedCondition));
        self
    }
}

impl<A: Asset + Reflect> Plugin for AssetInspectorPlugin<A> {
    fn build(&self, app: &mut bevy_app::App) {
        check_plugins(app, "AssetInspectorPlugin");

        if !app.is_plugin_added::<DefaultInspectorConfigPlugin>() {
            app.add_plugins(DefaultInspectorConfigPlugin);
        }

        let condition = self.condition.lock().unwrap().take();
        let mut system = asset_inspector_ui::<A>.into_configs();
        if let Some(condition) = condition {
            system.run_if_dyn(condition);
        }
        app.add_systems(EguiPrimaryContextPass, system);
    }
}

fn asset_inspector_ui<A: Asset + Reflect>(world: &mut World) {
    let egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single(world);

    let Ok(egui_context) = egui_context else {
        return;
    };
    let mut egui_context = egui_context.clone();

    egui::Window::new(pretty_type_name::<A>())
        .default_size(DEFAULT_SIZE)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
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
///         .add_plugins(FilterQueryInspectorPlugin::<With<Transform>>::default())
///         .run();
/// }
/// ```
pub struct FilterQueryInspectorPlugin<F> {
    condition: Mutex<Option<BoxedCondition>>,
    marker: PhantomData<fn() -> F>,
}

impl<F> Default for FilterQueryInspectorPlugin<F> {
    fn default() -> Self {
        Self {
            condition: Mutex::new(None),
            marker: PhantomData,
        }
    }
}
impl<A> FilterQueryInspectorPlugin<A> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Only show the UI of the specified condition is active
    pub fn run_if<M>(mut self, condition: impl SystemCondition<M>) -> Self {
        let condition_system = IntoSystem::into_system(condition);
        self.condition = Mutex::new(Some(Box::new(condition_system) as BoxedCondition));
        self
    }
}

impl<F: 'static> Plugin for FilterQueryInspectorPlugin<F>
where
    F: QueryFilter,
{
    fn build(&self, app: &mut bevy_app::App) {
        check_plugins(app, "FilterQueryInspectorPlugin");

        if !app.is_plugin_added::<DefaultInspectorConfigPlugin>() {
            app.add_plugins(DefaultInspectorConfigPlugin);
        }

        let condition: Option<Box<dyn ReadOnlySystem<In = (), Out = bool>>> =
            self.condition.lock().unwrap().take();
        let mut system = entity_query_ui::<F>.into_configs();
        if let Some(condition) = condition {
            system.run_if_dyn(condition);
        }
        app.add_systems(EguiPrimaryContextPass, system);
    }
}

fn entity_query_ui<F: QueryFilter>(world: &mut World) {
    let egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single(world);

    let Ok(egui_context) = egui_context else {
        return;
    };
    let mut egui_context = egui_context.clone();

    egui::Window::new(pretty_type_name::<F>())
        .default_size(DEFAULT_SIZE)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                bevy_inspector::ui_for_entities_filtered(world, ui, false, &Filter::<F>::all());
                ui.allocate_space(ui.available_size());
            });
        });
}

fn check_plugins(app: &bevy_app::App, name: &str) {
    if !app.is_plugin_added::<bevy_app::MainSchedulePlugin>() {
        panic!(
            r#"`{name}` should be added after the default plugins:
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins({name}::default())
            "#,
        );
    }

    if !app.is_plugin_added::<EguiPlugin>() {
        panic!(
            r#"`{name}` needs to be added after `EguiPlugin`:
        .add_plugins(EguiPlugin::default())
        .add_plugins({name}::default())
            "#,
        );
    }
}
