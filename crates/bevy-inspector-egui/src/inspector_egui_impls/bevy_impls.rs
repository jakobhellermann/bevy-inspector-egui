use std::any::{Any, TypeId};

use bevy_asset::{AssetServer, HandleId};
use bevy_render::color::Color;
use egui::{ecolor::Hsva, Color32};

use crate::{bevy_inspector::errors::show_error, egui_reflect_inspector::InspectorUi, many_ui};

pub fn handle_id_ui(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    options: &dyn Any,
    env: InspectorUi<'_, '_>,
) -> bool {
    handle_id_ui_readonly(value, ui, options, env);
    false
}

pub fn handle_id_ui_readonly(
    value: &dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    env: InspectorUi<'_, '_>,
) {
    let handle = *value.downcast_ref::<HandleId>().unwrap();

    if let Some(world) = &mut env.context.world {
        if world.allows_access_to_resource(TypeId::of::<AssetServer>()) {
            let asset_server = match world.get_resource_mut::<AssetServer>() {
                Ok(asset_server) => asset_server,
                Err(error) => {
                    show_error(error, ui, "AssetServer");
                    return;
                }
            };

            if let Some(path) = asset_server.get_handle_path(handle) {
                ui.label(format!("{:?}", path));
                return;
            }
        }
    }

    ui.label(format!("{:?}", handle));
}

pub fn color_ui(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    _: InspectorUi<'_, '_>,
) -> bool {
    let value = value.downcast_mut::<Color>().unwrap();

    color_ui_inner(value, ui)
}

pub fn color_ui_readonly(value: &dyn Any, ui: &mut egui::Ui, _: &dyn Any, _: InspectorUi<'_, '_>) {
    let value = value.downcast_ref::<Color>().unwrap();

    ui.add_enabled_ui(false, |ui| {
        let mut color = *value;
        color_ui_inner(&mut color, ui);
    });
}

many_ui!(color_ui_many color_ui Color);

fn color_ui_inner(value: &mut Color, ui: &mut egui::Ui) -> bool {
    match value {
        Color::Rgba {
            red,
            green,
            blue,
            alpha,
        } => {
            let mut color = Color32::from_rgba_premultiplied(
                (*red * 255.) as u8,
                (*green * 255.) as u8,
                (*blue * 255.) as u8,
                (*alpha * 255.) as u8,
            );
            if ui.color_edit_button_srgba(&mut color).changed() {
                let [r, g, b, a] = color.to_array();
                *red = r as f32 / 255.;
                *green = g as f32 / 255.;
                *blue = b as f32 / 255.;
                *alpha = a as f32 / 255.;
                return true;
            }
        }
        Color::RgbaLinear {
            red,
            green,
            blue,
            alpha,
        } => {
            let mut color = [*red, *green, *blue, *alpha];
            if ui
                .color_edit_button_rgba_premultiplied(&mut color)
                .changed()
            {
                *red = color[0];
                *green = color[1];
                *blue = color[2];
                *alpha = color[3];
                return true;
            }
        }
        Color::Hsla {
            hue,
            saturation,
            lightness,
            alpha,
        } => {
            let mut hsva = Hsva::new(*hue, *saturation, *lightness, *alpha);
            if ui.color_edit_button_hsva(&mut hsva).changed() {
                *hue = hsva.h;
                *saturation = hsva.s;
                *lightness = hsva.v;
                *alpha = hsva.a;
                return true;
            }
        }
    }
    false
}
