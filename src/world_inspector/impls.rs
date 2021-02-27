use super::{WorldInspectorParams, WorldUIContext};
use crate::Inspectable;
use bevy::{
    ecs::query::{FilterFetch, WorldQuery},
    prelude::*,
};
use bevy_egui::egui::CollapsingHeader;
use pretty_type_name::pretty_type_name;
use std::marker::PhantomData;

impl Inspectable for World {
    type Attributes = WorldInspectorParams;

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        options: Self::Attributes,
        context: &crate::Context,
    ) {
        let mut world_ui_ctx = WorldUIContext::new(context.ui_ctx, self);
        world_ui_ctx.world_ui(ui, &options);
    }
}

impl Inspectable for Entity {
    type Attributes = WorldInspectorParams;

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        options: Self::Attributes,
        context: &crate::Context,
    ) {
        let world = expect_world!(ui, context, "Entity");

        let world_ui_ctx = WorldUIContext::new(context.ui_ctx, world);
        ui.vertical(|ui| {
            world_ui_ctx.entity_ui_inner(ui, *self, &options, context.id());
        });
    }
}

/// Queries for entities matching the filter `F` and displays their entity trees.
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_inspector_egui::{Inspectable, InspectorPlugin};
/// use bevy_inspector_egui::widgets::InspectorQuery;
///
/// struct Collider;
///
/// #[derive(Inspectable, Default)]
/// struct Queries {
///   colliders: InspectorQuery<With<Collider>>,
///   root_entities: InspectorQuery<Without<Parent>>,
/// }
///
/// fn main() {
///     App::build()
///         .add_plugins(DefaultPlugins)
///         .add_plugin(InspectorPlugin::<Queries>::new())
///         .run();
/// }
/// ```
pub struct InspectorQuery<F>(PhantomData<F>);

impl<F> Default for InspectorQuery<F> {
    fn default() -> Self {
        InspectorQuery(PhantomData)
    }
}

#[derive(Clone)]
pub struct InspectorQueryAttributes {
    collapse: bool,
}
impl Default for InspectorQueryAttributes {
    fn default() -> Self {
        InspectorQueryAttributes { collapse: false }
    }
}

impl<F> Inspectable for InspectorQuery<F>
where
    F: WorldQuery,
    F::Fetch: FilterFetch,
{
    type Attributes = InspectorQueryAttributes;

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        options: Self::Attributes,
        context: &crate::Context,
    ) {
        let world = expect_world!(ui, context, "InspectorQuery");

        let mut query_state = world.query_filtered::<Entity, F>();
        let entities: Vec<Entity> = query_state.iter(world).collect();

        let ui_ctx = WorldUIContext::new(context.ui_ctx, world);
        let params = WorldInspectorParams::default();

        ui.vertical(|ui| {
            if options.collapse {
                let name = pretty_type_name::<F>();
                CollapsingHeader::new(name)
                    .id_source(context.id())
                    .show(ui, |ui| {
                        for entity in entities {
                            ui_ctx.entity_ui(ui, entity, &params, context.id());
                            ui.end_row();
                        }
                    });
            } else {
                for entity in entities {
                    ui_ctx.entity_ui(ui, entity, &params, context.id());
                }
            }
        });
    }
}
