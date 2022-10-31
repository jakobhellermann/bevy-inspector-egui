use std::{any::TypeId, borrow::Cow};

use bevy_asset::HandleId;
use bevy_ecs::entity::Entity;
use bevy_reflect::TypeRegistry;
use egui::FontId;

use crate::{egui_utils::layout_job, restricted_world_view::Error};

pub fn show_error(error: Error, ui: &mut egui::Ui, type_registry: &TypeRegistry) {
    match error {
        Error::NoAccessToResource(type_id) => {
            no_access_resource(ui, &name_of_type(type_id, type_registry))
        }
        Error::NoAccessToComponent((entity, component)) => {
            no_access_component(ui, entity, &name_of_type(component, type_registry))
        }
        Error::ComponentDoesNotExist((entity, component)) => {
            component_does_not_exist(ui, entity, &name_of_type(component, type_registry))
        }
        Error::ResourceDoesNotExist(type_id) => {
            resource_does_not_exist(ui, &name_of_type(type_id, type_registry))
        }
        Error::NoComponentId(type_id) => no_component_id(ui, &name_of_type(type_id, type_registry)),
        Error::NoTypeRegistration(type_id) => {
            not_in_type_registry(ui, &name_of_type(type_id, type_registry))
        }
        Error::NoTypeData(type_id, data) => {
            no_type_data(ui, &name_of_type(type_id, type_registry), data)
        }
    }
}

pub fn no_access_resource(ui: &mut egui::Ui, type_name: &str) {
    let job = layout_job(&[
        (FontId::default(), "No access to resource "),
        (FontId::monospace(14.0), type_name),
        (FontId::default(), "."),
    ]);

    ui.label(job);
}
pub fn no_access_component(ui: &mut egui::Ui, entity: Entity, type_name: &str) {
    let job = layout_job(&[
        (FontId::default(), "No access to component "),
        (FontId::monospace(14.0), type_name),
        (FontId::default(), " on entity "),
        (FontId::monospace(14.0), &format!("{entity:?}")),
        (FontId::default(), "."),
    ]);

    ui.label(job);
}

pub fn resource_does_not_exist(ui: &mut egui::Ui, name: &str) {
    let job = layout_job(&[
        (FontId::default(), "Resource "),
        (FontId::monospace(14.0), name),
        (FontId::default(), " does not exist in the world."),
    ]);

    ui.label(job);
}

pub fn component_does_not_exist(ui: &mut egui::Ui, entity: Entity, name: &str) {
    let job = layout_job(&[
        (FontId::default(), "Component "),
        (FontId::monospace(14.0), name),
        (FontId::default(), " does not exist on entity "),
        (FontId::monospace(14.0), &format!("{entity:?}")),
        (FontId::default(), "."),
    ]);

    ui.label(job);
}

pub fn no_component_id(ui: &mut egui::Ui, type_name: &str) {
    let job = layout_job(&[
        (FontId::monospace(14.0), type_name),
        (FontId::default(), " has no associated "),
        (FontId::monospace(14.0), "ComponentId"),
        (FontId::default(), "."),
    ]);

    ui.label(job);
}

pub fn no_type_data(ui: &mut egui::Ui, type_name: &str, type_data: &str) {
    let job = layout_job(&[
        (FontId::monospace(14.0), type_name),
        (FontId::default(), " has no "),
        (FontId::monospace(14.0), type_data),
        (FontId::default(), " type data, so it cannot be displayed"),
    ]);

    ui.label(job);
}

pub fn entity_does_not_exist(ui: &mut egui::Ui, entity: Entity) {
    let job = layout_job(&[
        (FontId::default(), "Entity "),
        (FontId::monospace(14.0), &format!("{entity:?}")),
        (FontId::default(), " does not exist."),
    ]);

    ui.label(job);
}

pub fn no_world_in_context(ui: &mut egui::Ui, type_name: &str) {
    let job = layout_job(&[
        (FontId::monospace(14.0), type_name),
        (FontId::default(), " needs the bevy world in the "),
        (FontId::monospace(14.0), "InspectorUi"),
        (
            FontId::default(),
            " context to provide meaningful information.",
        ),
    ]);

    ui.label(job);
}

pub fn dead_asset_handle(ui: &mut egui::Ui, handle: HandleId) {
    let job = layout_job(&[
        (FontId::default(), "Handle "),
        (FontId::monospace(14.0), &format!("{:?}", handle)),
        (FontId::default(), " points to no asset."),
    ]);

    ui.label(job);
}

pub fn not_in_type_registry(ui: &mut egui::Ui, type_name: &str) {
    let job = layout_job(&[
        (FontId::monospace(14.0), type_name),
        (FontId::default(), " is not registered in the "),
        (FontId::monospace(14.0), "TypeRegistry"),
    ]);

    ui.label(job);
}

pub fn name_of_type(type_id: TypeId, type_registry: &TypeRegistry) -> Cow<str> {
    type_registry
        .get(type_id)
        .map(|registration| Cow::Borrowed(registration.short_name()))
        .unwrap_or_else(|| Cow::Owned(format!("{type_id:?}")))
}
