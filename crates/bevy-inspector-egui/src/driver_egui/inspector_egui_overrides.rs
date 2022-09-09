use bevy_reflect::TypeRegistry;
use std::time::Instant;

use super::{Context, InspectorUi};
use crate::options::std_options::NumberOptions;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    time::Duration,
};

type InspectorOverrideFn =
    Box<dyn Fn(&mut dyn Any, &mut egui::Ui, &dyn Any, InspectorUi<'_, '_>) -> bool + Send + Sync>;

pub struct InspectorEguiOverrides {
    fns: HashMap<TypeId, InspectorOverrideFn>,
}

impl Default for InspectorEguiOverrides {
    fn default() -> Self {
        let mut overrides = Self::empty();
        overrides.register::<f32, _>(number_ui_subint);
        overrides.register::<f64, _>(number_ui_subint);
        overrides.register::<i8, _>(number_ui);
        overrides.register::<i16, _>(number_ui);
        overrides.register::<i32, _>(number_ui);
        overrides.register::<i64, _>(number_ui);
        overrides.register::<isize, _>(number_ui);
        overrides.register::<u8, _>(number_ui);
        overrides.register::<u16, _>(number_ui);
        overrides.register::<u32, _>(number_ui);
        overrides.register::<u64, _>(number_ui);
        overrides.register::<usize, _>(number_ui);
        overrides.register::<bool, _>(bool_ui);

        overrides.register::<Duration, _>(duration);
        overrides.register::<Instant, _>(instant);

        overrides
    }
}

impl InspectorEguiOverrides {
    fn empty() -> InspectorEguiOverrides {
        InspectorEguiOverrides {
            fns: HashMap::new(),
        }
    }

    pub fn register<T: 'static, F>(&mut self, f: F)
    where
        F: Fn(&mut T, &mut egui::Ui, &dyn Any, InspectorUi<'_, '_>) -> bool + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let callback = Box::new(
            move |value: &mut dyn Any,
                  ui: &mut egui::Ui,
                  options: &dyn Any,
                  env: InspectorUi<'_, '_>| {
                let value: &mut T = value.downcast_mut().unwrap();
                f(value, ui, options, env)
            },
        ) as InspectorOverrideFn;
        self.fns.insert(type_id, callback);
    }

    pub(crate) fn try_execute_mut<'a, 'c: 'a>(
        &'a self,
        value: &mut dyn Any,
        ui: &mut egui::Ui,
        options: &dyn Any,
        context: &'a mut Context<'c>,
        type_registry: &'a TypeRegistry,
    ) -> Option<bool> {
        let type_id = Any::type_id(value);
        self.fns.get(&type_id).map(|f| {
            f(
                value,
                ui,
                options,
                InspectorUi::new(type_registry, self, context),
            )
        })
    }
}

fn number_ui<T: egui::emath::Numeric>(
    value: &mut T,
    ui: &mut egui::Ui,
    options: &dyn Any,
    _: InspectorUi<'_, '_>,
) -> bool {
    let options = options
        .downcast_ref::<NumberOptions<T>>()
        .cloned()
        .unwrap_or_default();
    display_number(value, &options, ui, 1.0)
}
fn number_ui_subint<T: egui::emath::Numeric>(
    value: &mut T,
    ui: &mut egui::Ui,
    options: &dyn Any,
    _: InspectorUi<'_, '_>,
) -> bool {
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

fn bool_ui(value: &mut bool, ui: &mut egui::Ui, _: &dyn Any, _: InspectorUi<'_, '_>) -> bool {
    ui.checkbox(value, "").changed()
}

fn duration(
    value: &mut Duration,
    ui: &mut egui::Ui,
    _: &dyn Any,
    mut env: InspectorUi<'_, '_>,
) -> bool {
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
fn instant(value: &mut Instant, ui: &mut egui::Ui, _: &dyn Any, _: InspectorUi<'_, '_>) -> bool {
    ui.label(format!("{} seconds ago", value.elapsed().as_secs_f32()));
    false
}
