use super::Context;
use crate::options::std_options::NumberOptions;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

type InspectorOverrideFn =
    Box<dyn Fn(&mut dyn Any, &mut egui::Ui, &dyn Any, &mut Context) -> bool + Send + Sync>;
pub struct InspectorEguiOverrides {
    fns: HashMap<TypeId, InspectorOverrideFn>,
}

impl Default for InspectorEguiOverrides {
    fn default() -> Self {
        let mut overrides = Self::empty();
        overrides.register::<f32, _>(num_override_slow);
        overrides.register::<f64, _>(num_override_slow);
        overrides.register::<i8, _>(num_override);
        overrides.register::<i16, _>(num_override);
        overrides.register::<i32, _>(num_override);
        overrides.register::<i64, _>(num_override);
        overrides.register::<isize, _>(num_override);
        overrides.register::<u8, _>(num_override);
        overrides.register::<u16, _>(num_override);
        overrides.register::<u32, _>(num_override);
        overrides.register::<u64, _>(num_override);
        overrides.register::<usize, _>(num_override);

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
        F: Fn(&mut T, &mut egui::Ui, &dyn Any, &mut Context) -> bool + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let callback = Box::new(
            move |value: &mut dyn Any,
                  ui: &mut egui::Ui,
                  options: &dyn Any,
                  context: &mut Context| {
                let value: &mut T = value.downcast_mut().unwrap();
                f(value, ui, options, context)
            },
        ) as InspectorOverrideFn;
        self.fns.insert(type_id, callback);
    }

    pub(crate) fn try_execute_mut(
        &self,
        value: &mut dyn Any,
        ui: &mut egui::Ui,
        options: &dyn Any,
        context: &mut Context,
    ) -> Option<bool> {
        let type_id = Any::type_id(value);
        self.fns
            .get(&type_id)
            .map(|f| f(value, ui, options, context))
    }
}

fn num_override<T: egui::emath::Numeric>(
    value: &mut T,
    ui: &mut egui::Ui,
    options: &dyn Any,
    _: &mut Context,
) -> bool {
    let options = options
        .downcast_ref::<NumberOptions<T>>()
        .cloned()
        .unwrap_or_default();
    num_ui(value, &options, ui, 1.0)
}
fn num_override_slow<T: egui::emath::Numeric>(
    value: &mut T,
    ui: &mut egui::Ui,
    options: &dyn Any,
    _: &mut Context,
) -> bool {
    let options = options
        .downcast_ref::<NumberOptions<T>>()
        .cloned()
        .unwrap_or_default();
    num_ui(value, &options, ui, 0.1)
}

fn num_ui<T: egui::emath::Numeric>(
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
