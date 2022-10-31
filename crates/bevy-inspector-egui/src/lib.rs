#![warn(
    clippy::undocumented_unsafe_blocks,
    clippy::missing_safety_doc,
    unsafe_op_in_unsafe_fn
)]
#![allow(clippy::needless_lifetimes)]

//! This crate contains
//! - general purpose machinery for displaying [`Reflect`](bevy_reflect::Reflect) values in [`egui_reflect_inspector`],
//! - a way of associating arbitrary options with fields and enum variants in [`inspector_options`]
//! - utility functions for displaying bevy resource, entities and assets in [`bevy_ecs_inspector`]
//! - some drop-in plugins in [`quick`] to get you started without any code necessary.
//!
//! # Use case 1: Quick plugins
//! These plugins can be easily added to your app, but don't allow for customization of the presentation and content.
//!
//! ## WorldInspectorPlugin
//! Displays the world's entities, resources and assets.
//!
//! ![image of the world inspector](https://raw.githubusercontent.com/jakobhellermann/bevy-inspector-egui/rework/docs/world_inspector.png)
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_inspector_egui::quick::WorldInspectorPlugin;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(WorldInspectorPlugin)
//!         .add_startup_system(setup)
//!         .run();
//! }
//!
//! # fn setup() {}
//! ```
//! ## ResourceInspectorPlugin
//! Display a single resource in a window.
//!
//! ![image of the resource inspector](https://raw.githubusercontent.com/jakobhellermann/bevy-inspector-egui/rework/docs/resource_inspector.png)
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_inspector_egui::prelude::*;
//! use bevy_inspector_egui::quick::ResourceInspectorPlugin;
//!
//! // `InspectorOptions` are completely optional
//! #[derive(Reflect, Resource, Default, InspectorOptions)]
//! #[reflect(Resource, InspectorOptions)]
//! struct Configuration {
//!     name: String,
//!     #[inspector(min = 0.0, max = 1.0)]
//!     option: f32,
//! }
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .init_resource::<Configuration>() // `ResourceInspectorPlugin` won't initialize the resource
//!         .register_type::<Configuration>() // you need to register your type to display it
//!         .add_plugin(ResourceInspectorPlugin::<Configuration>::default())
//!         // also works with built-in resources, as long as they are `Reflect
//!         .add_plugin(ResourceInspectorPlugin::<Time>::default())
//!         .run();
//! }
//! ```
//!
//! # Use case 2: Manual UI
//! The [`quick`] plugins don't allow customization of the egui window or its content, but you can easily build your own UI:
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_egui::EguiPlugin;
//! use bevy_inspector_egui::prelude::*;
//! use std::any::TypeId;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(EguiPlugin)
//!         .add_plugin(bevy_inspector_egui::DefaultInspectorConfigPlugin) // adds default options and `InspectorEguiImpl`s
//!         .add_system(inspector_ui)
//!         .run();
//! }
//!
//! fn inspector_ui(world: &mut World) {
//!     let egui_context = world.resource_mut::<bevy_egui::EguiContext>().ctx_mut().clone();
//!     let type_registry = world.resource::<AppTypeRegistry>().clone();
//!     let type_registry = type_registry.read();
//!
//!     egui::Window::new("UI").show(&egui_context, |ui| {
//!         egui::ScrollArea::vertical().show(ui, |ui| {
//!             // equivalent to `WorldInspectorPlugin`
//!             // bevy_inspector_egui::bevy_ecs_inspector::ui_for_world(world, ui);
//!
//!             egui::CollapsingHeader::new("Materials").show(ui, |ui| {
//!                 bevy_inspector_egui::bevy_ecs_inspector::ui_for_asset(world, TypeId::of::<StandardMaterial>(), ui, &type_registry);
//!             });
//!
//!             ui.heading("Entities");
//!             bevy_inspector_egui::bevy_ecs_inspector::ui_for_world_entities(world, ui, &type_registry);
//!         });
//!     });
//! }
//! ```
//!
//! Pair this with a crate like [`egui_dock`](https://docs.rs/egui_dock/latest/egui_dock/) and you have your own editor in less than 100 lines: [`examples/egui_dock.rs`](https://github.com/jakobhellermann/bevy-inspector-egui/blob/rework/crates/bevy-inspector-egui/examples/egui_dock.rs).
//! ![image of the egui_dock example](https://raw.githubusercontent.com/jakobhellermann/bevy-inspector-egui/rework/docs/egui_dock.png)

pub mod bevy_ecs_inspector;
pub mod egui_reflect_inspector;
pub mod inspector_egui_impls;
pub mod inspector_options;
pub mod quick;
pub mod restricted_world_view;

mod egui_utils;
mod utils;

/// [`bevy_app::Plugin`] used to register default [`struct@InspectorOptions`] and [`InspectorEguiImpl`](crate::inspector_egui_impls::InspectorEguiImpl)s
pub struct DefaultInspectorConfigPlugin;
impl bevy_app::Plugin for DefaultInspectorConfigPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        if app.is_plugin_added::<Self>() {
            return;
        }

        let type_registry = app.world.resource::<bevy_app::AppTypeRegistry>();
        let mut type_registry = type_registry.write();

        inspector_options::default_options::register_default_options(&mut type_registry);
        inspector_egui_impls::register_std_impls(&mut type_registry);
        inspector_egui_impls::register_glam_impls(&mut type_registry);
        inspector_egui_impls::register_bevy_impls(&mut type_registry);
    }
}

#[doc(inline)]
pub use inspector_options::InspectorOptions;

#[doc(hidden)]
pub mod __macro_exports {
    pub use bevy_reflect;
}

/// Reexports of commonly used types
pub mod prelude {
    // for `#[derive(Reflect)] #[reflect(InspectorOptions)]
    pub use crate::inspector_options::InspectorOptions;
    pub use crate::inspector_options::ReflectInspectorOptions;
}
