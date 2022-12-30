use std::{borrow::Cow, ops::AddAssign, time::Instant};

use bevy_reflect::Reflect;
use egui::{DragValue, RichText};

use super::{change_slider, iter_all_eq, InspectorUi};
use crate::{inspector_options::std_options::NumberOptions, many_ui};
use std::{any::Any, time::Duration};

pub fn number_ui<T: egui::emath::Numeric>(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    options: &dyn Any,
    _: egui::Id,
    _: InspectorUi<'_, '_>,
) -> bool {
    let value = value.downcast_mut::<T>().unwrap();
    let options = options
        .downcast_ref::<NumberOptions<T>>()
        .cloned()
        .unwrap_or_default();
    display_number(value, &options, ui, 0.1)
}
pub fn number_ui_readonly<T: egui::emath::Numeric>(
    value: &dyn Any,
    ui: &mut egui::Ui,
    options: &dyn Any,
    _: egui::Id,
    _: InspectorUi<'_, '_>,
) {
    let value = value.downcast_ref::<T>().unwrap();
    let options = options
        .downcast_ref::<NumberOptions<T>>()
        .cloned()
        .unwrap_or_default();
    let decimal_range = 0..=1usize;
    ui.add(
        egui::Button::new(
            RichText::new(format!(
                "{}{}{}",
                options.prefix,
                egui::emath::format_with_decimals_in_range(value.to_f64(), decimal_range),
                options.suffix
            ))
            .monospace(),
        )
        .wrap(false)
        .sense(egui::Sense::hover()),
    );
}

fn display_number<T: egui::emath::Numeric>(
    value: &mut T,
    options: &NumberOptions<T>,
    ui: &mut egui::Ui,
    default_speed: f32,
) -> bool {
    let mut widget = egui::DragValue::new(value);
    if !options.prefix.is_empty() {
        widget = widget.prefix(&options.prefix);
    }
    if !options.suffix.is_empty() {
        widget = widget.suffix(&options.suffix);
    }
    match (options.min, options.max) {
        (Some(min), Some(max)) => widget = widget.clamp_range(min.to_f64()..=max.to_f64()),
        (Some(min), None) => widget = widget.clamp_range(min.to_f64()..=f64::MAX),
        (None, Some(max)) => widget = widget.clamp_range(f64::MIN..=max.to_f64()),
        (None, None) => {}
    }
    if options.speed != 0.0 {
        widget = widget.speed(options.speed);
    } else {
        widget = widget.speed(default_speed);
    }
    let mut changed = ui.add(widget).changed();
    if let Some(min) = options.min {
        let as_f64 = value.to_f64();
        let min = min.to_f64();
        if as_f64 < min {
            *value = T::from_f64(min);
            changed = true;
        }
    }
    if let Some(max) = options.max {
        let as_f64 = value.to_f64();
        let max = max.to_f64();
        if as_f64 > max {
            *value = T::from_f64(max);
            changed = true;
        }
    }
    changed
}

pub fn number_ui_many<T>(
    ui: &mut egui::Ui,
    _: &dyn Any,
    id: egui::Id,
    _env: InspectorUi<'_, '_>,
    values: &mut [&mut dyn Reflect],
    projector: &dyn Fn(&mut dyn Reflect) -> &mut dyn Reflect,
) -> bool
where
    T: Reflect + egui::emath::Numeric + AddAssign<T>,
{
    let same = iter_all_eq(
        values
            .iter_mut()
            .map(|value| *projector(*value).downcast_ref::<T>().unwrap()),
    )
    .map(T::to_f64);

    change_slider(ui, id, same, |change, overwrite| {
        for value in values.iter_mut() {
            let value = projector(*value).downcast_mut::<T>().unwrap();
            let change = T::from_f64(change);
            if overwrite {
                *value = change;
            } else {
                *value += change;
            }
        }
    })
}

pub fn bool_ui(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    _: egui::Id,
    _: InspectorUi<'_, '_>,
) -> bool {
    let value = value.downcast_mut::<bool>().unwrap();
    ui.checkbox(value, "").changed()
}
pub fn bool_ui_readonly(
    value: &dyn Any,
    ui: &mut egui::Ui,
    options: &dyn Any,
    id: egui::Id,
    env: InspectorUi<'_, '_>,
) {
    let mut copy = *value.downcast_ref::<bool>().unwrap();
    ui.add_enabled_ui(false, |ui| {
        bool_ui(&mut copy, ui, options, id, env);
    });
}

many_ui!(bool_ui_many bool_ui bool);

pub fn string_ui(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    _: egui::Id,
    _: InspectorUi<'_, '_>,
) -> bool {
    let value = value.downcast_mut::<String>().unwrap();
    if value.contains('\n') {
        ui.text_edit_multiline(value).changed()
    } else {
        ui.text_edit_singleline(value).changed()
    }
}

pub fn string_ui_readonly(
    value: &dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    _: egui::Id,
    _: InspectorUi<'_, '_>,
) {
    let value = value.downcast_ref::<String>().unwrap();
    if value.contains('\n') {
        ui.text_edit_multiline(&mut value.as_str());
    } else {
        ui.text_edit_singleline(&mut value.as_str());
    }
}

many_ui!(string_ui_many string_ui String);

pub fn cow_str_ui(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    _: egui::Id,
    _: InspectorUi<'_, '_>,
) -> bool {
    let value = value.downcast_mut::<Cow<str>>().unwrap();
    let mut clone = value.to_string();
    let changed = if value.contains('\n') {
        ui.text_edit_multiline(&mut clone).changed()
    } else {
        ui.text_edit_singleline(&mut clone).changed()
    };

    if changed {
        *value = Cow::Owned(clone);
    }

    changed
}

pub fn cow_str_ui_readonly(
    value: &dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    _: egui::Id,
    _: InspectorUi<'_, '_>,
) {
    let value = value.downcast_ref::<Cow<str>>().unwrap();
    if value.contains('\n') {
        ui.text_edit_multiline(&mut value.as_ref());
    } else {
        ui.text_edit_singleline(&mut value.as_ref());
    }
}

many_ui!(cow_str_ui_many cow_str_ui Cow<str>);

pub fn duration_ui(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    id: egui::Id,
    mut env: InspectorUi<'_, '_>,
) -> bool {
    let value = value.downcast_mut::<Duration>().unwrap();
    let mut seconds = value.as_secs_f64();
    let options = NumberOptions {
        min: Some(0.0f64),
        suffix: "s".to_string(),
        ..Default::default()
    };

    let changed = env.ui_for_reflect_with_options(&mut seconds, ui, id, &options);
    if changed {
        *value = Duration::from_secs_f64(seconds);
    }
    changed
}
pub fn duration_ui_readonly(
    value: &dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    id: egui::Id,
    mut env: InspectorUi<'_, '_>,
) {
    let value = value.downcast_ref::<Duration>().unwrap();
    let seconds = value.as_secs_f64();
    let options = NumberOptions {
        min: Some(0.0f64),
        suffix: "s".to_string(),
        ..Default::default()
    };
    env.ui_for_reflect_readonly_with_options(&seconds, ui, id, &options);
}

pub fn instant_ui(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    options: &dyn Any,
    id: egui::Id,
    env: InspectorUi<'_, '_>,
) -> bool {
    instant_ui_readonly(value, ui, options, id, env);
    false
}

pub fn instant_ui_readonly(
    value: &dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    _: egui::Id,
    _: InspectorUi<'_, '_>,
) {
    let value = value.downcast_ref::<Instant>().unwrap();
    let mut secs = value.elapsed().as_secs_f32();
    ui.horizontal(|ui| {
        ui.add_enabled(false, DragValue::new(&mut secs));
        ui.label("seconds ago");
    });
}
