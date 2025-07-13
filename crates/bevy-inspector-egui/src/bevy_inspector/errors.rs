use std::{any::TypeId, borrow::Cow};

use bevy_asset::UntypedAssetId;
use bevy_ecs::entity::Entity;
use bevy_reflect::TypeRegistry;
use egui::FontId;

use crate::{egui_utils::layout_job, restricted_world_view::Error};

pub fn show_error(error: Error, ui: &mut egui::Ui, name_of_type: &str) {
    match error {
        Error::NoAccessToResource(_) => no_access_resource(ui, name_of_type),
        Error::NoAccessToComponent((entity, _)) => no_access_component(ui, entity, name_of_type),
        Error::ComponentDoesNotExist((entity, _)) => {
            component_does_not_exist(ui, entity, name_of_type)
        }
        Error::ResourceDoesNotExist(_) => resource_does_not_exist(ui, name_of_type),
        Error::NoComponentId(_) => no_component_id(ui, name_of_type),
        Error::NoTypeRegistration(_) => {
            crate::reflect_inspector::errors::not_in_type_registry(ui, name_of_type)
        }
        Error::NoTypeData(_, data) => no_type_data(ui, name_of_type, data),
    }
}

pub fn no_access_resource(ui: &mut egui::Ui, type_name: &str) {
    let job = layout_job(&[
        (FontId::proportional(13.0), "No access to resource "),
        (FontId::monospace(12.0), type_name),
        (FontId::proportional(13.0), "."),
    ]);

    ui.label(job);
}
pub fn no_access_component(ui: &mut egui::Ui, entity: Entity, type_name: &str) {
    let job = layout_job(&[
        (FontId::proportional(13.0), "No access to component "),
        (FontId::monospace(12.0), type_name),
        (FontId::proportional(13.0), " on entity "),
        (FontId::monospace(12.0), &format!("{entity:?}")),
        (FontId::proportional(13.0), "."),
    ]);

    ui.label(job);
}

pub fn resource_does_not_exist(ui: &mut egui::Ui, name: &str) {
    let job = layout_job(&[
        (FontId::proportional(13.0), "Resource "),
        (FontId::monospace(12.0), name),
        (FontId::proportional(13.0), " does not exist in the world."),
    ]);

    ui.label(job);
}

pub fn component_does_not_exist(ui: &mut egui::Ui, entity: Entity, name: &str) {
    let job = layout_job(&[
        (FontId::proportional(13.0), "Component "),
        (FontId::monospace(12.0), name),
        (FontId::proportional(13.0), " does not exist on entity "),
        (FontId::monospace(12.0), &format!("{entity:?}")),
        (FontId::proportional(13.0), "."),
    ]);

    ui.label(job);
}

pub fn no_component_id(ui: &mut egui::Ui, type_name: &str) {
    let job = layout_job(&[
        (FontId::monospace(12.0), type_name),
        (FontId::proportional(13.0), " has no associated "),
        (FontId::monospace(12.0), "ComponentId"),
        (FontId::proportional(13.0), "."),
    ]);

    ui.label(job);
}

pub fn no_type_data(ui: &mut egui::Ui, type_name: &str, type_data: &str) {
    let job = layout_job(&[
        (FontId::monospace(12.0), type_name),
        (FontId::proportional(13.0), " has no "),
        (FontId::monospace(12.0), type_data),
        (
            FontId::proportional(13.0),
            " type data, so it cannot be displayed",
        ),
    ]);

    ui.label(job);
}

pub fn entity_does_not_exist(ui: &mut egui::Ui, entity: Entity) {
    let job = layout_job(&[
        (FontId::proportional(13.0), "Entity "),
        (FontId::monospace(12.0), &format!("{entity:?}")),
        (FontId::proportional(13.0), " does not exist."),
    ]);

    ui.label(job);
}

pub fn no_world_in_context(ui: &mut egui::Ui, type_name: &str) {
    let job = layout_job(&[
        (FontId::monospace(12.0), type_name),
        (FontId::proportional(13.0), " needs the bevy world in the "),
        (FontId::monospace(12.0), "InspectorUi"),
        (
            FontId::proportional(13.0),
            " context to provide meaningful information.",
        ),
    ]);

    ui.label(job);
}

pub fn dead_asset_handle(ui: &mut egui::Ui, handle: UntypedAssetId) {
    let job = layout_job(&[
        (FontId::proportional(13.0), "Handle "),
        (FontId::monospace(12.0), &format!("{handle:?}")),
        (FontId::proportional(13.0), " points to no asset."),
    ]);

    ui.label(job);
}

pub fn state_does_not_exist(ui: &mut egui::Ui, name: &str) {
    let job = layout_job(&[
        (FontId::proportional(13.0), "State "),
        (FontId::monospace(12.0), name),
        (
            FontId::proportional(13.0),
            " does not exist. Did you forget to call ",
        ),
        (
            FontId::monospace(12.0),
            &format!(".add_state::<{name}>(..)"),
        ),
        (FontId::proportional(13.0), "?"),
    ]);

    ui.label(job);
}

pub fn no_type_id(ui: &mut egui::Ui, component_name: &str) {
    let job = layout_job(&[
        (FontId::monospace(12.0), component_name),
        (
            FontId::proportional(13.0),
            " is not backed by a rust type, so it cannot be displayed.",
        ),
    ]);

    ui.label(job);
}

pub fn name_of_type(type_id: TypeId, type_registry: &TypeRegistry) -> Cow<'_, str> {
    type_registry
        .get(type_id)
        .map(|registration| Cow::Borrowed(registration.type_info().type_path_table().short_path()))
        .unwrap_or_else(|| Cow::Owned(format!("{type_id:?}")))
}
