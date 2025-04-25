use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::inspector_egui_impls::{InspectorEguiImpl, InspectorPrimitive};
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_inspector_egui::reflect_inspector::InspectorUi;

#[derive(Resource, Reflect, Default)]
struct Config {
    some_config_option: ToggleOption,
}

#[derive(Reflect, Default)]
struct ToggleOption(bool);

impl InspectorPrimitive for ToggleOption {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _: &dyn std::any::Any,
        _: egui::Id,
        _: InspectorUi<'_, '_>,
    ) -> bool {
        let mut changed = ui.radio_value(&mut self.0, false, "Disabled").changed();
        changed |= ui.radio_value(&mut self.0, true, "Enabled").changed();
        changed
    }

    fn ui_readonly(
        &self,
        ui: &mut egui::Ui,
        _: &dyn std::any::Any,
        _: egui::Id,
        _: InspectorUi<'_, '_>,
    ) {
        let mut copy = self.0;
        ui.add_enabled_ui(false, |ui| {
            ui.radio_value(&mut copy, false, "Disabled").changed();
            ui.radio_value(&mut copy, true, "Enabled").changed();
        });
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(ResourceInspectorPlugin::<Config>::new())
        .init_resource::<Config>()
        .register_type::<ToggleOption>()
        .register_type_data::<ToggleOption, InspectorEguiImpl>()
        .run();
}
