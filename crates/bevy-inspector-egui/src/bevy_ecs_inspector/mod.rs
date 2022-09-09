use std::{
    any::TypeId,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use bevy_app::prelude::AppTypeRegistry;
use bevy_ecs::{component::ComponentId, prelude::*, world::EntityRef};
use bevy_hierarchy::{Children, Parent};
use bevy_reflect::{ReflectFromPtr, TypeRegistry};
use egui::FontId;

use crate::{
    driver_egui::{split_world_permission, Context, InspectorEguiOverrides, InspectorUi},
    egui_utils::layout_job,
};

#[derive(Resource, Default, Clone)]
pub struct AppInspectorEguiOverrides(Arc<RwLock<InspectorEguiOverrides>>);
impl AppInspectorEguiOverrides {
    pub fn read(&self) -> RwLockReadGuard<InspectorEguiOverrides> {
        self.0.read().unwrap()
    }
    pub fn write(&self) -> RwLockWriteGuard<InspectorEguiOverrides> {
        self.0.write().unwrap()
    }
}

pub fn ui_for_world(world: &mut World, ui: &mut egui::Ui) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();
    let egui_overrides = world
        .get_resource_or_insert_with(AppInspectorEguiOverrides::default)
        .clone();
    let egui_overrides = egui_overrides.read();

    egui::CollapsingHeader::new("Entities").show(ui, |ui| {
        ui_for_world_entities_with(world, ui, &type_registry, &egui_overrides);
    });
    egui::CollapsingHeader::new("Resources").show(ui, |ui| {
        let resources: Vec<_> = type_registry
            .iter()
            .filter(|registration| registration.data::<ReflectResource>().is_some())
            .map(|registration| (registration.short_name().to_owned(), registration.type_id()))
            .collect();
        for (name, type_id) in resources {
            ui.collapsing(&name, |ui| {
                ui_for_resource_with(world, type_id, ui, &type_registry, &egui_overrides);
            });
        }
    });
}

pub fn ui_for_resource(world: &mut World, resource_type_id: TypeId, ui: &mut egui::Ui) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();
    let egui_overrides = world
        .get_resource_or_insert_with(AppInspectorEguiOverrides::default)
        .clone();
    let egui_overrides = egui_overrides.read();

    ui_for_resource_with(world, resource_type_id, ui, &type_registry, &egui_overrides);
}

pub fn ui_for_resource_with(
    world: &mut World,
    resource_type_id: TypeId,
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    egui_overrides: &InspectorEguiOverrides,
) {
    let (no_resource_refs_world, only_resource_access_world) =
        split_world_permission(world, Some(resource_type_id));

    let mut cx = Context {
        world: Some(only_resource_access_world),
    };
    let mut env = InspectorUi::new(&type_registry, &egui_overrides, &mut cx);

    // SAFETY: in the code below, the only reference to a resource is the one specified as `except` in `split_world_permission`
    let nrr_world = unsafe { no_resource_refs_world.get() };
    let component_id = nrr_world
        .components()
        .get_resource_id(resource_type_id)
        .unwrap();
    // SAFETY: component_id refers to the component use as the exception in `split_world_permission`,
    // `NoResourceRefsWorld` allows mutable access.
    let value = unsafe {
        nrr_world
            .get_resource_mut_by_id_unchecked(component_id)
            .unwrap()
    };
    let reflect_from_ptr = type_registry
        .get_type_data::<ReflectFromPtr>(resource_type_id)
        .unwrap();
    assert_eq!(reflect_from_ptr.type_id(), resource_type_id);
    // SAFETY: value type is the type of the `ReflectFromPtr`
    let value = unsafe { reflect_from_ptr.as_reflect_ptr_mut(value.into_inner()) };
    env.ui_for_reflect(value, ui, egui::Id::new(resource_type_id));
}

pub fn ui_for_world_entities(world: &mut World, ui: &mut egui::Ui) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();
    let egui_overrides = world
        .get_resource_or_insert_with(AppInspectorEguiOverrides::default)
        .clone();
    let egui_overrides = egui_overrides.read();

    ui_for_world_entities_with(world, ui, &type_registry, &egui_overrides);
}

pub fn ui_for_world_entities_with(
    world: &mut World,
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    egui_overrides: &InspectorEguiOverrides,
) {
    let mut root_entities = world.query_filtered::<Entity, Without<Parent>>();
    let mut entities = root_entities.iter(world).collect::<Vec<_>>();
    entities.sort();

    let id = egui::Id::new("world ui");
    for entity in entities {
        ui_for_entity_with(
            world,
            entity,
            ui,
            id.with(entity),
            type_registry,
            egui_overrides,
        );
    }
}

pub fn ui_for_entity(world: &mut World, entity: Entity, ui: &mut egui::Ui) {
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();
    let egui_overrides = world
        .get_resource_or_insert_with(AppInspectorEguiOverrides::default)
        .clone();
    let egui_overrides = egui_overrides.read();

    ui_for_entity_with(
        world,
        entity,
        ui,
        egui::Id::new(entity),
        &type_registry,
        &egui_overrides,
    );
}

pub fn ui_for_entity_with(
    world: &mut World,
    entity: Entity,
    ui: &mut egui::Ui,
    id: egui::Id,
    type_registry: &TypeRegistry,
    egui_overrides: &InspectorEguiOverrides,
) {
    let entity_name = guess_entity_name::entity_name(world, entity);

    egui::CollapsingHeader::new(entity_name)
        .id_source(id)
        .show(ui, |ui| {
            ui_for_entity_components(world, entity, ui, id, type_registry, egui_overrides);

            let children = world
                .get::<Children>(entity)
                .map(|children| children.iter().copied().collect::<Vec<_>>());
            if let Some(children) = children {
                if !children.is_empty() {
                    ui.label("Children");
                    for &child in children.iter() {
                        let id = id.with(child);
                        ui_for_entity_with(world, child, ui, id, type_registry, egui_overrides);
                    }
                }
            }
        });
}

fn ui_for_entity_components(
    world: &mut World,
    entity: Entity,
    ui: &mut egui::Ui,
    id: egui::Id,
    type_registry: &TypeRegistry,
    egui_overrides: &InspectorEguiOverrides,
) {
    let entity_ref = match world.get_entity(entity) {
        Some(entity) => entity,
        None => {
            error_message_entity_does_not_exist(ui, entity);
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
                        .get_mut_by_id_unchecked(component_id)
                        .unwrap()
                };

                if size == 0 {
                    return;
                }

                let type_id = match type_id {
                    Some(type_id) => type_id,
                    None => return error_message_no_type_id(ui, &name),
                };
                let reflect_from_ptr = match type_registry.get_type_data::<ReflectFromPtr>(type_id)
                {
                    Some(type_id) => type_id,
                    None => return error_message_no_reflect_from_ptr(ui, &name),
                };
                assert_eq!(reflect_from_ptr.type_id(), type_id);
                // SAFETY: value is of correct type, as checked above
                let value = unsafe { reflect_from_ptr.as_reflect_ptr_mut(value.into_inner()) };

                InspectorUi::new(type_registry, egui_overrides, &mut cx).ui_for_reflect(
                    value,
                    ui,
                    id.with(component_id),
                );
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

fn error_message_no_reflect_from_ptr(ui: &mut egui::Ui, type_name: &str) {
    let job = layout_job(&[
        (FontId::monospace(14.0), type_name),
        (FontId::default(), " has no "),
        (FontId::monospace(14.0), "ReflectFromPtr"),
        (FontId::default(), " type data, so it cannot be displayed"),
    ]);

    ui.label(job);
}

fn error_message_entity_does_not_exist(ui: &mut egui::Ui, entity: Entity) {
    let job = layout_job(&[
        (FontId::default(), "Entity "),
        (FontId::monospace(14.0), &format!("{entity:?}")),
        (FontId::default(), " does not exist."),
    ]);

    ui.label(job);
}

mod guess_entity_name {
    use bevy_core::Name;
    use bevy_ecs::{prelude::*, world::EntityRef};

    /// Guesses an appropriate entity name like `Light (6)` or falls back to `Entity (8)`
    pub fn entity_name(world: &World, entity: Entity) -> String {
        match world.get_entity(entity) {
            Some(entity) => guess_entity_name_inner(entity),
            None => format!("Entity {} (inexistent)", entity.id()),
        }
    }

    fn guess_entity_name_inner(entity: EntityRef) -> String {
        if let Some(name) = entity.get::<Name>() {
            return name.as_str().to_string();
        }

        let id = entity.id().id();

        format!("Entity ({:?})", id)
    }
}
