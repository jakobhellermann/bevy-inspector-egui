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
use bevy_ecs::{prelude::*, schedule::StateData};
use bevy_egui::EguiPlugin;
use bevy_reflect::Reflect;
use pretty_type_name::pretty_type_name;

use crate::{bevy_inspector, DefaultInspectorConfigPlugin};

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
                bevy_inspector::ui_for_world(world, ui);
                ui.allocate_space(ui.available_size());
            });
        });
}

/// Plugin displaying an egui window for a single resource.
/// Remember to insert the resource and call [`App::register_type`](bevy_app::App::register_type).
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
    let egui_context = world
        .resource_mut::<bevy_egui::EguiContext>()
        .ctx_mut()
        .clone();
    egui::Window::new(pretty_type_name::<T>())
        .default_size((0., 0.))
        .show(&egui_context, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                bevy_inspector::ui_for_resource::<T>(world, ui);

                ui.allocate_space(ui.available_size());
            });
        });
}

/// Plugin displaying an egui window for an app state.
/// Remember to call [`App::add_state`](bevy_app::App::add_state) .
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

impl<T: StateData + Reflect> Plugin for StateInspectorPlugin<T> {
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

fn state_ui<T: StateData + Reflect>(world: &mut World) {
    let egui_context = world
        .resource_mut::<bevy_egui::EguiContext>()
        .ctx_mut()
        .clone();
    egui::Window::new(std::any::type_name::<T>())
        .resizable(false)
        .title_bar(false)
        .show(&egui_context, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading(pretty_type_name::<T>());
                bevy_inspector::ui_for_state::<T>(world, ui);
            });
        });
}

/// Plugin displaying an egui window for all assets of type `A`.
/// Remember to call [`App::register_asset_reflect`](bevy_asset::AddAsset::register_asset_reflect).
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
    let egui_context = world
        .resource_mut::<bevy_egui::EguiContext>()
        .ctx_mut()
        .clone();
    egui::Window::new(pretty_type_name::<A>())
        .default_size(DEFAULT_SIZE)
        .show(&egui_context, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                bevy_inspector::ui_for_assets::<A>(world, ui);

                ui.allocate_space(ui.available_size());
            });
        });
}
