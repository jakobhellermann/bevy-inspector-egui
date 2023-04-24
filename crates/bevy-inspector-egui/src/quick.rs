//! Easy plugins for showing UI panels.
//!
//! **Pros:** no manual code required
//!
//! **Cons:** not configurable
//!
//! When you want something more custom, you can use these plugins as a starting point.

use std::{marker::PhantomData, sync::Mutex};

use bevy_app::Plugin;
use bevy_asset::Asset;
use bevy_ecs::prelude::*;
use bevy_ecs::{query::ReadOnlyWorldQuery, schedule::BoxedCondition, system::ReadOnlySystem};
use bevy_egui::{EguiContext, EguiPlugin};
use bevy_reflect::Reflect;
use bevy_window::PrimaryWindow;
use pretty_type_name::pretty_type_name;

use crate::{bevy_inspector, DefaultInspectorConfigPlugin};

const DEFAULT_SIZE: (f32, f32) = (320., 160.);

/// Plugin displaying a egui window with an entity list, resources and assets
///
/// You can use [`WorldInspectorPlugin::run_if`] to control when the window is shown, for example
/// in combination with `input_toggle_active`.
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_inspector_egui::prelude::*;
/// use bevy_inspector_egui::quick::WorldInspectorPlugin;
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugin(WorldInspectorPlugin::new())
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
    pub fn run_if<M>(mut self, condition: impl Condition<M>) -> Self {
        let condition_system = IntoSystem::into_system(condition);
        self.condition = Mutex::new(Some(Box::new(condition_system) as BoxedCondition));
        self
    }
}

impl Plugin for WorldInspectorPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        if !app.is_plugin_added::<DefaultInspectorConfigPlugin>() {
            app.add_plugin(DefaultInspectorConfigPlugin);
        }
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugin(EguiPlugin);
        }

        let condition = self.condition.lock().unwrap().take();
        let mut system = world_inspector_ui.into_config();
        if let Some(condition) = condition {
            system = system.run_if(BoxedConditionHelper(condition));
        }
        app.add_system(system);
    }
}

fn world_inspector_ui(world: &mut World) {
    let egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world);

    let Ok(egui_context) = egui_context else {return;};
    let mut egui_context = egui_context.clone();
    
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
///         .add_plugin(ResourceInspectorPlugin::<Configuration>::default())
///         // also works with built-in resources, as long as they implement `Reflect`
///         .add_plugin(ResourceInspectorPlugin::<Time>::default())
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
    pub fn run_if<M>(mut self, condition: impl Condition<M>) -> Self {
        let condition_system = IntoSystem::into_system(condition);
        self.condition = Mutex::new(Some(Box::new(condition_system) as BoxedCondition));
        self
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

        let condition = self.condition.lock().unwrap().take();
        let mut system = inspector_ui::<T>.into_config();
        if let Some(condition) = condition {
            system = system.run_if(BoxedConditionHelper(condition));
        }
        app.add_system(system);
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
/// You can use [`StateInspectorPlugin::run_if`] to control when the window is shown, for example
/// in combination with `input_toggle_active`.
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_inspector_egui::quick::StateInspectorPlugin;
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .insert_resource(ClearColor(Color::BLACK))
///         .add_state::<AppState>()
///         .register_type::<AppState>()
///         .add_plugin(StateInspectorPlugin::<AppState>::default())
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
    pub fn run_if<M>(mut self, condition: impl Condition<M>) -> Self {
        let condition_system = IntoSystem::into_system(condition);
        self.condition = Mutex::new(Some(Box::new(condition_system) as BoxedCondition));
        self
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

        let condition = self.condition.lock().unwrap().take();
        let mut system = state_ui::<T>.into_config();
        if let Some(condition) = condition {
            system = system.run_if(BoxedConditionHelper(condition));
        }
        app.add_system(system);
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
///         .add_plugin(AssetInspectorPlugin::<StandardMaterial>::default())
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
    pub fn run_if<M>(mut self, condition: impl Condition<M>) -> Self {
        let condition_system = IntoSystem::into_system(condition);
        self.condition = Mutex::new(Some(Box::new(condition_system) as BoxedCondition));
        self
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

        let condition = self.condition.lock().unwrap().take();
        let mut system = asset_inspector_ui::<A>.into_config();
        if let Some(condition) = condition {
            system = system.run_if(BoxedConditionHelper(condition));
        }
        app.add_system(system);
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
    pub fn run_if<M>(mut self, condition: impl Condition<M>) -> Self {
        let condition_system = IntoSystem::into_system(condition);
        self.condition = Mutex::new(Some(Box::new(condition_system) as BoxedCondition));
        self
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

        let condition = self.condition.lock().unwrap().take();
        let mut system = IntoSystemConfig::into_config(entity_query_ui::<F>);
        if let Some(condition) = condition {
            system = system.run_if(BoxedConditionHelper(condition));
        }
        app.add_system(system);
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

struct BoxedConditionHelper(BoxedCondition);
// SAFETY: BoxedCondition is a Box<dyn ReadOnlySystem>
unsafe impl ReadOnlySystem for BoxedConditionHelper {}
impl System for BoxedConditionHelper {
    type In = ();
    type Out = bool;

    fn name(&self) -> std::borrow::Cow<'static, str> {
        self.0.name()
    }

    fn type_id(&self) -> std::any::TypeId {
        self.0.type_id()
    }

    fn component_access(&self) -> &bevy_ecs::query::Access<bevy_ecs::component::ComponentId> {
        self.0.component_access()
    }

    fn archetype_component_access(
        &self,
    ) -> &bevy_ecs::query::Access<bevy_ecs::archetype::ArchetypeComponentId> {
        self.0.archetype_component_access()
    }

    fn is_send(&self) -> bool {
        self.0.is_send()
    }

    fn is_exclusive(&self) -> bool {
        self.0.is_exclusive()
    }

    unsafe fn run_unsafe(&mut self, input: Self::In, world: &World) -> Self::Out {
        // SAFETY: same as this method
        unsafe { self.0.run_unsafe(input, world) }
    }

    fn apply_buffers(&mut self, world: &mut World) {
        self.0.apply_buffers(world)
    }

    fn initialize(&mut self, _world: &mut World) {
        self.0.initialize(_world)
    }

    fn update_archetype_component_access(&mut self, world: &World) {
        self.0.update_archetype_component_access(world)
    }

    fn check_change_tick(&mut self, change_tick: u32) {
        self.0.check_change_tick(change_tick)
    }

    fn get_last_change_tick(&self) -> u32 {
        self.0.get_last_change_tick()
    }

    fn set_last_change_tick(&mut self, last_change_tick: u32) {
        self.0.set_last_change_tick(last_change_tick)
    }
}
