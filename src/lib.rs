#![forbid(unsafe_code)]
#![warn(missing_docs, unreachable_pub, missing_debug_implementations)]

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

#[allow(missing_docs)]
mod impls;
mod plugin;

/// `Inspectable` implementation for foreign types implementing `Reflect`
pub mod reflect;

pub use bevy_egui;
pub use bevy_egui::egui;

/// Derives the [`Inspectable`](Inspectable) trait.
pub use bevy_inspector_egui_derive::Inspectable;
pub use plugin::InspectorPlugin;

/// Attributes for the built-in [`Inspectable`](Inspectable) implementations
pub mod options {
    pub use crate::impls::*;
}

#[non_exhaustive]
#[derive(Default)]
pub struct Context<'a> {
    pub resources: Option<&'a bevy::ecs::Resources>,
}

impl<'a> Context<'a> {
    pub fn new(resources: &'a bevy::ecs::Resources) -> Self {
        Context {
            resources: Some(resources),
        }
    }
}

impl std::fmt::Debug for Context<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct ResourcesDebug;
        impl std::fmt::Debug for ResourcesDebug {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "<bevy::ecs::Resources>")
            }
        }

        f.debug_struct("Context")
            .field("resources", &self.resources.map(|_| ResourcesDebug))
            .finish()
    }
}

/// This trait describes how a struct should be displayed.
/// It can be derived for structs and enums, see the [crate-level docs](index.html) for how to do that.
pub trait Inspectable {
    /// The `Attributes` associated type specifies what attributes can be passed to a field.
    /// See the following snippet for an example:
    /// ```rust
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
}
