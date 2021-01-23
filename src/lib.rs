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

#![allow(missing_docs)]
mod impls;
mod plugin;

/// `Inspectable` implementation for foreign types implementing `Reflect`
pub mod reflect;

pub use bevy_egui::egui;

/// Derives the [`Inspectable`](Inspectable) trait.
pub use bevy_inspector_egui_derive::Inspectable;
pub use plugin::InspectorPlugin;

/// Attributes for the built-in [`Inspectable`](Inspectable) implementations
pub mod options {
    pub use crate::impls::*;
}

/// This trait describes how a struct should be displayed.
/// It can be derived for structs and enums, see the [crate-level docs](index.html) for how to do that.
pub trait Inspectable {
    /// The `Attributes` associated type specifies what attributes can be passed to a field.
    /// See the following snippet for an example:
    /// ```rust
    /// # use bevy_inspector_egui::{egui, Inspectable};
    /// struct MyCustomType;
    /// # #[derive(Default)]
    /// struct MyWidgetAttributes { a: f32, b: Option<String> }
    ///
    /// impl Inspectable for MyCustomType {
    ///   type Attributes = MyWidgetAttributes;
    ///
    ///   fn ui(&mut self, _: &mut egui::Ui, options: MyWidgetAttributes) {
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
    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes);
}

/// Gives implementors of [`InspectableWithContext`](InspectableWithContext) access to bevy resources.
///
/// This is needed for example for the displaying `Handle<T>`'s.
pub struct Context<'a> {
    pub resources: &'a bevy::ecs::Resources,
}
impl std::fmt::Debug for Context<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context").field("resources", &"..").finish()
    }
}

/// Similar to [`Inspectable`](Inspectable), but also passes a [`Context`](Context),
/// from which bevy's [`Resources`](bevy::ecs::Resources) can be accessed and modified.
///
/// The disadvantage is, that any system using these resources must run as a thread-local system,
/// so you need to add the [`ThreadLocalInspectorPlugin`] when using it.
pub trait InspectableWithContext {
    /// See [Inspectable::Attributes](Inspectable::Attributes)
    type Attributes: Default;

    /// Same as [Inspectable::ui](Inspectable::ui) but with [`Context`](Context)
    fn ui_with_context(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context);
}

impl<T: Inspectable> InspectableWithContext for T {
    type Attributes = T::Attributes;

    fn ui_with_context(&mut self, ui: &mut egui::Ui, options: Self::Attributes, _: &Context) {
        self.ui(ui, options);
    }
}
