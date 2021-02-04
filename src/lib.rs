#![warn(missing_docs)]

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

#[macro_use]
mod utils;

/// Utitly types implementing [`Inspectable`](crate::Inspectable)
pub mod widgets;

#[allow(missing_docs)]
mod impls;
mod plugin;

/// configuration for the [`WorldInspectorPlugin`](crate::world_inspector::WorldInspectorPlugin)
mod world_inspector;

use bevy::prelude::{AppBuilder, Resources, World};
pub use world_inspector::{InspectableRegistry, WorldInspectorParams, WorldInspectorPlugin};

/// `Inspectable` implementation for foreign types implementing `Reflect`
mod reflect;

pub use bevy_egui;
pub use bevy_egui::egui;

/// Derives the [`Inspectable`](Inspectable) trait.
pub use bevy_inspector_egui_derive::Inspectable;
pub use plugin::InspectorPlugin;

/// Attributes for the built-in [`Inspectable`](Inspectable) implementations
pub mod options {
    pub use crate::impls::*;
    pub use crate::widgets::button::ButtonAttributes;
}

#[derive(Default)]
/// The context passed to [`Inspectable::ui`].
pub struct Context<'a> {
    /// The resources are only available when not using `InspectablePlugin::shared()`
    pub resources: Option<&'a Resources>,
    /// The world is only available when not using `InspectablePlugin::shared()`
    pub world: Option<&'a World>,

    /// Something to distinguish between siblings.
    pub id: Option<u64>,
}

impl<'a> Context<'a> {
    /// Create new context with access to the `World` and `Resources`
    pub fn new(world: &'a World, resources: &'a Resources) -> Self {
        Context {
            resources: Some(resources),
            world: Some(world),
            id: None,
        }
    }

    /// Same context but with a specified `id`
    pub fn with_id(&self, id: u64) -> Self {
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
pub trait Inspectable {
    /// The `Attributes` associated type specifies what attributes can be passed to a field.
    /// See the following snippet for an example:
    /// ```rust,no_run
    /// # use bevy_inspector_egui::{egui, Inspectable, Context};
    /// struct MyCustomType;
    /// # #[derive(Default)]
    /// struct MyWidgetAttributes { a: f32, b: Option<String> }
    ///
    /// impl Inspectable for MyCustomType {
    ///   type Attributes = MyWidgetAttributes;
    ///
    ///   fn ui(&mut self, _: &mut egui::Ui, options: MyWidgetAttributes, context: &Context) {
    ///     println!("a = {}, b = {:?}", options.a, options.b);
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
    type Attributes: Default;

    /// This methods is responsible for building the egui ui.
    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context);

    /// Required setup for the bevy application, e.g. registering Resources.
    #[allow(unused_variables)]
    fn setup(app: &mut AppBuilder) {}
}
