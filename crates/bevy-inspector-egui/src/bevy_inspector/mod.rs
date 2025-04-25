//! Methods for displaying `bevy` resources, assets and entities
//!
//! # Example
//!
//! ```rust
//! use bevy_inspector_egui::bevy_inspector;
//! # use bevy_ecs::prelude::*;
//! # use bevy_state::prelude::States;
//! # use bevy_reflect::Reflect;
//! # use bevy_time::Time;
//! # use bevy_math::Vec3;
//!
//! #[derive(States, Debug, Clone, Eq, PartialEq, Hash, Reflect, Default)]
//! enum AppState { #[default] A, B, C }
//!
//! fn show_ui(world: &mut World, ui: &mut egui::Ui) {
//!     let mut any_reflect_value = Vec3::new(1.0, 2.0, 3.0);
//!     bevy_inspector::ui_for_value(&mut any_reflect_value, ui, world);
//!
//!     ui.heading("Time resource");
//!     bevy_inspector::ui_for_resource::<Time>(world, ui);
//!
//!     ui.heading("App State");
//!     bevy_inspector::ui_for_state::<AppState>(world, ui);
//!
//!     egui::CollapsingHeader::new("Entities")
//!         .default_open(true)
//!         .show(ui, |ui| {
//!             bevy_inspector::ui_for_entities(world, ui);
//!         });
//!     egui::CollapsingHeader::new("Resources").show(ui, |ui| {
//!         bevy_inspector::ui_for_resources(world, ui);
//!     });
//!     egui::CollapsingHeader::new("Assets").show(ui, |ui| {
//!         bevy_inspector::ui_for_all_assets(world, ui);
//!     });
//! }
//! ```

use std::any::TypeId;
use std::marker::PhantomData;

use crate::utils::{pretty_type_name, pretty_type_name_str};
use bevy_asset::{Asset, AssetServer, Assets, ReflectAsset, UntypedAssetId};
use bevy_ecs::query::{QueryFilter, WorldQuery};
use bevy_ecs::world::CommandQueue;
use bevy_ecs::{component::ComponentId, prelude::*};
use bevy_reflect::{Reflect, TypeRegistry};
use bevy_state::state::{FreelyMutableState, NextState, State};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

pub(crate) mod errors;

/// UI for displaying the entity hierarchy
pub mod hierarchy;

use crate::reflect_inspector::{Context, InspectorUi};
use crate::restricted_world_view::RestrictedWorldView;

/// Display a single [`&mut dyn Reflect`](bevy_reflect::Reflect).
///
/// If you are wondering why this function takes in a [`&mut World`](bevy_ecs::world::World), it's so that if the value contains e.g. a
/// `Handle<StandardMaterial>` it can look up the corresponding asset resource and display the asset value inline.
///
/// If all you're displaying is a simple value without any references into the bevy world, consider just using
/// [`reflect_inspector::ui_for_value`](crate::reflect_inspector::ui_for_value).
pub fn ui_for_value(value: &mut dyn Reflect, ui: &mut egui::Ui, world: &mut World) -> bool {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    let mut queue = CommandQueue::default();
    let mut cx = Context {
        world: Some(RestrictedWorldView::new(world)),
        queue: Some(&mut queue),
    };
    let mut env = InspectorUi::for_bevy(&type_registry, &mut cx);
    let changed = env.ui_for_reflect(value.as_partial_reflect_mut(), ui);
    queue.apply(world);
    changed
}

/// Display `Entities`, `Resources` and `Assets` using their respective functions inside headers
pub fn ui_for_world(world: &mut World, ui: &mut egui::Ui) {
    egui::CollapsingHeader::new("Entities")
        .default_open(true)
        .show(ui, |ui| {
            ui_for_entities(world, ui);
        });
    egui::CollapsingHeader::new("Resources").show(ui, |ui| {
        ui_for_resources(world, ui);
    });
    egui::CollapsingHeader::new("Assets").show(ui, |ui| {
        ui_for_all_assets(world, ui);
    });
}

/// Display all reflectable resources in the world
pub fn ui_for_resources(world: &mut World, ui: &mut egui::Ui) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    let mut resources: Vec<_> = type_registry
        .iter()
        .filter(|registration| registration.data::<ReflectResource>().is_some())
        .map(|registration| {
            (
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
            )
        })
        .collect();
    resources.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));
    for (name, type_id) in resources {
        ui.collapsing(name, |ui| {
            by_type_id::ui_for_resource(world, type_id, ui, name, &type_registry);
        });
    }
}

/// Display the resource `R`
pub fn ui_for_resource<R: Resource + Reflect>(world: &mut World, ui: &mut egui::Ui) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    // create a context with access to the world except for the `R` resource
    let Some((mut resource, world_view)) =
        RestrictedWorldView::new(world).split_off_resource_typed::<R>()
    else {
        errors::resource_does_not_exist(ui, &pretty_type_name::<R>());
        return;
    };
    let mut queue = CommandQueue::default();
    let mut cx = Context {
        world: Some(world_view),
        queue: Some(&mut queue),
    };
    let mut env = InspectorUi::for_bevy(&type_registry, &mut cx);

    if env.ui_for_reflect(resource.bypass_change_detection(), ui) {
        resource.set_changed();
    }

    queue.apply(world);
}

/// Display all reflectable assets
pub fn ui_for_all_assets(world: &mut World, ui: &mut egui::Ui) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    let mut assets: Vec<_> = type_registry
        .iter()
        .filter(|registration| registration.data::<ReflectAsset>().is_some())
        .map(|registration| {
            (
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
            )
        })
        .collect();
    assets.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));
    for (name, type_id) in assets {
        ui.collapsing(name, |ui| {
            by_type_id::ui_for_assets(world, type_id, ui, &type_registry);
        });
    }
}

/// Display all assets of the specified asset type `A`
pub fn ui_for_assets<A: Asset + Reflect>(world: &mut World, ui: &mut egui::Ui) {
    let asset_server = world.get_resource::<AssetServer>().cloned();

    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    // create a context with access to the world except for the `R` resource
    let Some((mut assets, world_view)) =
        RestrictedWorldView::new(world).split_off_resource_typed::<Assets<A>>()
    else {
        errors::resource_does_not_exist(ui, &pretty_type_name::<Assets<A>>());
        return;
    };

    let mut queue = CommandQueue::default();
    let mut cx = Context {
        world: Some(world_view),
        queue: Some(&mut queue),
    };

    let mut assets: Vec<_> = assets.iter_mut().collect();
    assets.sort_by(|(a, _), (b, _)| a.cmp(b));
    for (handle_id, asset) in assets {
        let id = egui::Id::new(handle_id);

        egui::CollapsingHeader::new(handle_name(handle_id.untyped(), asset_server.as_ref()))
            .id_salt(id)
            .show(ui, |ui| {
                let mut env = InspectorUi::for_bevy(&type_registry, &mut cx);
                env.ui_for_reflect_with_options(asset, ui, id, &());
            });
    }

    queue.apply(world);
}

/// Display state `T` and change state on edit
pub fn ui_for_state<T: FreelyMutableState + Reflect>(world: &mut World, ui: &mut egui::Ui) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    // create a context with access to the world except for the `State<T>` resource
    let Some((state, world_view)) =
        RestrictedWorldView::new(world).split_off_resource_typed::<State<T>>()
    else {
        errors::state_does_not_exist(ui, &pretty_type_name::<T>());
        return;
    };
    let Some((mut next_state, world_view)) = world_view.split_off_resource_typed::<NextState<T>>()
    else {
        errors::state_does_not_exist(ui, &pretty_type_name::<T>());
        return;
    };
    let mut queue = CommandQueue::default();
    let mut cx = Context {
        world: Some(world_view),
        queue: Some(&mut queue),
    };
    let mut env = InspectorUi::for_bevy(&type_registry, &mut cx);

    let mut current = state.get().clone();
    let changed = env.ui_for_reflect(&mut current, ui);

    if changed {
        *next_state = NextState::Pending(current);
    }
    queue.apply(world);
}

/// Display all entities matching [`Without<Parent>`] and their components
///
/// Includes basic [`EntityFilter`]
#[deprecated(since = "0.28.1", note = "use ui_for_entities instead")]
pub fn ui_for_world_entities(world: &mut World, ui: &mut egui::Ui) {
    ui_for_entities(world, ui);
}
/// Display all entities matching the static [`QueryFilter`]
#[deprecated(since = "0.28.1", note = "use ui_for_entities_filtered instead")]
pub fn ui_for_world_entities_filtered<QF: WorldQuery + QueryFilter>(
    world: &mut World,
    ui: &mut egui::Ui,
    with_children: bool,
) {
    ui_for_entities_filtered(world, ui, with_children, &Filter::<QF>::all());
}

/// Display all root entities.
pub fn ui_for_entities(world: &mut World, ui: &mut egui::Ui) {
    let filter: Filter = Filter::from_ui_fuzzy(ui, egui::Id::new("default_world_entities_filter"));
    ui_for_entities_filtered(world, ui, true, &filter);
}

/// Display all entities matching the given [`EntityFilter`].
///
/// You can use the [`Filter`] type to specify both a static filter as a generic parameter (default is `Without<Parent>`),
/// and a word to match. [`Filter::from_ui`] will display a search box and fuzzy filter checkbox.
pub fn ui_for_entities_filtered<F>(
    world: &mut World,
    ui: &mut egui::Ui,
    with_children: bool,
    filter: &F,
) where
    F: EntityFilter,
{
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    let mut root_entities = world.query_filtered::<Entity, F::StaticFilter>();
    let mut entities = root_entities.iter(world).collect::<Vec<_>>();

    filter.filter_entities(world, &mut entities);

    entities.sort();

    let id = egui::Id::new("world ui");
    for entity in entities {
        let id = id.with(entity);

        let entity_name = guess_entity_name(world, entity);

        egui::CollapsingHeader::new(&entity_name)
            .id_salt(id)
            .show(ui, |ui| {
                if with_children {
                    ui_for_entity_with_children_inner(
                        world,
                        entity,
                        ui,
                        id,
                        &type_registry,
                        filter,
                    );
                } else {
                    let mut queue = CommandQueue::default();
                    ui_for_entity_components(
                        &mut world.into(),
                        Some(&mut queue),
                        entity,
                        ui,
                        id,
                        &type_registry,
                    );
                    queue.apply(world);
                }
            });
    }
}

pub trait EntityFilter {
    type StaticFilter: QueryFilter;

    /// Returns true if the filter term is currently active
    ///
    /// Used in the default impl of [`EntityFilter::filter_entities`] to skip filtering if false
    ///
    /// default impl is true
    fn is_active(&self) -> bool {
        true
    }

    /// Filters entities in place
    ///
    /// default impl:
    /// - uses [`EntityFilter::filter_entity`] to mark what entities to retain
    /// - skips filtering if [`EntityFilter::is_active`] returns false
    fn filter_entities(&self, world: &mut World, entities: &mut Vec<Entity>) {
        if !self.is_active() {
            return;
        }
        entities.retain(|&entity| self.filter_entity(world, entity));
    }

    /// Returns true if entity matches the filter term
    fn filter_entity(&self, world: &mut World, entity: Entity) -> bool;
}

#[derive(Debug)]
pub struct Filter<F: QueryFilter = Without<ChildOf>> {
    pub word: String,
    pub is_fuzzy: bool,
    pub marker: PhantomData<F>,
}

impl<F: QueryFilter + Clone> Clone for Filter<F> {
    fn clone(&self) -> Self {
        Self {
            word: self.word.clone(),
            is_fuzzy: self.is_fuzzy,
            marker: PhantomData,
        }
    }
}

impl<F: QueryFilter> Filter<F> {
    pub fn from_ui_fuzzy(ui: &mut egui::Ui, id: egui::Id) -> Self {
        let word = {
            let id = id.with("word");
            // filter, using eguis memory and provided id
            let mut filter_string = ui.memory_mut(|mem| {
                let filter: &mut String = mem.data.get_persisted_mut_or_default(id);
                filter.clone()
            });
            ui.text_edit_singleline(&mut filter_string);
            ui.memory_mut(|mem| {
                *mem.data.get_persisted_mut_or_default(id) = filter_string.clone();
            });

            // improves overall matching
            filter_string.to_lowercase()
        };

        Filter {
            word,
            is_fuzzy: true,
            marker: PhantomData,
        }
    }

    pub fn from_ui(ui: &mut egui::Ui, id: egui::Id) -> Self {
        ui.horizontal(|ui| {
            // filter kind
            let is_fuzzy = {
                let id = id.with("is_fuzzy");
                let mut is_fuzzy = ui.memory_mut(|mem| {
                    let fuzzy: &mut bool = mem.data.get_persisted_mut_or_default(id);
                    *fuzzy
                });
                ui.checkbox(&mut is_fuzzy, "Fuzzy");
                ui.memory_mut(|mem| {
                    *mem.data.get_persisted_mut_or_default(id) = is_fuzzy;
                });
                is_fuzzy
            };
            let word = {
                let id = id.with("word");
                // filter, using eguis memory and provided id
                let mut filter_string = ui.memory_mut(|mem| {
                    let filter: &mut String = mem.data.get_persisted_mut_or_default(id);
                    filter.clone()
                });
                ui.text_edit_singleline(&mut filter_string);
                ui.memory_mut(|mem| {
                    *mem.data.get_persisted_mut_or_default(id) = filter_string.clone();
                });

                // improves overall matching
                filter_string.to_lowercase()
            };

            Filter {
                word,
                is_fuzzy,
                marker: PhantomData,
            }
        })
        .inner
    }

    /// empty filter which does nothing
    pub fn all() -> Self {
        Self {
            word: String::from(""),
            is_fuzzy: false,
            marker: PhantomData,
        }
    }
}

impl<F: QueryFilter> EntityFilter for Filter<F> {
    type StaticFilter = F;

    fn is_active(&self) -> bool {
        !self.word.is_empty()
    }

    fn filter_entity(&self, world: &mut World, entity: Entity) -> bool {
        self_or_children_satisfy_filter(world, entity, self.word.as_str(), self.is_fuzzy)
    }
}

fn self_or_children_satisfy_filter(
    world: &mut World,
    entity: Entity,
    filter: &str,
    is_fuzzy: bool,
) -> bool {
    let name = guess_entity_name(world, entity);
    let self_matches = if is_fuzzy {
        let matcher = SkimMatcherV2::default();
        matcher.fuzzy_match(name.as_str(), filter).is_some()
    } else {
        name.to_lowercase().contains(filter)
    };
    self_matches || {
        let Ok(children) = world
            .query::<&Children>()
            .get(world, entity)
            .map(|children| children.to_vec())
        else {
            return false;
        };

        children
            .iter()
            .any(|child| self_or_children_satisfy_filter(world, *child, filter, is_fuzzy))
    }
}

/// Display the given entity with all its components and children
pub fn ui_for_entity_with_children(world: &mut World, entity: Entity, ui: &mut egui::Ui) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    let entity_name = guess_entity_name(world, entity);
    ui.label(entity_name);

    let filter: Filter = Filter::all();
    ui_for_entity_with_children_inner(
        world,
        entity,
        ui,
        egui::Id::new(entity),
        &type_registry,
        &filter,
    )
}

fn ui_for_entity_with_children_inner<F>(
    world: &mut World,
    entity: Entity,
    ui: &mut egui::Ui,
    id: egui::Id,
    type_registry: &TypeRegistry,
    filter: &F,
) where
    F: EntityFilter,
{
    let mut queue = CommandQueue::default();
    ui_for_entity_components(
        &mut world.into(),
        Some(&mut queue),
        entity,
        ui,
        id,
        type_registry,
    );

    let children = world
        .get::<Children>(entity)
        .map(|children| children.iter().collect::<Vec<_>>());
    if let Some(mut children) = children {
        if !children.is_empty() {
            filter.filter_entities(world, &mut children);
            ui.label("Children");
            for child in children {
                let id = id.with(child);

                let child_entity_name = guess_entity_name(world, child);
                egui::CollapsingHeader::new(&child_entity_name)
                    .id_salt(id)
                    .show(ui, |ui| {
                        ui.label(&child_entity_name);

                        ui_for_entity_with_children_inner(
                            world,
                            child,
                            ui,
                            id,
                            type_registry,
                            filter,
                        );
                    });
            }
        }
    }

    queue.apply(world);
}

/// Display the components of the given entity
pub fn ui_for_entity(world: &mut World, entity: Entity, ui: &mut egui::Ui) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    let entity_name = guess_entity_name(world, entity);
    ui.label(entity_name);

    let mut queue = CommandQueue::default();
    ui_for_entity_components(
        &mut world.into(),
        Some(&mut queue),
        entity,
        ui,
        egui::Id::new(entity),
        &type_registry,
    );
    queue.apply(world);
}

/// Display the components of the given entity
pub(crate) fn ui_for_entity_components(
    world: &mut RestrictedWorldView<'_>,
    mut queue: Option<&mut CommandQueue>,
    entity: Entity,
    ui: &mut egui::Ui,
    id: egui::Id,
    type_registry: &TypeRegistry,
) {
    let Ok(components) = components_of_entity(world, entity) else {
        errors::entity_does_not_exist(ui, entity);
        return;
    };

    for (name, component_id, component_type_id, size) in components {
        let id = id.with(component_id);

        let header = egui::CollapsingHeader::new(&name).id_salt(id);

        let Some(component_type_id) = component_type_id else {
            header.show(ui, |ui| errors::no_type_id(ui, &name));
            continue;
        };

        #[cfg(feature = "documentation")]
        let type_docs = type_registry
            .get_type_info(component_type_id)
            .and_then(|info| info.docs());

        if size == 0 {
            ui.indent(id, |ui| {
                let _response = ui.label(&name);
                #[cfg(feature = "documentation")]
                crate::egui_utils::show_docs(_response, type_docs);
            });
            continue;
        }

        // create a context with access to the world except for the currently viewed component
        let (mut component_view, world) = world.split_off_component((entity, component_type_id));
        let mut cx = Context {
            world: Some(world),
            #[allow(clippy::needless_option_as_deref)]
            queue: queue.as_deref_mut(),
        };

        let mut value = match component_view.get_entity_component_reflect(
            entity,
            component_type_id,
            type_registry,
        ) {
            Ok(value) => value,
            Err(e) => {
                ui.indent(id, |ui| {
                    let response = ui.label(egui::RichText::new(&name).underline());
                    response.on_hover_ui(|ui| errors::show_error(e, ui, &name));
                });
                continue;
            }
        };

        if value.is_changed() {
            #[cfg(feature = "highlight_changes")]
            set_highlight_style(ui);
        }

        let _response = header.show(ui, |ui| {
            ui.reset_style();

            let inspector_changed = InspectorUi::for_bevy(type_registry, &mut cx)
                .ui_for_reflect_with_options(
                    value.bypass_change_detection().as_partial_reflect_mut(),
                    ui,
                    id.with(component_id),
                    &(),
                );

            if inspector_changed {
                value.set_changed();
            }
        });
        #[cfg(feature = "documentation")]
        crate::egui_utils::show_docs(_response.header_response, type_docs);
        ui.reset_style();
    }
}

#[cfg(feature = "highlight_changes")]
fn set_highlight_style(ui: &mut egui::Ui) {
    let highlight_color = egui::Color32::GOLD;

    let visuals = &mut ui.style_mut().visuals;
    visuals.collapsing_header_frame = true;
    visuals.widgets.inactive.bg_stroke = egui::Stroke {
        width: 1.0,
        color: highlight_color,
    };
    visuals.widgets.active.bg_stroke = egui::Stroke {
        width: 1.0,
        color: highlight_color,
    };
    visuals.widgets.hovered.bg_stroke = egui::Stroke {
        width: 1.0,
        color: highlight_color,
    };
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke {
        width: 1.0,
        color: highlight_color,
    };
}

fn components_of_entity(
    world: &mut RestrictedWorldView<'_>,
    entity: Entity,
) -> Result<Vec<(String, ComponentId, Option<TypeId>, usize)>> {
    let entity_ref = world.world().get_entity(entity)?;

    let archetype = entity_ref.archetype();
    let mut components: Vec<_> = archetype
        .components()
        .map(|component_id| {
            let info = world.world().components().get_info(component_id).unwrap();
            let name = pretty_type_name_str(info.name());

            (name, component_id, info.type_id(), info.layout().size())
        })
        .collect();
    components.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));
    Ok(components)
}

/// Display the given entity with all its components and children
pub fn ui_for_entities_shared_components(
    world: &mut World,
    entities: &[Entity],
    ui: &mut egui::Ui,
) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    let Some(&first) = entities.first() else {
        return;
    };

    let Ok(mut components) = components_of_entity(&mut world.into(), first) else {
        return errors::entity_does_not_exist(ui, first);
    };

    for &entity in entities.iter().skip(1) {
        components.retain(|(_, id, _, _)| {
            world
                .get_entity(entity)
                .map_or(true, |entity| entity.contains_id(*id))
        })
    }

    let (resources_view, components_view) = RestrictedWorldView::resources_components(world);
    let mut queue = CommandQueue::default();
    let mut cx = Context {
        world: Some(resources_view),
        queue: Some(&mut queue),
    };
    let mut env = InspectorUi::for_bevy(&type_registry, &mut cx);

    let id = egui::Id::NULL;
    for (name, component_id, component_type_id, size) in components {
        let id = id.with(component_id);
        egui::CollapsingHeader::new(&name)
            .id_salt(id)
            .show(ui, |ui| {
                if size == 0 {
                    return;
                }
                let Some(component_type_id) = component_type_id else {
                    return errors::no_type_id(ui, &name);
                };

                let mut values = Vec::with_capacity(entities.len());

                for (i, &entity) in entities.iter().enumerate() {
                    // skip duplicate entities
                    if entities[0..i].contains(&entity) {
                        continue;
                    };

                    // SAFETY: entities are distinct, env has a context with just resources
                    match unsafe {
                        components_view.get_entity_component_reflect_unchecked(
                            entity,
                            component_type_id,
                            &type_registry,
                        )
                    } {
                        Ok(value) => {
                            values.push(value);
                        }
                        Err(error) => {
                            errors::show_error(error, ui, &name);
                            return;
                        }
                    }
                }

                let mut values_reflect: Vec<_> = values
                    .iter_mut()
                    .map(|value| value.bypass_change_detection().as_partial_reflect_mut())
                    .collect();
                let changed = env.ui_for_reflect_many_with_options(
                    component_type_id,
                    &name,
                    ui,
                    id.with(component_id),
                    &(),
                    values_reflect.as_mut_slice(),
                    &|a| a,
                );
                if changed {
                    for value in values.iter_mut() {
                        value.set_changed();
                    }
                }
            });
    }

    queue.apply(world);
}

pub mod by_type_id {
    use std::any::TypeId;

    use bevy_asset::{AssetServer, ReflectAsset, ReflectHandle, UntypedAssetId, UntypedHandle};
    use bevy_ecs::{prelude::*, world::CommandQueue};
    use bevy_reflect::TypeRegistry;

    use crate::{
        reflect_inspector::{Context, InspectorUi},
        restricted_world_view::RestrictedWorldView,
    };

    use super::{
        errors::{self, name_of_type},
        handle_name,
    };

    /// Display the resource with the given [`TypeId`]
    pub fn ui_for_resource(
        world: &mut World,
        resource_type_id: TypeId,
        ui: &mut egui::Ui,
        name_of_type: &str,
        type_registry: &TypeRegistry,
    ) {
        let mut queue = CommandQueue::default();

        {
            // create a context with access to the world except for the current resource
            let mut world_view = RestrictedWorldView::new(world);
            let (mut resource_view, world_view) = world_view.split_off_resource(resource_type_id);
            let mut cx = Context {
                world: Some(world_view),
                queue: Some(&mut queue),
            };
            let mut env = InspectorUi::for_bevy(type_registry, &mut cx);

            let mut resource = match resource_view
                .get_resource_reflect_mut_by_id(resource_type_id, type_registry)
            {
                Ok(resource) => resource,
                Err(err) => return errors::show_error(err, ui, name_of_type),
            };

            let changed = env.ui_for_reflect(
                resource.bypass_change_detection().as_partial_reflect_mut(),
                ui,
            );
            if changed {
                resource.set_changed();
            }
        }

        queue.apply(world);
    }

    /// Display all assets of the given asset [`TypeId`]
    pub fn ui_for_assets(
        world: &mut World,
        asset_type_id: TypeId,
        ui: &mut egui::Ui,
        type_registry: &TypeRegistry,
    ) {
        let asset_server = world.get_resource::<AssetServer>().cloned();

        let Some(registration) = type_registry.get(asset_type_id) else {
            return crate::reflect_inspector::errors::not_in_type_registry(
                ui,
                &name_of_type(asset_type_id, type_registry),
            );
        };
        let Some(reflect_asset) = registration.data::<ReflectAsset>() else {
            return errors::no_type_data(
                ui,
                &name_of_type(asset_type_id, type_registry),
                "ReflectAsset",
            );
        };
        let Some(reflect_handle) =
            type_registry.get_type_data::<ReflectHandle>(reflect_asset.handle_type_id())
        else {
            return errors::no_type_data(
                ui,
                &name_of_type(reflect_asset.handle_type_id(), type_registry),
                "ReflectHandle",
            );
        };

        let ids: Vec<_> = reflect_asset.ids(world).collect();

        // Create a context with access to the entire world. Displaying the `Handle<T>` will short circuit into
        // displaying the T with a world view excluding Assets<T>.
        let world_view = RestrictedWorldView::new(world);
        let mut queue = CommandQueue::default();
        let mut cx = Context {
            world: Some(world_view),
            queue: Some(&mut queue),
        };

        for handle_id in ids {
            let id = egui::Id::new(handle_id);
            let mut handle = reflect_handle
                .typed(UntypedHandle::Weak(handle_id))
                .into_partial_reflect();

            egui::CollapsingHeader::new(handle_name(handle_id, asset_server.as_ref()))
                .id_salt(id)
                .show(ui, |ui| {
                    let mut env = InspectorUi::for_bevy(type_registry, &mut cx);
                    env.ui_for_reflect_with_options(&mut *handle, ui, id, &());
                });
        }

        queue.apply(world)
    }

    /// Display a given asset by handle and asset [`TypeId`]
    pub fn ui_for_asset(
        world: &mut World,
        asset_type_id: TypeId,
        handle: UntypedAssetId,
        ui: &mut egui::Ui,
        type_registry: &TypeRegistry,
    ) -> bool {
        let Some(registration) = type_registry.get(asset_type_id) else {
            crate::reflect_inspector::errors::not_in_type_registry(
                ui,
                &name_of_type(asset_type_id, type_registry),
            );
            return false;
        };
        let Some(reflect_asset) = registration.data::<ReflectAsset>() else {
            errors::no_type_data(
                ui,
                &name_of_type(asset_type_id, type_registry),
                "ReflectAsset",
            );
            return false;
        };
        let Some(reflect_handle) =
            type_registry.get_type_data::<ReflectHandle>(reflect_asset.handle_type_id())
        else {
            errors::no_type_data(
                ui,
                &name_of_type(reflect_asset.handle_type_id(), type_registry),
                "ReflectHandle",
            );
            return false;
        };

        let _: Vec<_> = reflect_asset.ids(world).collect();

        // Create a context with access to the entire world. Displaying the `Handle<T>` will short circuit into
        // displaying the T with a world view excluding Assets<T>.
        let world_view = RestrictedWorldView::new(world);
        let mut queue = CommandQueue::default();
        let mut cx = Context {
            world: Some(world_view),
            queue: Some(&mut queue),
        };

        let id = egui::Id::new(handle);
        let mut handle = reflect_handle
            .typed(UntypedHandle::Weak(handle))
            .into_partial_reflect();

        let mut env = InspectorUi::for_bevy(type_registry, &mut cx);
        let changed = env.ui_for_reflect_with_options(&mut *handle, ui, id, &());

        queue.apply(world);

        changed
    }
}

fn handle_name(handle: UntypedAssetId, asset_server: Option<&AssetServer>) -> String {
    if let Some(path) = asset_server
        .as_ref()
        .and_then(|server| server.get_path(handle))
    {
        return path.to_string();
    }

    match handle {
        UntypedAssetId::Index { index, .. } => {
            format!("{:?}", egui::Id::new(index))
        }
        UntypedAssetId::Uuid { uuid, .. } => {
            format!("{uuid}")
        }
    }
}

impl<'a, 'c> InspectorUi<'a, 'c> {
    /// [`InspectorUi`] with short circuiting methods able to display `bevy_asset` [`Handle`](bevy_asset::Handle)s
    pub fn for_bevy(
        type_registry: &'a TypeRegistry,
        context: &'a mut Context<'c>,
    ) -> InspectorUi<'a, 'c> {
        InspectorUi::new(
            type_registry,
            context,
            Some(short_circuit::short_circuit),
            Some(short_circuit::short_circuit_readonly),
            Some(short_circuit::short_circuit_many),
        )
    }
}

/// Short circuiting methods for the [`InspectorUi`] to enable it to display [`Handle`](bevy_asset::Handle)s
pub mod short_circuit {
    use std::any::{Any, TypeId};

    use bevy_asset::ReflectAsset;
    use bevy_reflect::PartialReflect;

    use crate::reflect_inspector::{Context, InspectorUi, ProjectorReflect};

    use super::errors::{self, name_of_type};

    pub fn short_circuit(
        env: &mut InspectorUi,
        value: &mut dyn PartialReflect,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) -> Option<bool> {
        let value = value.try_as_reflect()?;

        if let Some(reflect_handle) = env
            .type_registry
            .get_type_data::<bevy_asset::ReflectHandle>(value.type_id())
        {
            let handle = reflect_handle
                .downcast_handle_untyped(value.as_any())
                .unwrap();
            let handle_id = handle.id();
            let Some(reflect_asset) = env
                .type_registry
                .get_type_data::<ReflectAsset>(reflect_handle.asset_type_id())
            else {
                errors::no_type_data(
                    ui,
                    &name_of_type(reflect_handle.asset_type_id(), env.type_registry),
                    "ReflectAsset",
                );
                return Some(false);
            };

            let Context {
                world: Some(world),
                queue,
            } = &mut env.context
            else {
                errors::no_world_in_context(ui, value.reflect_short_type_path());
                return Some(false);
            };

            let (assets_view, world) =
                world.split_off_resource(reflect_asset.assets_resource_type_id());

            let asset_value = {
                assert!(
                    assets_view.allows_access_to_resource(reflect_asset.assets_resource_type_id())
                );
                let asset_value =
                // SAFETY: the world allows mutable access to `Assets<T>`
                unsafe { reflect_asset.get_unchecked_mut(world.world(), handle) };
                match asset_value {
                    Some(value) => value,
                    None => {
                        errors::dead_asset_handle(ui, handle_id);
                        return Some(false);
                    }
                }
            };

            let mut restricted_env = InspectorUi {
                type_registry: env.type_registry,
                context: &mut Context {
                    world: Some(world),
                    queue: queue.as_deref_mut(),
                },
                short_circuit: env.short_circuit,
                short_circuit_readonly: env.short_circuit_readonly,
                short_circuit_many: env.short_circuit_many,
            };
            return Some(restricted_env.ui_for_reflect_with_options(
                asset_value.as_partial_reflect_mut(),
                ui,
                id.with("asset"),
                options,
            ));
        }

        None
    }

    pub fn short_circuit_many(
        env: &mut InspectorUi,
        type_id: TypeId,
        type_name: &str,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
        values: &mut [&mut dyn PartialReflect],
        projector: &dyn ProjectorReflect,
    ) -> Option<bool> {
        if let Some(reflect_handle) = env
            .type_registry
            .get_type_data::<bevy_asset::ReflectHandle>(type_id)
        {
            let Some(reflect_asset) = env
                .type_registry
                .get_type_data::<ReflectAsset>(reflect_handle.asset_type_id())
            else {
                errors::no_type_data(
                    ui,
                    &name_of_type(reflect_handle.asset_type_id(), env.type_registry),
                    "ReflectAsset",
                );
                return Some(false);
            };

            let Context {
                world: Some(world),
                queue,
            } = &mut env.context
            else {
                errors::no_world_in_context(ui, type_name);
                return Some(false);
            };

            let (assets_view, world) =
                world.split_off_resource(reflect_asset.assets_resource_type_id());

            let mut new_values = Vec::with_capacity(values.len());
            let mut used_handles = Vec::with_capacity(values.len());

            for value in values {
                let handle = projector(*value);
                let Some(handle) = handle.try_as_reflect() else {
                    // Edge case, continue as normal:
                    // this for loop should only work if we're multi-editing a bunch of Handles
                    return None;
                };
                let handle = reflect_handle
                    .downcast_handle_untyped(handle.as_any())
                    .unwrap();
                let handle_id = handle.id();

                if used_handles.contains(&handle_id) {
                    continue;
                };
                used_handles.push(handle_id);

                let asset_value = {
                    assert!(assets_view
                        .allows_access_to_resource(reflect_asset.assets_resource_type_id()));
                    let asset_value =
                        // SAFETY: the world allows mutable access to `Assets<T>`
                        unsafe { reflect_asset.get_unchecked_mut(world.world(), handle) };
                    match asset_value {
                        Some(value) => value,
                        None => {
                            errors::dead_asset_handle(ui, handle_id);
                            return Some(false);
                        }
                    }
                };

                new_values.push(asset_value.as_partial_reflect_mut());
            }

            let mut restricted_env = InspectorUi {
                type_registry: env.type_registry,
                context: &mut Context {
                    world: Some(world),
                    queue: queue.as_deref_mut(),
                },
                short_circuit: env.short_circuit,
                short_circuit_readonly: env.short_circuit_readonly,
                short_circuit_many: env.short_circuit_many,
            };
            return Some(restricted_env.ui_for_reflect_many_with_options(
                reflect_handle.asset_type_id(),
                "",
                ui,
                id.with("asset"),
                options,
                new_values.as_mut_slice(),
                &|a| a,
            ));
        }

        None
    }

    pub fn short_circuit_readonly(
        env: &mut InspectorUi,
        value: &dyn PartialReflect,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) -> Option<()> {
        let value = value.try_as_reflect()?;

        if let Some(reflect_handle) = env
            .type_registry
            .get_type_data::<bevy_asset::ReflectHandle>(value.type_id())
        {
            let handle = reflect_handle
                .downcast_handle_untyped(value.as_any())
                .unwrap();
            let handle_id = handle.id();
            let Some(reflect_asset) = env
                .type_registry
                .get_type_data::<ReflectAsset>(reflect_handle.asset_type_id())
            else {
                errors::no_type_data(
                    ui,
                    &name_of_type(reflect_handle.asset_type_id(), env.type_registry),
                    "ReflectAsset",
                );
                return Some(());
            };

            let Context {
                world: Some(world),
                queue,
            } = &mut env.context
            else {
                errors::no_world_in_context(ui, value.reflect_short_type_path());
                return Some(());
            };

            let (assets_view, world) =
                world.split_off_resource(reflect_asset.assets_resource_type_id());

            let asset_value = {
                // SAFETY: the following code only accesses a resources it has access to, `Assets<T>`
                let interior_mutable_world = unsafe { assets_view.world().world() };
                assert!(
                    assets_view.allows_access_to_resource(reflect_asset.assets_resource_type_id())
                );
                let asset_value = reflect_asset.get(interior_mutable_world, handle);
                match asset_value {
                    Some(value) => value,
                    None => {
                        errors::dead_asset_handle(ui, handle_id);
                        return Some(());
                    }
                }
            }
            .as_partial_reflect();

            let mut restricted_env = InspectorUi {
                type_registry: env.type_registry,
                context: &mut Context {
                    world: Some(world),
                    queue: queue.as_deref_mut(),
                },
                short_circuit: env.short_circuit,
                short_circuit_readonly: env.short_circuit_readonly,
                short_circuit_many: env.short_circuit_many,
            };
            restricted_env.ui_for_reflect_readonly_with_options(
                asset_value,
                ui,
                id.with("asset"),
                options,
            );
            return Some(());
        }

        None
    }
}

pub use crate::utils::guess_entity_name::guess_entity_name;
