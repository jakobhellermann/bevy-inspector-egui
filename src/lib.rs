#![warn(missing_docs)]
#![allow(
    clippy::unit_arg,
    clippy::needless_doctest_main,
    clippy::too_many_arguments,
    clippy::collapsible_if
)]

//! This crate provides the ability to annotate structs with a `#[derive(Inspectable)]`,
//! which opens a debug interface using [egui](https://github.com/emilk/egui) where you can visually edit the values of your struct live.
//!
//! Your struct will then be available to you as a bevy resource.
//!
//! ## Example
//! ```rust
//! use bevy_inspector_egui::Inspectable;
//!
//! #[derive(Inspectable, Default)]
//! struct Data {
//!     should_render: bool,
//!     text: String,
//!     #[inspectable(min = 42.0, max = 100.0)]
//!     size: f32,
//! }
//! ```
//! Add the [`InspectorPlugin`] to your App.
//! ```rust,no_run
//! use bevy_inspector_egui::InspectorPlugin;
//! # use bevy::prelude::*;
//!
//! # #[derive(bevy_inspector_egui::Inspectable, Default)] struct Data {}
//! fn main() {
//!     App::build()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(InspectorPlugin::<Data>::new())
//!         .add_system(your_system.system())
//!         .run();
//! }
//!
//! # fn your_system() {}
//! // fn your_system(data: Res<Data>) { /* */ }
//! ```
//!
//! The list of built-in attributes is documented [here](trait.Inspectable.html#default-attributes).
//!
//! ## World Inspector
//!
//! If you want to display all world entities you can add the [`WorldInspectorPlugin`]:
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_inspector_egui::WorldInspectorPlugin;
//!
//! fn main() {
//!     App::build()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(WorldInspectorPlugin::new())
//!         .add_startup_system(setup.system())
//!         .run();
//! }
//! # fn setup() {}
//! ```
//! You can configure it by inserting the [`WorldInspectorParams`] resource.

#[macro_use]
mod utils;

/// Utitly types implementing [`Inspectable`](crate::Inspectable)
pub mod widgets;

#[allow(missing_docs)]
mod impls;
mod plugin;

/// configuration for the [`WorldInspectorPlugin`](crate::world_inspector::WorldInspectorPlugin)
mod world_inspector;

use std::hash::Hasher;

use bevy::prelude::{AppBuilder, World};
use egui::CtxRef;
pub use world_inspector::{InspectableRegistry, WorldInspectorParams, WorldInspectorPlugin};

/// [`Inspectable`] implementation for foreign types implementing [`Reflect`](bevy::reflect::Reflect)
pub mod reflect;

pub use bevy_egui;
pub use bevy_egui::egui;

/// Derives the [`Inspectable`](Inspectable) trait.
pub use bevy_inspector_egui_derive::Inspectable;
pub use plugin::InspectorPlugin;

/// Attributes for the built-in [`Inspectable`](Inspectable) implementations
pub mod options {
    pub use crate::impls::*;
    pub use crate::widgets::button::ButtonAttributes;
    pub use crate::widgets::new_window::WindowAttributes;
    pub use crate::world_inspector::impls::EntityAttributes;
}

/// The context passed to [`Inspectable::ui`].
pub struct Context<'a> {
    /// egui ui context
    pub ui_ctx: Option<&'a CtxRef>,
    /// The world is only available when not using `InspectablePlugin::shared()`
    world: Option<*mut World>,

    /// Something to distinguish between siblings.
    pub id: Option<u64>,
}
impl<'a> Context<'a> {
    /// Gives mutable access to the [bevy::ecs::world::World]
    /// # Safety
    /// The pointer provided in `Context::new_raw` must give unique access.
    pub unsafe fn world(&'a self) -> Option<&'a mut World> {
        self.world.map(|ptr| &mut *ptr)
    }
}

impl<'a> Context<'a> {
    /// Create new context with exclusive access to the `World`
    pub fn new(ui_ctx: &'a CtxRef, world: &'a mut World) -> Self {
        Context {
            ui_ctx: Some(ui_ctx),
            world: Some(world as *mut _),
            id: None,
        }
    }
    /// Create a new context with access to the world
    /// # Safety
    /// The `world` pointer must come from a mutable reference to a world and no
    /// other threads must be writing to it.
    pub unsafe fn new_ptr(ui_ctx: Option<&'a CtxRef>, world: *mut World) -> Self {
        Context {
            ui_ctx,
            world: Some(world),
            id: None,
        }
    }

    /// Creates a context without access to `World`
    pub fn new_shared(ui_ctx: Option<&'a CtxRef>) -> Self {
        Context {
            ui_ctx,
            world: None,
            id: None,
        }
    }

    /// Same context but with a specified `id`
    pub fn with_id(&self, id: u64) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::default();
        if let Some(id) = self.id {
            hasher.write_u64(id);
        }
        hasher.write_u64(id);
        let id = hasher.finish();

        Context {
            id: Some(id),
            ..*self
        }
    }

    /// Returns the [id](struct.Context.html#structfield.id) if present, otherwise a dummy id.
    pub fn id(&self) -> egui::Id {
        let dummy_id = egui::Id::new(42);
        match self.id {
            Some(id) => egui::Id::new(id),
            None => dummy_id,
        }
    }
}

/// This trait describes how a struct should be displayed.
/// It can be derived for structs and enums, see the [crate-level docs](index.html) for how to do that.
///
/// ## Default attributes
/// - **ignore**: hides the field in the inspector
/// - **label**: provides a label instead of using the field name
/// - **read_only**: disables the UI
/// - **collapse**: wraps the ui in an [`egui::CollapsingHeader`].
/// - **default**: only for enums, specifies the default value when selecting a new variant
/// - **wrapper**: wrap field UI in a custom function. Demo in the [rust_types example](https://github.com/jakobhellermann/bevy-inspector-egui/blob/main/examples/rust_types.rs#L20).
pub trait Inspectable {
    /// The `Attributes` associated type specifies what attributes can be passed to a field.
    /// See the following snippet for an example:
    /// ```rust,no_run
    /// # use bevy_inspector_egui::{egui, Inspectable, Context};
    /// struct MyCustomType;
    /// # #[derive(Clone, Default)]
    /// struct MyWidgetAttributes { a: f32, b: Option<String> }
    ///
    /// impl Inspectable for MyCustomType {
    ///   type Attributes = MyWidgetAttributes;
    ///
    ///   fn ui(&mut self, _: &mut egui::Ui, options: MyWidgetAttributes, context: &Context) -> bool {
    ///     println!("a = {}, b = {:?}", options.a, options.b);
    ///     false
    ///   }
    /// }
    ///
    /// // ...
    ///
    /// #[derive(Inspectable)]
    /// struct InspectorData {
    ///   #[inspectable(a = 10.0, b = None)]
    ///   value: MyCustomType,
    /// }
    /// ```
    type Attributes: Default + Clone;

    /// This methods is responsible for building the egui ui.
    /// Returns whether any data was modified.
    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context) -> bool;

    /// Displays the value without any context. Useful for usage outside of the plugins, where
    /// there is no access to the world or [`EguiContext`](bevy_egui::EguiContext).
    fn ui_raw(&mut self, ui: &mut egui::Ui, options: Self::Attributes) {
        let empty_context = Context::new_shared(None);
        self.ui(ui, options, &empty_context);
    }

    /// Required setup for the bevy application, e.g. registering events. Note that this method will run for every instance of a type.
    #[allow(unused_variables)]
    fn setup(app: &mut AppBuilder) {}
}
