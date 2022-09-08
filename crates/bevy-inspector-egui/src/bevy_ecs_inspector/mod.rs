use std::{any::TypeId, sync::Arc};

use bevy_app::prelude::AppTypeRegistry;
use bevy_ecs::prelude::*;
use bevy_reflect::ReflectFromPtr;

use crate::driver_egui::{split_world_permission, Context, InspectorEguiOverrides, InspectorUi};

#[derive(Resource, Default, Clone)]
pub struct AppInspectorEguiOverrides(pub Arc<InspectorEguiOverrides>);

pub fn ui_for_resource(world: &mut World, resource_type_id: TypeId, ui: &mut egui::Ui) {
    world.init_resource::<AppInspectorEguiOverrides>();

    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    let egui_overrides = world
        .get_resource_or_insert_with(AppInspectorEguiOverrides::default)
        .clone();

    let (no_resource_refs_world, only_resource_access_world) =
        split_world_permission(world, resource_type_id);

    let mut cx = Context {
        world: Some(only_resource_access_world),
    };
    let mut env = InspectorUi::new(&type_registry, &egui_overrides.0, &mut cx);

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
