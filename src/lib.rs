#![warn(missing_docs)]
#![allow(
    clippy::return_self_not_must_use,
    clippy::unit_arg,
    clippy::needless_doctest_main,
    clippy::too_many_arguments,
    clippy::collapsible_if,
    clippy::result_unit_err
)]

//! This crate provides the ability to annotate structs with a `#[derive(Inspectable)]`,
//! which opens a debug interface using [egui](https://github.com/emilk/egui) where you can visually edit the values of your struct live.
//!
//! Your struct will then be available to you as a bevy resource.
//!
//! ## Example
//! ```rust,no_run
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
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(InspectorPlugin::<Data>::new())
//!         .run();
//! }
//! ```
//!
//! The list of built-in attributes is documented [here](Inspectable#default-attributes).
//!
//! ## World Inspector
//!
//! If you want to display all world entities you can add the [`WorldInspectorPlugin`]:
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_inspector_egui::WorldInspectorPlugin;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(WorldInspectorPlugin::new())
//!         .add_startup_system(setup)
//!         .run();
//! }
//! # fn setup() {}
//! ```
//!
//! You can configure the `WorldInspectorPlugin` by inserting the `WorldInspectorParams` resource.
//! If you want to only display some components, you may want to use the [InspectorQuery](./examples/README.md#inspector-query-source) instead.
//!
//! ### Custom components in the world inspector
//!
//! By default, types implementing `Inspectable` will not be displayed in the `WorldInspector`, because the there is no way to know of the trait implementation at runtime.
//! You can call `app.register_inspectable::<T>()` to tell `bevy-inspector-egui` how that type should be displayed, and it will show up correctly in the world inspector.
//!
//! Alternatively, you can `#[derive(Reflect)]` and call `app.register_type::<T>()`. This will enable bevy's reflection feature for the type, and it will show up in the world inspector.
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_inspector_egui::{WorldInspectorPlugin, Inspectable, RegisterInspectable};
//!
//! #[derive(Inspectable, Component)]
//! struct InspectableType;
//!
//! #[derive(Reflect, Component, Default)]
//! #[reflect(Component)]
//! struct ReflectedType;
//!
//! fn main() {
//!   App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugin(WorldInspectorPlugin::new())
//!     .register_inspectable::<InspectableType>() // tells bevy-inspector-egui how to display the struct in the world inspector
//!     .register_type::<ReflectedType>() // registers the type in the `bevy_reflect` machinery, so that even without implementing `Inspectable` we can display the struct fields
//!     .run();
//! }
//! ```
//!
//! # Features
//! - **clipboard** (enabled by default): enables `egui`'s clipboard integratoin

#[macro_use]
mod utils;

/// Utitly types implementing [`Inspectable`](crate::Inspectable)
pub mod widgets;

#[allow(missing_docs)]
mod impls;
/// Internals for the inspector plugins
pub mod plugin;

/// Configuration for the [`WorldInspectorPlugin`](crate::world_inspector::WorldInspectorPlugin)
pub mod world_inspector;

/// Commonly used imports
pub mod prelude {
    pub use crate::{Inspectable, InspectorPlugin, RegisterInspectable, WorldInspectorPlugin};
}

use std::hash::Hasher;
use std::marker::PhantomData;

use bevy::ecs::system::Resource;
use bevy::prelude::{App, Mut, World};
use utils::error_label_needs_world;
#[doc(inline)]
pub use world_inspector::{InspectableRegistry, WorldInspectorParams, WorldInspectorPlugin};

/// [`Inspectable`] implementation for foreign types implementing [`Reflect`](bevy::reflect::Reflect)
pub mod reflect;

pub use bevy_egui;
pub use bevy_egui::egui;

/// Derives the [`Inspectable`](Inspectable) trait.
pub use bevy_inspector_egui_derive::Inspectable;
#[doc(inline)]
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
    pub ui_ctx: Option<&'a egui::Context>,
    /// The world is only available when not using `InspectablePlugin::shared()`
    world: Option<*mut World>,
    _world_marker: PhantomData<&'a mut ()>,

    /// Something to distinguish between siblings.
    pub id: Option<u64>,
}

impl<'a> Context<'a> {
    /// Returns a reference to the world if the context has access to it
    pub fn world(&self) -> Option<&'a World> {
        self.world.map(|world| unsafe { &*world })
    }

    /// Get a mutable reference to the `world` if the inspector has access to it.
    ///
    /// # Safety
    /// Users of this function are only allowed to mutate resources and components,
    /// but can't do any changes that would result in changes of archetypes.
    pub unsafe fn world_mut(&mut self) -> Option<&'a mut World> {
        match self.world {
            Some(world) => Some(&mut *world),
            None => None,
        }
    }

    /// Returns the provided closure with mutable access to the world, and a context
    /// that has *no* access to the world.
    ///
    /// Will display an error message if the context doesn't have world access.
    pub fn world_scope(
        &mut self,
        ui: &mut egui::Ui,
        ty: &str,
        f: impl FnOnce(&mut World, &mut egui::Ui, &mut Context) -> bool,
    ) -> bool {
        let world = match self.world.take() {
            Some(world) => world,
            None => return error_label_needs_world(ui, ty),
        };
        let mut cx = Context {
            world: None,
            ..*self
        };
        let world = unsafe { &mut *world };
        let changed = f(world, ui, &mut cx);
        self.world = Some(world);

        changed
    }

    /// Like [`Context::world_scope`], but the context will have world access as well.
    ///
    /// # Safety
    /// The function must not use the world in ways that would invalidate archetypes, as
    /// types like the `InspectorQuery` will rely on them being the same before and after
    /// calling `Inspectable::ui`.
    pub unsafe fn world_scope_unchecked(
        &mut self,
        ui: &mut egui::Ui,
        ty: &str,
        f: impl FnOnce(&mut World, &mut egui::Ui, &mut Context) -> bool,
    ) -> bool {
        let world = match self.world {
            Some(world) => &mut *world,
            None => return error_label_needs_world(ui, ty),
        };
        let changed = f(world, ui, self);
        self.world = Some(world);

        changed
    }

    /// Temporarily removes a resource from the world and calls the provided closure with that resource
    /// while still having `&mut Context` available.
    ///
    /// Will display an error message if the context doesn't have world access.
    pub fn resource_scope<T: Resource, F: FnOnce(&mut egui::Ui, &mut Context, Mut<T>) -> bool>(
        &mut self,
        ui: &mut egui::Ui,
        ty: &str,
        f: F,
    ) -> bool {
        // Safety: the world is only used to modify a resource and doesn't change any archetypes
        unsafe {
            self.world_scope_unchecked(ui, ty, |world, ui, context| {
                world.resource_scope(|world, res: Mut<T>| {
                    let mut context = context.with_world(world);
                    f(ui, &mut context, res)
                })
            })
        }
    }

    fn take_world(&mut self) -> Option<(Context, &'a mut World)> {
        let world = self.world.take()?;
        let world = unsafe { &mut *world };
        let context = Context {
            world: None,
            ..*self
        };
        Some((context, world))
    }
}

impl<'a> Context<'a> {
    /// Create a new context with access to the world
    pub fn new_world_access(ui_ctx: Option<&'a egui::Context>, world: &'a mut World) -> Self {
        Context {
            ui_ctx,
            world: Some(world),
            _world_marker: PhantomData,
            id: None,
        }
    }

    /// Creates a context without access to `World`
    pub fn new_shared(ui_ctx: Option<&'a egui::Context>) -> Self {
        Context {
            ui_ctx,
            world: None,
            _world_marker: PhantomData,
            id: None,
        }
    }

    /// Same context but with different `world`
    pub fn with_world(&self, world: &'a mut World) -> Self {
        Context {
            world: Some(world),
            ..*self
        }
    }

    /// Same context but with a derived `id`
    pub fn with_id(&mut self, id: u64) -> Context<'_> {
        let mut hasher = std::collections::hash_map::DefaultHasher::default();
        if let Some(id) = self.id {
            hasher.write_u64(id);
        }
        hasher.write_u64(id);
        let id = hasher.finish();

        Context {
            id: Some(id),
            world: self.world.as_mut().map(|world| *world),
            _world_marker: PhantomData,
            ui_ctx: self.ui_ctx,
        }
    }

    /// Returns the [id](Context#structfield.id) if present, otherwise a dummy id.
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
/// - **override_where_clause**: specifies which type bounds should be used for generics. For example `#[inspectable(override_where_clause = "") struct Struct(PhantomData<M>)` won't include a `M: Inspectable` bound.
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
    ///   fn ui(&mut self, _: &mut egui::Ui, options: MyWidgetAttributes, context: &mut Context) -> bool {
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
    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &mut Context) -> bool;

    /// Displays the value without any context. Useful for usage outside of the plugins, where
    /// there is no access to the world or [`EguiContext`](bevy_egui::EguiContext).
    fn ui_raw(&mut self, ui: &mut egui::Ui, options: Self::Attributes) {
        let mut empty_context = Context::new_shared(None);
        self.ui(ui, options, &mut empty_context);
    }

    /// Required setup for the bevy application, e.g. registering events. Note that this method will run for every instance of a type.
    #[allow(unused_variables)]
    fn setup(app: &mut App) {}
}

/// Helper trait for enabling `app.register_inspectable::<T>()`
pub trait RegisterInspectable {
    /// Register type `T` so that it can be displayed by the [`WorldInspectorPlugin`](crate::WorldInspectorPlugin).
    /// Forwards to [`InspectableRegistry::register`].
    fn register_inspectable<T: Inspectable + 'static>(&mut self) -> &mut Self;
}

impl RegisterInspectable for App {
    fn register_inspectable<T: Inspectable + 'static>(&mut self) -> &mut Self {
        self.world
            .get_resource_mut::<InspectableRegistry>()
            .unwrap()
            .register::<T>();
        self
    }
}
