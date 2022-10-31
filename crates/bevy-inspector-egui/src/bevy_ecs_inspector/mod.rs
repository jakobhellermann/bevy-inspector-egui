//! Methods for displaying `bevy` resources, assets and entities

use std::any::{Any, TypeId};

use bevy_app::prelude::AppTypeRegistry;
use bevy_asset::{HandleUntyped, ReflectAsset, ReflectHandle};
use bevy_ecs::{component::ComponentId, prelude::*, world::EntityRef};
use bevy_hierarchy::{Children, Parent};
use bevy_reflect::{Reflect, TypeRegistry};
use egui::FontId;

pub(crate) mod errors;

/// UI for displaying the entity hierarchy
pub mod hierarchy;

use crate::split_world_permission;
use crate::{
    egui_reflect_inspector::{Context, InspectorUi},
    egui_utils::layout_job,
    split_world_permission::split_world_permission,
    utils::guess_entity_name,
};

use self::errors::name_of_type;

/// Display `Entities`, `Resources` and `Assets` using their respective functions inside headers
pub fn ui_for_world(world: &mut World, ui: &mut egui::Ui) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    egui::CollapsingHeader::new("Entities")
        .default_open(true)
        .show(ui, |ui| {
            ui_for_world_entities(world, ui, &type_registry);
        });
    egui::CollapsingHeader::new("Resources").show(ui, |ui| {
        ui_for_resources(world, ui, &type_registry);
    });
    egui::CollapsingHeader::new("Assets").show(ui, |ui| {
        ui_for_assets(world, ui, &type_registry);
    });
}

/// Display all reflectable resources in the world
pub fn ui_for_resources(world: &mut World, ui: &mut egui::Ui, type_registry: &TypeRegistry) {
    let mut resources: Vec<_> = type_registry
        .iter()
        .filter(|registration| registration.data::<ReflectResource>().is_some())
        .map(|registration| (registration.short_name().to_owned(), registration.type_id()))
        .collect();
    resources.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));
    for (name, type_id) in resources {
        ui.collapsing(&name, |ui| {
            ui_for_resource(world, type_id, ui, &type_registry);
        });
    }
}

fn show_error(
    error: split_world_permission::Error,
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
) {
    match error {
        split_world_permission::Error::ResourceDoesNotExist(type_id) => {
            errors::resource_does_not_exist(ui, &name_of_type(type_id, type_registry));
        }
        split_world_permission::Error::NoTypeRegistration(type_id) => {
            errors::not_in_type_registry(ui, &name_of_type(type_id, type_registry));
        }
        split_world_permission::Error::NoTypeData(type_id, data) => {
            errors::no_type_data(ui, &name_of_type(type_id, type_registry), data);
        }
    }
}

/// Display the resource with the given [`TypeId`]
pub fn ui_for_resource(
    world: &mut World,
    resource_type_id: TypeId,
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
) {
    // no_resource_refs_world can only access `resource_type_id`, only_resource_access_world can access everything else
    let (mut no_resource_refs_world, only_resource_access_world) =
        split_world_permission(world, Some(resource_type_id));

    let (resource, set_changed) = match no_resource_refs_world
        .get_resource_reflect_mut_by_id(resource_type_id, type_registry)
    {
        Ok(resource) => resource,
        Err(err) => return show_error(err, ui, type_registry),
    };

    let mut cx = Context {
        world: Some(only_resource_access_world),
    };
    let mut env = InspectorUi::for_bevy(type_registry, &mut cx);

    let changed = env.ui_for_reflect(resource, ui, egui::Id::new(resource_type_id));

    if changed {
        set_changed();
    }
}

/// Display all reflectable assets
pub fn ui_for_assets(world: &mut World, ui: &mut egui::Ui, type_registry: &TypeRegistry) {
    let mut assets: Vec<_> = type_registry
        .iter()
        .filter(|registration| registration.data::<ReflectAsset>().is_some())
        .map(|registration| (registration.short_name().to_owned(), registration.type_id()))
        .collect();
    assets.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));
    for (name, type_id) in assets {
        ui.collapsing(&name, |ui| {
            ui_for_asset(world, type_id, ui, &type_registry);
        });
    }
}

/// Display all assets of the given asset [`TypeId`]
pub fn ui_for_asset(
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

    let (_, only_resource_access_world) = split_world_permission(world, None);
    let mut cx = Context {
        world: Some(only_resource_access_world),
    };

    for handle_id in ids {
        let id = egui::Id::new(handle_id);
        let mut handle = reflect_handle.typed(HandleUntyped::weak(handle_id));

        egui::CollapsingHeader::new(format!("Handle({id:?})"))
            .id_source(id)
            .show(ui, |ui| {
                let mut env = InspectorUi::for_bevy(type_registry, &mut cx);
                env.ui_for_reflect(&mut *handle, ui, id);
            });
    }
}

/// Display all entities and their components
pub fn ui_for_world_entities(world: &mut World, ui: &mut egui::Ui, type_registry: &TypeRegistry) {
    let mut root_entities = world.query_filtered::<Entity, Without<Parent>>();
    let mut entities = root_entities.iter(world).collect::<Vec<_>>();
    entities.sort();

    let id = egui::Id::new("world ui");
    for entity in entities {
        ui_for_entity(world, entity, ui, id.with(entity), type_registry, true);
    }
}

/// Display the given entity with all its components and its children
pub fn ui_for_entity(
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
                    ui_for_entity(world, child, ui, id, type_registry, true);
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

    let (mut no_resource_refs_world, only_resource_access_world) =
        split_world_permission(world, None);
    let mut cx = Context {
        world: Some(only_resource_access_world),
    };

    for (name, component_id, type_id, size) in components {
        let id = id.with(component_id);
        egui::CollapsingHeader::new(&name)
            .id_source(id)
            .show(ui, |ui| {
                if size == 0 {
                    return;
                }
                let Some(type_id) = type_id else {
                    return error_message_no_type_id(ui, &name);
                };

                let (value, set_changed) = match no_resource_refs_world
                    .get_entity_component_reflect(entity, type_id, type_registry)
                {
                    Ok(value) => value,
                    Err(e) => return show_error(e, ui, type_registry),
                };

                let changed = InspectorUi::for_bevy(type_registry, &mut cx).ui_for_reflect(
                    value,
                    ui,
                    id.with(component_id),
                );

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

fn error_message_no_type_id(ui: &mut egui::Ui, component_name: &str) {
    let job = layout_job(&[
        (FontId::monospace(14.0), component_name),
        (
            FontId::default(),
            " is not backed by a rust type, so it cannot be displayed.",
        ),
    ]);

    ui.label(job);
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

        let Some(world) = &env.context.world else {
            errors::no_world_in_context(ui, value.type_name());
            return Some(false);
        };
        assert!(!world.forbids_access_to(reflect_asset.assets_resource_type_id()));
        // SAFETY: the following code only accesses resources through the world (namely `Assets<T>`)
        let ora_world = unsafe { world.get() };
        // SAFETY: the `OnlyResourceAccessWorld` allows mutable access (except for the `except_resource`),
        // and we create only one reference to an asset at the same time.
        let asset_value = unsafe { reflect_asset.get_unchecked_mut(ora_world, handle) };
        let Some(asset_value) = asset_value else {
            errors::dead_asset_handle(ui, handle_id);
            return Some(false);
        };

        let more_restricted_world = env.context.world.as_ref().map(|world| {
            // SAFETY: while this world is active, the only live reference to a resource through the `world` is
            // through the `assets_resource_type_id`.
            unsafe { world.with_more_restriction(reflect_asset.assets_resource_type_id()) }
        });

        let mut restricted_env = InspectorUi {
            type_registry: env.type_registry,
            context: &mut Context {
                world: more_restricted_world,
            },
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

        let Some(world) = &env.context.world else {
            errors::no_world_in_context(ui, value.type_name());
            return Some(());
        };
        assert!(!world.forbids_access_to(reflect_asset.assets_resource_type_id()));
        // SAFETY: the following code only accesses resources through the world (namely `Assets<T>`)
        let ora_world = unsafe { world.get() };
        let asset_value = reflect_asset.get(ora_world, handle);
        let asset_value = match asset_value {
            Some(value) => value,
            None => {
                errors::dead_asset_handle(ui, handle_id);
                return Some(());
            }
        };

        let more_restricted_world = env.context.world.as_ref().map(|world| {
            // SAFETY: while this world is active, the only live reference to a resource through the `world` is
            // through the `assets_resource_type_id`.
            unsafe { world.with_more_restriction(reflect_asset.assets_resource_type_id()) }
        });

        let mut restricted_env = InspectorUi {
            type_registry: env.type_registry,
            context: &mut Context {
                world: more_restricted_world,
            },
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
