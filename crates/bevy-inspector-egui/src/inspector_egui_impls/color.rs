use std::any::Any;

use bevy_render::color::Color;
use egui::{color::Hsva, Color32};

use crate::egui_reflect_inspector::InspectorUi;

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
