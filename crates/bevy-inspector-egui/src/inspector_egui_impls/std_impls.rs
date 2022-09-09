use std::time::Instant;

use super::InspectorUi;
use crate::inspector_options::std_options::NumberOptions;
use std::{any::Any, time::Duration};

pub fn number_ui<T: egui::emath::Numeric>(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    options: &dyn Any,
    _: InspectorUi<'_, '_>,
) -> bool {
    let value = value.downcast_mut::<T>().unwrap();
    let options = options
        .downcast_ref::<NumberOptions<T>>()
        .cloned()
        .unwrap_or_default();
    display_number(value, &options, ui, 1.0)
}
pub fn number_ui_subint<T: egui::emath::Numeric>(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    options: &dyn Any,
    _: InspectorUi<'_, '_>,
) -> bool {
    let value = value.downcast_mut::<T>().unwrap();
    let options = options
        .downcast_ref::<NumberOptions<T>>()
        .cloned()
        .unwrap_or_default();
    display_number(value, &options, ui, 0.1)
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

pub fn bool_ui(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    _: InspectorUi<'_, '_>,
) -> bool {
    let value = value.downcast_mut::<bool>().unwrap();
    ui.checkbox(value, "").changed()
}

pub fn duration_ui(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    mut env: InspectorUi<'_, '_>,
) -> bool {
    let value = value.downcast_mut::<Duration>().unwrap();
    let mut seconds = value.as_secs_f64();
    let options = NumberOptions {
        min: Some(0.0f64),
        suffix: "s".to_string(),
        ..Default::default()
    };

    let changed = env.ui_for_reflect_with_options(&mut seconds, ui, egui::Id::new(0), &options);
    if changed {
        *value = Duration::from_secs_f64(seconds);
    }
    changed
}

pub fn instant_ui(
    value: &mut dyn Any,
    ui: &mut egui::Ui,
    _: &dyn Any,
    _: InspectorUi<'_, '_>,
) -> bool {
    let value = value.downcast_mut::<Instant>().unwrap();
    ui.label(format!("{} seconds ago", value.elapsed().as_secs_f32()));
    false
}
