use super::{WorldInspectorParams, WorldUIContext};
use crate::Inspectable;
use bevy::{
    ecs::query::{Fetch, WorldQuery, WorldQueryGats},
    prelude::*,
};
use bevy_egui::egui::CollapsingHeader;
use std::marker::PhantomData;

impl Inspectable for World {
    type Attributes = WorldInspectorParams;

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        mut options: Self::Attributes,
        context: &mut crate::Context,
    ) -> bool {
        let mut world_ui_ctx = WorldUIContext::new(self, context.ui_ctx);
        world_ui_ctx.world_ui::<()>(ui, &mut options)
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
        context: &mut crate::Context,
    ) -> bool {
        unsafe {
            context.world_scope_unchecked(ui, "Entity", |world, ui, context| {
                let mut world_inspector_params =
                    world.get_resource_or_insert_with(WorldInspectorParams::default);
                let params =
                    std::mem::replace(&mut *world_inspector_params, WorldInspectorParams::empty());

                let mut world_ui_ctx = WorldUIContext::new(world, context.ui_ctx);
                let changed = ui
                    .vertical(|ui| {
                        world_ui_ctx.entity_ui_inner(ui, *self, &params, context.id(), &options)
                    })
                    .inner;
                drop(world_ui_ctx);

                *world.get_resource_mut::<WorldInspectorParams>().unwrap() = params;

                changed
            })
        }
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
/// #[derive(Component)]
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
///     App::new()
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

type WorldQueryItem<'w, 's, Q> = <<Q as WorldQueryGats<'w>>::Fetch as Fetch<'w>>::Item;

unsafe fn extend_lifetime<'w, 's, Q>(
    item: &<WorldQueryItem<'static, 'static, Q> as Inspectable>::Attributes,
) -> <WorldQueryItem<'w, 's, Q> as Inspectable>::Attributes
where
    Q: WorldQuery,
    for<'world, 'state> WorldQueryItem<'world, 'state, Q>: Inspectable,
{
    std::mem::transmute_copy(item)
}

impl<Q: 'static, F: 'static> Inspectable for InspectorQuery<Q, F>
where
    Q: WorldQuery,
    F: WorldQuery,
    for<'w, 's> WorldQueryItem<'w, 's, Q>: Inspectable,
{
    type Attributes = <WorldQueryItem<'static, 'static, Q> as Inspectable>::Attributes;

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        options: Self::Attributes,
        context: &mut crate::Context,
    ) -> bool {
        unsafe {
            context.world_scope_unchecked(ui, "InspectorQuery", |world, ui, context| {
                let mut changed = false;

                ui.vertical(move |ui| {
                    let mut query_state = world.query_filtered::<Q, F>();

                    for (i, mut value) in query_state.iter_mut(world).enumerate() {
                        let name = pretty_type_name::pretty_type_name::<Q>();
                        CollapsingHeader::new(name)
                            .id_source(context.id().with(i))
                            .show(ui, |ui| {
                                let options = extend_lifetime::<Q>(&options);
                                changed |= value.ui(ui, options, &mut context.with_id(i as u64));
                            });
                    }
                });

                changed
            })
        }
    }
}

/// Executes [Queries](bevy::ecs::system::Query) and displays the only result.
///
/// You can use any types and filters which are allowed in regular bevy queries,
/// however you may need to specify a `'static` lifetime since you can't elide them in structs.
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_inspector_egui::{Inspectable, InspectorPlugin};
/// use bevy_inspector_egui::widgets::InspectorQuerySingle;
///
/// #[derive(Component)]
/// struct Player;
///
/// #[derive(Inspectable, Default)]
/// struct Queries {
///   player: InspectorQuerySingle<Entity, With<Player>>
/// }
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugin(InspectorPlugin::<Queries>::new())
///         .run();
/// }
/// ```
pub struct InspectorQuerySingle<Q, F = ()>(PhantomData<(Q, F)>);

impl<Q, F> Default for InspectorQuerySingle<Q, F> {
    fn default() -> Self {
        InspectorQuerySingle(PhantomData)
    }
}

impl<Q, F> Inspectable for InspectorQuerySingle<Q, F>
where
    Q: WorldQuery + 'static,
    F: WorldQuery + 'static,
    for<'w, 's> WorldQueryItem<'w, 's, Q>: Inspectable,
{
    type Attributes = <WorldQueryItem<'static, 'static, Q> as Inspectable>::Attributes;

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        options: Self::Attributes,
        context: &mut crate::Context,
    ) -> bool {
        unsafe {
            context.world_scope_unchecked(ui, "InspectorQuerySingle", |world, ui, context| {
                let mut changed = false;
                ui.vertical(move |ui| {
                    let mut query_state = world.query_filtered::<Q, F>();
                    let mut iter = query_state.iter_mut(world);
                    let value = iter.next();
                    let has_more = iter.next().is_some();

                    match (value, has_more) {
                        (None, _) => {
                            ui.label("No entity matches the query.");
                        }
                        (Some(_), true) => {
                            ui.label("More than one entity matches the query.");
                        }
                        (Some(mut value), false) => {
                            let name = pretty_type_name::pretty_type_name::<Q>();
                            CollapsingHeader::new(name)
                                .id_source(context.id())
                                .show(ui, |ui| {
                                    let options = extend_lifetime::<Q>(&options);
                                    changed |= value.ui(ui, options, context);
                                });
                        }
                    };
                });

                changed
            })
        }
    }
}
