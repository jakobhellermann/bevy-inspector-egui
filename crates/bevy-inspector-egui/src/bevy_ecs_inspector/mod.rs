//! Methods for displaying `bevy` resources, assets and entities

use std::any::{Any, TypeId};

use bevy_app::prelude::AppTypeRegistry;
use bevy_asset::{HandleUntyped, ReflectAsset, ReflectHandle};
use bevy_ecs::change_detection::MutUntyped;
use bevy_ecs::ptr::PtrMut;
use bevy_ecs::{component::ComponentId, prelude::*, world::EntityRef};
use bevy_hierarchy::{Children, Parent};
use bevy_reflect::{Reflect, ReflectFromPtr, TypeRegistry};
use egui::FontId;

pub(crate) mod errors;

/// UI for displaying the entity hierarchy
pub mod hierarchy;

use crate::{
    egui_reflect_inspector::{Context, InspectorUi},
    egui_utils::layout_job,
    split_world_permission::split_world_permission,
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

/// Display the resource with the given [`TypeId`]
pub fn ui_for_resource(
    world: &mut World,
    resource_type_id: TypeId,
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
) {
    let (no_resource_refs_world, only_resource_access_world) =
        split_world_permission(world, Some(resource_type_id));

    let mut cx = Context {
        world: Some(only_resource_access_world),
    };
    let mut env = InspectorUi::for_bevy(type_registry, &mut cx);

    // SAFETY: in the code below, the only reference to a resource is the one specified as `except` in `split_world_permission`;
    debug_assert!(no_resource_refs_world.allows_access_to(resource_type_id));
    let nrr_world = unsafe { no_resource_refs_world.get() };
    let Some(component_id) = nrr_world.components().get_resource_id(resource_type_id) else {
        return errors::resource_does_not_exist(ui, &name_of_type(resource_type_id, type_registry),
        );
    };
    // SAFETY: component_id refers to the component use as the exception in `split_world_permission`,
    // `NoResourceRefsWorld` allows mutable access.
    let Some(mut_untyped) = (unsafe { nrr_world.get_resource_unchecked_mut_by_id(component_id) }) else {
        return errors::resource_does_not_exist(ui, &name_of_type(resource_type_id, type_registry));
    };
    let (ptr, set_changed) = mut_untyped_split(mut_untyped);

    let Some(registration) = type_registry.get(resource_type_id) else {
        return errors::not_in_type_registry(ui, &name_of_type(resource_type_id, type_registry));
    };
    let Some(reflect_from_ptr) = registration.data::<ReflectFromPtr>() else {
        return errors::no_type_data(ui, &name_of_type(resource_type_id, type_registry), "ReflectFromPtr");
    };
    assert_eq!(reflect_from_ptr.type_id(), resource_type_id);
    // SAFETY: value type is the type of the `ReflectFromPtr`
    let value = unsafe { reflect_from_ptr.as_reflect_ptr_mut(ptr) };
    let changed = env.ui_for_reflect(value, ui, egui::Id::new(resource_type_id));

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

    let (no_resource_refs_world, only_resource_access_world) = split_world_permission(world, None);
    let mut cx = Context {
        world: Some(only_resource_access_world),
    };
    // SAFETY: in the code below, no references to resources are held
    let nrr_world = unsafe { no_resource_refs_world.get() };

    for (name, component_id, type_id, size) in components {
        let id = id.with(component_id);
        egui::CollapsingHeader::new(&name)
            .id_source(id)
            .show(ui, |ui| {
                // SAFETY: mutable access is allowed through `NoResourceRefsWorld`, just not to resources
                let value = unsafe {
                    nrr_world
                        .entity(entity)
                        .get_unchecked_mut_by_id(component_id)
                        .unwrap()
                };
                let (ptr, set_changed) = mut_untyped_split(value);

                if size == 0 {
                    return;
                }

                let type_id = match type_id {
                    Some(type_id) => type_id,
                    None => return error_message_no_type_id(ui, &name),
                };
                let Some(reflect_from_ptr) = type_registry.get_type_data::<ReflectFromPtr>(type_id) else {
                    return errors::no_type_data(ui, &name, "ReflectFromPtr");
                };
                assert_eq!(reflect_from_ptr.type_id(), type_id);
                // SAFETY: value is of correct type, as checked above
                let value = unsafe { reflect_from_ptr.as_reflect_ptr_mut(ptr) };

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

mod guess_entity_name {
    use bevy_core::Name;
    use bevy_ecs::{prelude::*, world::EntityRef};
    use bevy_reflect::TypeRegistry;

    /// Guesses an appropriate entity name like `Light (6)` or falls back to `Entity (8)`
    pub fn entity_name(world: &World, type_registry: &TypeRegistry, entity: Entity) -> String {
        match world.get_entity(entity) {
            Some(entity) => guess_entity_name_inner(world, entity, type_registry),
            None => format!("Entity {} (inexistent)", entity.id()),
        }
    }

    fn guess_entity_name_inner(
        world: &World,
        entity: EntityRef,
        type_registry: &TypeRegistry,
    ) -> String {
        let id = entity.id().id();

        if let Some(name) = entity.get::<Name>() {
            return name.as_str().to_string();
        }

        #[rustfmt::skip]
        let associations = &[
            ("bevy_core_pipeline::core_3d::camera_3d::Camera3d", "Camera3d"),
            ("bevy_core_pipeline::core_2d::camera_2d::Camera2d", "Camera2d"),
            ("bevy_pbr::light::PointLight", "PointLight"),
            ("bevy_pbr::light::DirectionalLight", "DirectionalLight"),
            ("bevy_text::text::Text", "Text"),
            ("bevy_ui::ui_node::Node", "Node"),
            ("bevy_asset::handle::Handle<bevy_pbr::pbr_material::StandardMaterial>", "Pbr Mesh")

        ];

        let type_names = entity.archetype().components().filter_map(|id| {
            let type_id = world.components().get_info(id)?.type_id()?;
            let registration = type_registry.get(type_id)?;
            Some(registration.type_name())
        });

        for component_type in type_names {
            if let Some(name) = associations
                .iter()
                .find_map(|&(name, matches)| (component_type == name).then_some(matches))
            {
                return format!("{} ({:?})", name, id);
            }
        }

        format!("Entity ({:?})", id)
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
        restricted_env.ui_for_reflect_ref_with_options(asset_value, ui, id.with("asset"), options);
        return Some(());
    }

    None
}

fn mut_untyped_split<'a>(mut mut_untyped: MutUntyped<'a>) -> (PtrMut<'a>, impl FnOnce() + 'a) {
    // bypass_change_detection returns a `&mut PtrMut` which is basically useless, because all its methods take `self`
    let ptr = mut_untyped.bypass_change_detection();
    // SAFETY: this is exactly the same PtrMut, just not in a `&mut`. The old one is no longer accessible
    let ptr = unsafe { PtrMut::new(std::ptr::NonNull::new_unchecked(ptr.as_ptr())) };

    (ptr, move || mut_untyped.set_changed())
}