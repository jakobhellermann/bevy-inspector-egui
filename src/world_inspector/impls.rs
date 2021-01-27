use super::{short_name, WorldInspectorParams, WorldUIContext};
use crate::{utils, Inspectable};
use bevy::{ecs::QueryFilter, prelude::*};
use std::{any::type_name, marker::PhantomData};

impl Inspectable for World {
    type Attributes = WorldInspectorParams;

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        options: Self::Attributes,
        context: &crate::Context,
    ) {
        let resources = expect_context!(ui, context.resources, "World");

        let ui_ctx = WorldUIContext::new(self, resources);
        ui_ctx.ui(ui, &options);
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
        let resources = expect_context!(ui, context.resources, "Entity");
        let world = expect_context!(ui, context.world, "Entity");

        let ui_ctx = WorldUIContext::new(world, resources);
        ui_ctx.entity_ui(ui, *self, &options);
    }
}

/// Queries for entities matching the filter `F` and displays their entity trees.
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_inspector_egui::{Inspectable, world_inspector::InspectorQuery, InspectorPlugin};
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

pub struct InspectorQueryAttributes {
    collapse: bool,
}
impl Default for InspectorQueryAttributes {
    fn default() -> Self {
        InspectorQueryAttributes { collapse: false }
    }
}

impl<F: QueryFilter> Inspectable for InspectorQuery<F> {
    type Attributes = InspectorQueryAttributes;

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        options: Self::Attributes,
        context: &crate::Context,
    ) {
        let resources = expect_context!(ui, context.resources, "InspectorQuery");
        let world = expect_context!(ui, context.world, "InspectorQuery");

        let ui_ctx = WorldUIContext::new(world, resources);

        let entities: Vec<Entity> = world.query_filtered::<Entity, F>().collect();

        let params = WorldInspectorParams::default();
        ui.vertical_centered(|ui| {
            if options.collapse {
                let name = short_name(type_name::<F>());
                ui.collapsing(name, |ui| {
                    for entity in entities {
                        ui_ctx.entity_ui(ui, entity, &params);
                        ui.end_row();
                    }
                });
            } else {
                for entity in entities {
                    ui_ctx.entity_ui(ui, entity, &params);
                    ui.end_row();
                }
            }
        });
    }
}
