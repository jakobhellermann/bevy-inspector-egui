use std::{any::TypeId, sync::Arc};

use bevy_app::prelude::AppTypeRegistry;
use bevy_ecs::prelude::*;
use bevy_reflect::ReflectFromPtr;

use crate::driver_egui::{Context, InspectorEguiOverrides, InspectorUi};

#[derive(Resource, Default, Clone)]
pub struct AppInspectorEguiOverrides(pub Arc<InspectorEguiOverrides>);

pub fn ui_for_resource(world: &mut World, resource_type_id: TypeId, ui: &mut egui::Ui) {
    world.init_resource::<AppInspectorEguiOverrides>();

    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    let egui_overrides = world
        .get_resource_or_insert_with(AppInspectorEguiOverrides::default)
        .clone();

    let mut cx = Context;
    let mut env = InspectorUi::new(&type_registry, &egui_overrides.0, &mut cx);

    let component_id = world.components().get_id(resource_type_id).unwrap();
    let value = world.get_resource_mut_by_id(component_id).unwrap();
    let reflect_from_ptr = type_registry
        .get_type_data::<ReflectFromPtr>(resource_type_id)
        .unwrap();
    assert_eq!(reflect_from_ptr.type_id(), resource_type_id);
    let value = unsafe { reflect_from_ptr.as_reflect_ptr_mut(value.into_inner()) };
    env.ui_for_reflect(value, ui, egui::Id::new(resource_type_id));
}
