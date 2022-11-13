use std::marker::PhantomData;

use bevy::prelude::{Mut, Resource};

use crate::Inspectable;

/// The resource inspector can be used to edit resources in the inspector.
///
/// # Example
///
/// ```rust,no_run
/// use bevy::{prelude::*, pbr::AmbientLight};
/// use bevy_inspector_egui::{Inspectable, InspectorPlugin, widgets::ResourceInspector};
///
/// #[derive(Resource, Inspectable, Default)]
/// struct Data {
///     clear_color: ResourceInspector<ClearColor>,
///     ambient_light: ResourceInspector<AmbientLight>,
/// }
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugin(InspectorPlugin::<Data>::new())
///         .run();
/// }
/// ```
pub struct ResourceInspector<T>(PhantomData<fn() -> T>);
impl<T: Resource + Inspectable + Send + Sync + 'static> Inspectable for ResourceInspector<T> {
    type Attributes = T::Attributes;

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        options: Self::Attributes,
        context: &mut crate::Context,
    ) -> bool {
        context.resource_scope(ui, "ResourceInspector", |ui, context, mut res: Mut<T>| {
            res.ui(ui, options, context)
        })
    }
}

impl<T> std::fmt::Debug for ResourceInspector<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceInspector").finish()
    }
}
impl<T> Default for ResourceInspector<T> {
    fn default() -> Self {
        ResourceInspector(PhantomData)
    }
}
