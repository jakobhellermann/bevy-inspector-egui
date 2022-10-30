use std::{any::TypeId, borrow::Cow};

use bevy_asset::HandleId;
use bevy_ecs::entity::Entity;
use bevy_reflect::TypeRegistry;
use egui::FontId;

use crate::egui_utils::layout_job;

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

pub fn resource_does_not_exist(ui: &mut egui::Ui, name: &str) {
    let job = layout_job(&[
        (FontId::default(), "Resource "),
        (FontId::monospace(14.0), name),
        (FontId::default(), " does not exist in the world."),
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
