use crate::{utils, Inspectable};
use bevy::prelude::*;

use super::{WorldInspectorParams, WorldUIContext};

impl Inspectable for World {
    type Attributes = WorldInspectorParams;

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        options: Self::Attributes,
        context: &crate::Context,
    ) {
        let resources = expect_context!(ui, context.resources, "World");

        let ui_ctx = WorldUIContext::new(self, resources);
        ui_ctx.ui(ui, &options);
    }
}

impl Inspectable for Entity {
    type Attributes = WorldInspectorParams;

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        options: Self::Attributes,
        context: &crate::Context,
    ) {
        let resources = expect_context!(ui, context.resources, "Entity");
        let world = expect_context!(ui, context.world, "Entity");

        let ui_ctx = WorldUIContext::new(world, resources);
        ui_ctx.entity_ui(ui, *self, &options);
    }
}
