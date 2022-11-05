//! Methods for displaying `bevy` resources, assets and entities

use std::any::{Any, TypeId};

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

use crate::restricted_world_view::RestrictedWorldView;
use crate::{
    egui_reflect_inspector::{Context, InspectorUi},
    utils::guess_entity_name,
};

use self::errors::{name_of_type, show_error};

/// Display a single [`&mut dyn Reflect`](bevy_reflect::Reflect).
///
/// If you are wondering why this function takes in a [`&mut World`](bevy_ecs::world::World), it's so that if the value contains e.g. a
/// `Handle<StandardMaterial>` it can look up the corresponding asset resource and display the asset value inline.
///
/// If all you're displaying is a simple value without any references into the bevy world, consider just using
/// [`egui_reflect_inspector::ui_for_value`](crate::egui_reflect_inspector::ui_for_value).
pub fn ui_for_value(value: &mut dyn Reflect, world: &mut World, ui: &mut egui::Ui) -> bool {
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

    for (handle_id, asset) in assets.iter_mut() {
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
    let entity_name = guess_entity_name::entity_name(world, type_registry, entity);

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
        egui::CollapsingHeader::new(&name)
            .id_source(id)
            .show(ui, |ui| {
                if size == 0 {
                    return;
                }
                let Some(component_type_id) = component_type_id else {
                    return errors::error_message_no_type_id(ui, &name);
                };

                // create a context with access to the world except for the currently viewed component
                let mut world = RestrictedWorldView::new(world);
                let (mut component_view, world) =
                    world.split_off_component((entity, component_type_id));
                let mut cx = Context { world: Some(world) };

                let (value, set_changed) = match component_view.get_entity_component_reflect(
                    entity,
                    component_type_id,
                    type_registry,
                ) {
                    Ok(value) => value,
                    Err(e) => return show_error(e, ui, &name),
                };

                let changed = InspectorUi::for_bevy(type_registry, &mut cx)
                    .ui_for_reflect_with_options(value, ui, id.with(component_id), &());

                if changed {
                    set_changed();
                }
            });
    }
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

pub mod by_type_id {
    use std::any::TypeId;

    use bevy_asset::{HandleUntyped, ReflectAsset, ReflectHandle};
    use bevy_ecs::prelude::*;
    use bevy_reflect::TypeRegistry;

    use crate::{
        egui_reflect_inspector::{Context, InspectorUi},
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
        return errors::not_in_type_registry(ui, &name_of_type(asset_type_id, type_registry));
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
            Some(short_circuit),
            Some(short_circuit_readonly),
        )
    }
}

// Short circuit reflect UI in cases where we have better information available through the world (e.g. handles to assets)
fn short_circuit(
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
            let interior_mutable_world = unsafe { assets_view.get() };
            assert!(assets_view.allows_access_to_resource(reflect_asset.assets_resource_type_id()));
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

fn short_circuit_readonly(
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
            assert!(assets_view.allows_access_to_resource(reflect_asset.assets_resource_type_id()));
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
