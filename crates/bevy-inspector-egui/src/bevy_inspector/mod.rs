//! Methods for displaying `bevy` resources, assets and entities
//!
//! # Example
//!
//! ```rust
//! use bevy_inspector_egui::bevy_inspector;
//! # use bevy_ecs::prelude::*;
//! # use bevy_reflect::Reflect;
//! # use bevy_render::prelude::Msaa;
//! # use bevy_math::Vec3;
//!
//! #[derive(Debug, Clone, Eq, PartialEq, Hash, Reflect)]
//! enum AppState { A, B, C }
//!
//! fn show_ui(world: &mut World, ui: &mut egui::Ui) {
//!     let mut any_reflect_value = Vec3::new(1.0, 2.0, 3.0);
//!     bevy_inspector::ui_for_value(&mut any_reflect_value, ui, world);
//!
//!     ui.heading("Msaa resource");
//!     bevy_inspector::ui_for_resource::<Msaa>(world, ui);
//!
//!     ui.heading("App State");
//!     bevy_inspector::ui_for_state::<AppState>(world, ui);
//!
//!     egui::CollapsingHeader::new("Entities")
//!         .default_open(true)
//!         .show(ui, |ui| {
//!             bevy_inspector::ui_for_world_entities(world, ui);
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

use bevy_app::prelude::AppTypeRegistry;
use bevy_asset::{Asset, Assets, ReflectAsset};
use bevy_ecs::schedule::StateData;
use bevy_ecs::{component::ComponentId, prelude::*, world::EntityRef};
use bevy_hierarchy::{Children, Parent};
use bevy_reflect::{Reflect, TypeRegistry};
use pretty_type_name::pretty_type_name;

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

    let mut cx = Context {
        world: Some(RestrictedWorldView::new(world)),
    };
    let mut env = InspectorUi::for_bevy(&type_registry, &mut cx);
    env.ui_for_reflect(value, ui)
}

/// Display `Entities`, `Resources` and `Assets` using their respective functions inside headers
pub fn ui_for_world(world: &mut World, ui: &mut egui::Ui) {
    egui::CollapsingHeader::new("Entities")
        .default_open(true)
        .show(ui, |ui| {
            ui_for_world_entities(world, ui);
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
        .map(|registration| (registration.short_name().to_owned(), registration.type_id()))
        .collect();
    resources.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));
    for (name, type_id) in resources {
        ui.collapsing(&name, |ui| {
            by_type_id::ui_for_resource(world, type_id, ui, &name, &type_registry);
        });
    }
}

/// Display the resource `R`
pub fn ui_for_resource<R: Resource + Reflect>(world: &mut World, ui: &mut egui::Ui) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    // create a context with access to the world except for the `R` resource
    let Some((mut resource, world)) = RestrictedWorldView::new(world).split_off_resource_typed::<R>() else {
        errors::resource_does_not_exist(ui, &pretty_type_name::<R>());
        return;
    };
    let mut cx = Context { world: Some(world) };
    let mut env = InspectorUi::for_bevy(&type_registry, &mut cx);

    if env.ui_for_reflect(resource.bypass_change_detection(), ui) {
        resource.set_changed();
    }
}

/// Display all reflectable assets
pub fn ui_for_all_assets(world: &mut World, ui: &mut egui::Ui) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    let mut assets: Vec<_> = type_registry
        .iter()
        .filter(|registration| registration.data::<ReflectAsset>().is_some())
        .map(|registration| (registration.short_name().to_owned(), registration.type_id()))
        .collect();
    assets.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));
    for (name, type_id) in assets {
        ui.collapsing(&name, |ui| {
            by_type_id::ui_for_assets(world, type_id, ui, &type_registry);
        });
    }
}

/// Display all assets of the specified asset type `A`
pub fn ui_for_assets<A: Asset + Reflect>(world: &mut World, ui: &mut egui::Ui) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    // create a context with access to the world except for the `R` resource
    let Some((mut assets, world)) = RestrictedWorldView::new(world).split_off_resource_typed::<Assets<A>>() else {
        errors::resource_does_not_exist(ui, &pretty_type_name::<Assets<A>>());
        return;
    };
    let mut cx = Context { world: Some(world) };

    let mut assets: Vec<_> = assets.iter_mut().collect();
    assets.sort_by(|(a, _), (b, _)| a.cmp(b));
    for (handle_id, asset) in assets {
        let id = egui::Id::new(handle_id);

        egui::CollapsingHeader::new(format!("Handle({id:?})"))
            .id_source(id)
            .show(ui, |ui| {
                let mut env = InspectorUi::for_bevy(&type_registry, &mut cx);
                env.ui_for_reflect_with_options(asset, ui, id, &());
            });
    }
}

/// Display state `T` and change state on edit
pub fn ui_for_state<T: StateData + Reflect>(world: &mut World, ui: &mut egui::Ui) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    // create a context with access to the world except for the `State<T>` resource
    let Some((mut state, world)) = RestrictedWorldView::new(world).split_off_resource_typed::<State<T>>() else {
        errors::state_does_not_exist(ui, &pretty_type_name::<T>());
        return;
    };
    let mut cx = Context { world: Some(world) };
    let mut env = InspectorUi::for_bevy(&type_registry, &mut cx);

    let mut current = state.current().clone();

    let changed = env.ui_for_reflect(&mut current, ui);
    if changed {
        if let Err(e) = state.set(current) {
            ui.label(format!("{e:?}"));
        }
    }
}

/// Display all entities and their components
pub fn ui_for_world_entities(world: &mut World, ui: &mut egui::Ui) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    let mut root_entities = world.query_filtered::<Entity, Without<Parent>>();
    let mut entities = root_entities.iter(world).collect::<Vec<_>>();
    entities.sort();

    let id = egui::Id::new("world ui");
    for entity in entities {
        ui_for_entity_inner(world, entity, ui, id.with(entity), &type_registry, true);
    }
}

/// Display the given entity with all its components and children
pub fn ui_for_entity(world: &mut World, entity: Entity, ui: &mut egui::Ui, in_header: bool) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    ui_for_entity_inner(
        world,
        entity,
        ui,
        egui::Id::new(entity),
        &type_registry,
        in_header,
    )
}

fn ui_for_entity_inner(
    world: &mut World,
    entity: Entity,
    ui: &mut egui::Ui,
    id: egui::Id,
    type_registry: &TypeRegistry,
    in_header: bool,
) {
    let entity_name = guess_entity_name(world, type_registry, entity);

    let mut inner = |ui: &mut egui::Ui| {
        ui_for_entity_components(world, entity, ui, id, type_registry);

        let children = world
            .get::<Children>(entity)
            .map(|children| children.iter().copied().collect::<Vec<_>>());
        if let Some(children) = children {
            if !children.is_empty() {
                ui.label("Children");
                for &child in children.iter() {
                    let id = id.with(child);
                    ui_for_entity_inner(world, child, ui, id, type_registry, true);
                }
            }
        }
    };

    if in_header {
        egui::CollapsingHeader::new(entity_name)
            .id_source(id)
            .show(ui, |ui| {
                inner(ui);
            });
    } else {
        ui.label(entity_name);
        inner(ui);
    }
}

/// Display the components of the given entity
fn ui_for_entity_components(
    world: &mut World,
    entity: Entity,
    ui: &mut egui::Ui,
    id: egui::Id,
    type_registry: &TypeRegistry,
) {
    let entity_ref = match world.get_entity(entity) {
        Some(entity) => entity,
        None => {
            errors::entity_does_not_exist(ui, entity);
            return;
        }
    };
    let components = components_of_entity(entity_ref, world);

    for (name, component_id, component_type_id, size) in components {
        let id = id.with(component_id);

        let header = egui::CollapsingHeader::new(&name).id_source(id);

        let Some(component_type_id) = component_type_id else {
                header.show(ui, |ui| errors::no_type_id(ui, &name));
                return;
            };

        // create a context with access to the world except for the currently viewed component
        let mut world = RestrictedWorldView::new(world);
        let (mut component_view, world) = world.split_off_component((entity, component_type_id));
        let mut cx = Context { world: Some(world) };

        let (value, is_changed, set_changed) = match component_view.get_entity_component_reflect(
            entity,
            component_type_id,
            type_registry,
        ) {
            Ok(value) => value,
            Err(e) => {
                header.show(ui, |ui| errors::show_error(e, ui, &name));
                return;
            }
        };

        if is_changed {
            set_highlight_style(ui);
        }

        header.show(ui, |ui| {
            ui.reset_style();

            if size == 0 {
                return;
            }

            let inspector_changed = InspectorUi::for_bevy(type_registry, &mut cx)
                .ui_for_reflect_with_options(value, ui, id.with(component_id), &());

            if inspector_changed {
                set_changed();
            }
        });
        ui.reset_style();
    }
}

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
    entity_ref: EntityRef,
    world: &World,
) -> Vec<(String, ComponentId, Option<TypeId>, usize)> {
    let archetype = entity_ref.archetype();
    let mut components: Vec<_> = archetype
        .components()
        .map(|component_id| {
            let info = world.components().get_info(component_id).unwrap();
            let name = pretty_type_name::pretty_type_name_str(info.name());

            (name, component_id, info.type_id(), info.layout().size())
        })
        .collect();
    components.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));
    components
}

/// Display the given entity with all its components and children
pub fn ui_for_entities_shared_components(
    world: &mut World,
    entities: &[Entity],
    ui: &mut egui::Ui,
) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    let Some(&first) = entities.first() else { return };

    let Some(entity_ref) = world.get_entity(first) else {
        return errors::entity_does_not_exist(ui, first);
    };

    let mut components = components_of_entity(entity_ref, world);

    for &entity in entities.iter().skip(1) {
        components.retain(|(_, id, _, _)| {
            world
                .get_entity(entity)
                .map_or(true, |entity| entity.contains_id(*id))
        })
    }

    let (resources_view, components_view) = RestrictedWorldView::resources_components(world);
    let mut cx = Context {
        world: Some(resources_view),
    };
    let mut env = InspectorUi::for_bevy(&type_registry, &mut cx);

    let id = egui::Id::null();
    for (name, component_id, component_type_id, size) in components {
        let id = id.with(component_id);
        egui::CollapsingHeader::new(&name)
            .id_source(id)
            .show(ui, |ui| {
                if size == 0 {
                    return;
                }
                let Some(component_type_id) = component_type_id else {
                    return errors::no_type_id(ui, &name);
                };

                let mut values = Vec::with_capacity(entities.len());
                let mut mark_changeds = Vec::with_capacity(entities.len());

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
                        Ok((value, mark_changed)) => {
                            values.push(value);
                            mark_changeds.push(mark_changed);
                        }
                        Err(error) => {
                            errors::show_error(error, ui, &name);
                            return;
                        }
                    }
                }

                let changed = env.ui_for_reflect_many_with_options(
                    component_type_id,
                    &name,
                    ui,
                    id.with(component_id),
                    &(),
                    values.as_mut_slice(),
                    &|a| a,
                );
                if changed {
                    mark_changeds.into_iter().for_each(|f| f());
                }
            });
    }
}

pub mod by_type_id {
    use std::any::TypeId;

    use bevy_asset::{HandleId, HandleUntyped, ReflectAsset, ReflectHandle};
    use bevy_ecs::prelude::*;
    use bevy_reflect::TypeRegistry;

    use crate::{
        reflect_inspector::{Context, InspectorUi},
        restricted_world_view::RestrictedWorldView,
    };

    use super::errors::{self, name_of_type};

    /// Display the resource with the given [`TypeId`]
    pub fn ui_for_resource(
        world: &mut World,
        resource_type_id: TypeId,
        ui: &mut egui::Ui,
        name_of_type: &str,
        type_registry: &TypeRegistry,
    ) {
        // create a context with access to the world except for the current resource
        let mut world = RestrictedWorldView::new(world);
        let (mut resource_view, world) = world.split_off_resource(resource_type_id);
        let mut cx = Context { world: Some(world) };
        let mut env = InspectorUi::for_bevy(type_registry, &mut cx);

        let (resource, set_changed) =
            match resource_view.get_resource_reflect_mut_by_id(resource_type_id, type_registry) {
                Ok(resource) => resource,
                Err(err) => return errors::show_error(err, ui, name_of_type),
            };

        let changed = env.ui_for_reflect(resource, ui);
        if changed {
            set_changed();
        }
    }

    /// Display all assets of the given asset [`TypeId`]
    pub fn ui_for_assets(
        world: &mut World,
        asset_type_id: TypeId,
        ui: &mut egui::Ui,
        type_registry: &TypeRegistry,
    ) {
        let Some(registration) = type_registry.get(asset_type_id) else {
            return crate::reflect_inspector::errors::not_in_type_registry(ui, &name_of_type(asset_type_id, type_registry));
        };
        let Some(reflect_asset) = registration.data::<ReflectAsset>() else {
            return errors::no_type_data(ui, &name_of_type(asset_type_id, type_registry), "ReflectAsset");
        };
        let Some(reflect_handle) = type_registry.get_type_data::<ReflectHandle>(reflect_asset.handle_type_id()) else {
            return errors::no_type_data(ui, &name_of_type(reflect_asset.handle_type_id(), type_registry), "ReflectHandle");
        };

        let mut ids: Vec<_> = reflect_asset.ids(world).collect();
        ids.sort();

        // Create a context with access to the entire world. Displaying the `Handle<T>` will short circuit into
        // displaying the T with a world view excluding Assets<T>.
        let world = RestrictedWorldView::new(world);
        let mut cx = Context { world: Some(world) };

        for handle_id in ids {
            let id = egui::Id::new(handle_id);
            let mut handle = reflect_handle.typed(HandleUntyped::weak(handle_id));

            egui::CollapsingHeader::new(format!("Handle({id:?})"))
                .id_source(id)
                .show(ui, |ui| {
                    let mut env = InspectorUi::for_bevy(type_registry, &mut cx);
                    env.ui_for_reflect_with_options(&mut *handle, ui, id, &());
                });
        }
    }

    /// Display a given asset by handle and asset [`TypeId`]
    pub fn ui_for_asset(
        world: &mut World,
        asset_type_id: TypeId,
        handle: HandleId,
        ui: &mut egui::Ui,
        type_registry: &TypeRegistry,
    ) {
        let Some(registration) = type_registry.get(asset_type_id) else {
            return crate::reflect_inspector::errors::not_in_type_registry(ui, &name_of_type(asset_type_id, type_registry));
        };
        let Some(reflect_asset) = registration.data::<ReflectAsset>() else {
            return errors::no_type_data(ui, &name_of_type(asset_type_id, type_registry), "ReflectAsset");
        };
        let Some(reflect_handle) = type_registry.get_type_data::<ReflectHandle>(reflect_asset.handle_type_id()) else {
            return errors::no_type_data(ui, &name_of_type(reflect_asset.handle_type_id(), type_registry), "ReflectHandle");
        };

        let mut ids: Vec<_> = reflect_asset.ids(world).collect();
        ids.sort();

        // Create a context with access to the entire world. Displaying the `Handle<T>` will short circuit into
        // displaying the T with a world view excluding Assets<T>.
        let world = RestrictedWorldView::new(world);
        let mut cx = Context { world: Some(world) };

        let id = egui::Id::new(handle);
        let mut handle = reflect_handle.typed(HandleUntyped::weak(handle));

        let mut env = InspectorUi::for_bevy(type_registry, &mut cx);
        env.ui_for_reflect_with_options(&mut *handle, ui, id, &());
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
    use bevy_reflect::Reflect;

    use crate::reflect_inspector::{Context, InspectorUi};

    use super::errors::{self, name_of_type};

    pub fn short_circuit(
        env: &mut InspectorUi,
        value: &mut dyn Reflect,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) -> Option<bool> {
        if let Some(reflect_handle) = env
            .type_registry
            .get_type_data::<bevy_asset::ReflectHandle>(Any::type_id(value))
        {
            let handle = reflect_handle
                .downcast_handle_untyped(value.as_any())
                .unwrap();
            let handle_id = handle.id;
            let Some(reflect_asset) = env
            .type_registry
            .get_type_data::<ReflectAsset>(reflect_handle.asset_type_id())
            else {
                errors::no_type_data(ui, &name_of_type(reflect_handle.asset_type_id(), env.type_registry), "ReflectAsset");
                return Some(false);
            };

            let Some(world) = &mut env.context.world else {
                errors::no_world_in_context(ui, value.type_name());
                return Some(false);
            };

            let (assets_view, world) =
                world.split_off_resource(reflect_asset.assets_resource_type_id());

            let asset_value = {
                // SAFETY: the following code only accesses a resources it has access to, `Assets<T>`
                // The world borrow is then immediately discarded and not live while the other part of the world is continued to be used
                let interior_mutable_world = unsafe { assets_view.get() };
                assert!(
                    assets_view.allows_access_to_resource(reflect_asset.assets_resource_type_id())
                );
                let asset_value =
                // SAFETY: the world allows mutable access to `Assets<T>`
                unsafe { reflect_asset.get_unchecked_mut(interior_mutable_world, handle) };
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
                context: &mut Context { world: Some(world) },
                short_circuit: env.short_circuit,
                short_circuit_readonly: env.short_circuit_readonly,
                short_circuit_many: env.short_circuit_many,
            };
            return Some(restricted_env.ui_for_reflect_with_options(
                asset_value,
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
        values: &mut [&mut dyn Reflect],
        projector: &dyn Fn(&mut dyn Reflect) -> &mut dyn Reflect,
    ) -> Option<bool> {
        if let Some(reflect_handle) = env
            .type_registry
            .get_type_data::<bevy_asset::ReflectHandle>(type_id)
        {
            let Some(reflect_asset) = env
                .type_registry
                .get_type_data::<ReflectAsset>(reflect_handle.asset_type_id())
            else {
                errors::no_type_data(ui, &name_of_type(reflect_handle.asset_type_id(), env.type_registry), "ReflectAsset");
                return Some(false);
            };

            let Some(world) = &mut env.context.world else {
                errors::no_world_in_context(ui, type_name);
                return Some(false);
            };

            let (assets_view, world) =
                world.split_off_resource(reflect_asset.assets_resource_type_id());

            let mut new_values = Vec::with_capacity(values.len());
            let mut used_handles = Vec::with_capacity(values.len());

            for value in values {
                let handle = projector(*value);
                let handle = reflect_handle
                    .downcast_handle_untyped(handle.as_any())
                    .unwrap();
                let handle_id = handle.id;

                if used_handles.contains(&handle_id) {
                    continue;
                };
                used_handles.push(handle.id);

                let asset_value = {
                    // SAFETY: the following code only accesses a resources it has access to, `Assets<T>`
                    // The world borrow is then immediately discarded and not live while the other part of the world is continued to be used
                    let interior_mutable_world = unsafe { assets_view.get() };
                    assert!(assets_view
                        .allows_access_to_resource(reflect_asset.assets_resource_type_id()));
                    let asset_value =
                        // SAFETY: the world allows mutable access to `Assets<T>` 
                        unsafe { reflect_asset.get_unchecked_mut(interior_mutable_world, handle) };
                    match asset_value {
                        Some(value) => value,
                        None => {
                            errors::dead_asset_handle(ui, handle_id);
                            return Some(false);
                        }
                    }
                };

                new_values.push(asset_value);
            }

            let mut restricted_env = InspectorUi {
                type_registry: env.type_registry,
                context: &mut Context { world: Some(world) },
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
        value: &dyn Reflect,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &dyn Any,
    ) -> Option<()> {
        if let Some(reflect_handle) = env
            .type_registry
            .get_type_data::<bevy_asset::ReflectHandle>(Any::type_id(value))
        {
            let handle = reflect_handle
                .downcast_handle_untyped(value.as_any())
                .unwrap();
            let handle_id = handle.id;
            let Some(reflect_asset) = env
            .type_registry
            .get_type_data::<ReflectAsset>(reflect_handle.asset_type_id())
            else {
                errors::no_type_data(ui, &name_of_type(reflect_handle.asset_type_id(), env.type_registry), "ReflectAsset");
                return Some(());
            };

            let Some(world) = &mut env.context.world else {
                errors::no_world_in_context(ui, value.type_name());
                return Some(());
            };

            let (assets_view, world) =
                world.split_off_resource(reflect_asset.assets_resource_type_id());

            let asset_value = {
                // SAFETY: the following code only accesses a resources it has access to, `Assets<T>`
                let interior_mutable_world = unsafe { assets_view.get() };
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
            };

            let mut restricted_env = InspectorUi {
                type_registry: env.type_registry,
                context: &mut Context { world: Some(world) },
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
