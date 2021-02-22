use super::{WorldInspectorParams, WorldUIContext};
use crate::Inspectable;
use bevy::{ecs::QueryFilter, prelude::*};
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
        let resources = expect_context!(ui, context.resources, "World");

        let ui_ctx = WorldUIContext::new(context.ui_ctx, self, resources);
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

        let ui_ctx = WorldUIContext::new(context.ui_ctx, world, resources);
        ui.vertical(|ui| {
            ui_ctx.entity_ui_inner(ui, *self, &options, context.id());
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

        let ui_ctx = WorldUIContext::new(context.ui_ctx, world, resources);

        let entities: Vec<Entity> = world.query_filtered::<Entity, F>().collect();

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
