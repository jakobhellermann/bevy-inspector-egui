use bevy_asset::HandleId;
use bevy_ecs::entity::Entity;
use egui::FontId;

use crate::egui_utils::layout_job;

pub fn error_message_no_reflect_from_ptr(ui: &mut egui::Ui, type_name: &str) {
    let job = layout_job(&[
        (FontId::monospace(14.0), type_name),
        (FontId::default(), " has no "),
        (FontId::monospace(14.0), "ReflectFromPtr"),
        (FontId::default(), " type data, so it cannot be displayed"),
    ]);

    ui.label(job);
}

pub fn error_message_entity_does_not_exist(ui: &mut egui::Ui, entity: Entity) {
    let job = layout_job(&[
        (FontId::default(), "Entity "),
        (FontId::monospace(14.0), &format!("{entity:?}")),
        (FontId::default(), " does not exist."),
    ]);

    ui.label(job);
}

pub fn error_message_no_world_in_context(ui: &mut egui::Ui, type_name: &str) {
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

pub fn error_message_dead_asset_handle(ui: &mut egui::Ui, handle: HandleId) {
    let job = layout_job(&[
        (FontId::default(), "Handle "),
        (FontId::monospace(14.0), &format!("{:?}", handle)),
        (FontId::default(), " points to no asset."),
    ]);

    ui.label(job);
}
