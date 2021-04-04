use super::{WorldInspectorParams, WorldUIContext};
use crate::{utils, Inspectable};
use bevy::{
    ecs::query::{Fetch, FilterFetch, WorldQuery},
    prelude::*,
};
use bevy_egui::egui::CollapsingHeader;
use std::marker::PhantomData;
use utils::error_label;

impl Inspectable for World {
    type Attributes = WorldInspectorParams;

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        options: Self::Attributes,
        context: &crate::Context,
    ) {
        let mut world_ui_ctx = WorldUIContext::new(context.ui_ctx, self);
        world_ui_ctx.world_ui::<()>(ui, &options);
    }
}

#[derive(Clone)]
/// Inspectable Attributes for `Entity`
pub struct EntityAttributes {
    /// Whether a button for despawning the entity should be shown
    pub despawnable: bool,
}
impl Default for EntityAttributes {
    fn default() -> Self {
        EntityAttributes { despawnable: true }
    }
}

impl Inspectable for Entity {
    type Attributes = EntityAttributes;

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        options: Self::Attributes,
        context: &crate::Context,
    ) {
        let world = expect_world!(ui, context, "Entity");
        let mut world_inspector_params =
            world.get_resource_or_insert_with(WorldInspectorParams::default);
        let params = std::mem::replace(&mut *world_inspector_params, WorldInspectorParams::empty());

        let world_ui_ctx = WorldUIContext::new(context.ui_ctx, world);
        ui.vertical(|ui| {
            world_ui_ctx.entity_ui_inner(ui, *self, &params, context.id(), &options);
        });
        drop(world_ui_ctx);

        *world.get_resource_mut::<WorldInspectorParams>().unwrap() = params;
    }
}

/// Executes [Queries](bevy::ecs::system::Query) and displays the result.
///
/// You can use any types and filters which are allowed in regular bevy queries,
/// however you may need to specify a `'static` lifetime since you can't elide them in structs.
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_inspector_egui::{Inspectable, InspectorPlugin};
/// use bevy_inspector_egui::widgets::InspectorQuery;
///
/// struct Collider;
///
/// #[derive(Inspectable, Default)]
/// struct Queries {
///   colliders: InspectorQuery<Entity, With<Collider>>,
///   root_entities: InspectorQuery<Entity, Without<Parent>>,
///   transforms: InspectorQuery<&'static mut Transform>,
/// }
///
/// fn main() {
///     App::build()
///         .add_plugins(DefaultPlugins)
///         .add_plugin(InspectorPlugin::<Queries>::new())
///         .run();
/// }
/// ```
pub struct InspectorQuery<Q, F = ()>(PhantomData<(Q, F)>);

impl<Q, F> Default for InspectorQuery<Q, F> {
    fn default() -> Self {
        InspectorQuery(PhantomData)
    }
}

#[derive(Clone)]
pub struct InspectorQueryAttributes {
    pub collapse: bool,
}
impl Default for InspectorQueryAttributes {
    fn default() -> Self {
        InspectorQueryAttributes { collapse: false }
    }
}

impl<'w, Q, F> Inspectable for InspectorQuery<Q, F>
where
    Q: WorldQuery,
    F: WorldQuery,
    F::Fetch: FilterFetch,
    <<Q as WorldQuery>::Fetch as Fetch<'static>>::Item: Inspectable,
{
    type Attributes =
        <<<Q as WorldQuery>::Fetch as Fetch<'static>>::Item as Inspectable>::Attributes;

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        options: Self::Attributes,
        context: &crate::Context,
    ) {
        let world = match context.world {
            // Safety: the pointer provided in `Context::new` must be exclusive and valid.
            Some(world) => unsafe { &mut *world },
            None => {
                return error_label(ui, format!("Query needs exclusive access to the world"));
            }
        };

        ui.vertical(move |ui| {
            let mut query_state = world.query_filtered::<Q, F>();

            for (i, mut value) in query_state.iter_mut(world).enumerate() {
                let name = pretty_type_name::pretty_type_name::<Q>();
                CollapsingHeader::new(name)
                    .id_source(context.id().with(i))
                    .show(ui, |ui| {
                        value.ui(ui, options.clone(), &context.with_id(i as u64));
                    });
            }
        });
    }
}
