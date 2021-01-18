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

// TODO document attributes

#![allow(missing_docs)]
mod impls;
mod plugin;

pub use bevy_egui::egui;

/// Derives the [`Inspectable`](Inspectable) trait.
pub use bevy_inspector_egui_derive::Inspectable;
pub use plugin::InspectorPlugin;

/// Option types for the built-in widget implementations
pub mod options {
    pub use crate::impls::*;
}

#[non_exhaustive]
#[derive(Default, Debug, Clone)]
/// This type is passed to the [`Inspectable::ui`](Inspectable::ui) method
/// to give access to the attributes specified in the `#[derive(Inspectable)]`.
/// For an example of defining custom attributes, see the [docs of Inspectable](Inspectable::FieldOptions).
pub struct Options<T> {
    /// The user defined options
    pub custom: T,
}
impl<T: Default> Options<T> {
    /// Create new options
    pub fn new(custom: T) -> Self {
        Options { custom }
    }
}

/// This trait describes how a struct should be displayed.
/// It can be derived for structs and enums, see the [crate-level docs](index.html) for how to do that.
pub trait Inspectable {
    /// The `FieldOptions` associated type specifies what attributes can be passed to a field.
    /// See the following snippet for an example:
    /// ```rust
    /// # use bevy_inspector_egui::{egui, Inspectable, Options};
    /// struct MyCustomType;
    /// # #[derive(Default)]
    /// struct MyWidgetOptions { a: f32, b: Option<String> }
    ///
    /// impl Inspectable for MyCustomType {
    ///   type FieldOptions = MyWidgetOptions;
    ///
    ///   fn ui(&mut self, _: &mut egui::Ui, options: Options<MyWidgetOptions>) {
    ///     println!("a = {}, b = {:?}", options.custom.a, options.custom.b);
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
    type FieldOptions: Default;

    /// This methods is responsible for building the egui ui.
    fn ui(&mut self, ui: &mut egui::Ui, options: Options<Self::FieldOptions>);
}
