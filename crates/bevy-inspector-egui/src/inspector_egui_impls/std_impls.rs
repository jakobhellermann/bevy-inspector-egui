use std::{borrow::Cow, ops::AddAssign, path::PathBuf};

use bevy_platform::time::Instant;
use bevy_reflect::{PartialReflect, Reflect, TypePath};
use egui::{DragValue, RichText, TextBuffer};

use super::{change_slider, iter_all_eq, InspectorPrimitive, InspectorUi};
use crate::{
    inspector_options::{
        std_options::{NumberDisplay, NumberOptions, RangeOptions},
        InspectorOptionsType,
    },
    reflect_inspector::ProjectorReflect,
};
use std::{any::Any, time::Duration};

// just for orphan rules
trait Num: egui::emath::Numeric {}

macro_rules! impl_num {
    ($($ty:ty),*) => {
        $(
            impl Num for $ty {}
        )*
    };
}

impl_num!(f32, f64, i8, u8, i16, u16, i32, u32, i64, u64, isize, usize);

impl<T: Reflect + Num> InspectorPrimitive for T {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        options: &dyn Any,
        _: egui::Id,
        _: InspectorUi<'_, '_>,
    ) -> bool {
        let options = options
            .downcast_ref::<NumberOptions<T>>()
            .cloned()
            .unwrap_or_default();
        display_number(self, &options, ui, 0.1)
    }

    fn ui_readonly(
        &self,
        ui: &mut egui::Ui,
        options: &dyn Any,
        _: egui::Id,
        _: InspectorUi<'_, '_>,
    ) {
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
                    egui::emath::format_with_decimals_in_range(self.to_f64(), decimal_range),
                    options.suffix
                ))
                .monospace(),
            )
            .truncate()
            .sense(egui::Sense::hover()),
        );
    }
}

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
        .truncate()
        .sense(egui::Sense::hover()),
    );
}

fn display_number<T: egui::emath::Numeric>(
    value: &mut T,
    options: &NumberOptions<T>,
    ui: &mut egui::Ui,
    default_speed: f32,
) -> bool {
    let mut changed = match options.display {
        NumberDisplay::Drag => {
            let mut widget = egui::DragValue::new(value);
            if !options.prefix.is_empty() {
                widget = widget.prefix(&options.prefix);
            }
            if !options.suffix.is_empty() {
                widget = widget.suffix(&options.suffix);
            }
            match (options.min, options.max) {
                (Some(min), Some(max)) => widget = widget.range(min.to_f64()..=max.to_f64()),
                (Some(min), None) => widget = widget.range(min.to_f64()..=f64::MAX),
                (None, Some(max)) => widget = widget.range(f64::MIN..=max.to_f64()),
                (None, None) => {}
            }
            if options.speed != 0.0 {
                widget = widget.speed(options.speed);
            } else {
                widget = widget.speed(default_speed);
            }
            ui.add(widget).changed()
        }
        NumberDisplay::Slider => {
            let min = options.min.unwrap_or_else(|| T::from_f64(0.0));
            let max = options.max.unwrap_or_else(|| T::from_f64(1.0));
            let range = min..=max;
            let widget = egui::Slider::new(value, range);
            ui.add(widget).changed()
        }
    };

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
    values: &mut [&mut dyn PartialReflect],
    projector: &dyn ProjectorReflect,
) -> bool
where
    T: Reflect + egui::emath::Numeric + AddAssign<T>,
{
    let same = iter_all_eq(
        values
            .iter_mut()
            .map(|value| *projector(*value).try_downcast_ref::<T>().unwrap()),
    )
    .map(T::to_f64);

    change_slider(ui, id, same, |change, overwrite| {
        for value in values.iter_mut() {
            let value = projector(*value)
                .try_downcast_mut::<T>()
                .expect("non-fully-reflected value passed to number_ui_many");
            let change = T::from_f64(change);
            if overwrite {
                *value = change;
            } else {
                *value += change;
            }
        }
    })
}

impl InspectorPrimitive for bool {
    fn ui(&mut self, ui: &mut egui::Ui, _: &dyn Any, _: egui::Id, _: InspectorUi<'_, '_>) -> bool {
        ui.checkbox(self, "").changed()
    }

    fn ui_readonly(
        &self,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) {
        let mut copy = *self;
        ui.add_enabled_ui(false, |ui| {
            copy.ui(ui, options, id, env);
        });
    }
}

impl InspectorPrimitive for String {
    fn ui(&mut self, ui: &mut egui::Ui, _: &dyn Any, _: egui::Id, _: InspectorUi<'_, '_>) -> bool {
        if self.contains('\n') {
            ui.text_edit_multiline(self).changed()
        } else {
            ui.text_edit_singleline(self).changed()
        }
    }

    fn ui_readonly(&self, ui: &mut egui::Ui, _: &dyn Any, _: egui::Id, _: InspectorUi<'_, '_>) {
        if self.contains('\n') {
            ui.text_edit_multiline(&mut self.as_str());
        } else {
            ui.text_edit_singleline(&mut self.as_str());
        }
    }
}

impl InspectorPrimitive for Cow<'static, str> {
    fn ui(&mut self, ui: &mut egui::Ui, _: &dyn Any, _: egui::Id, _: InspectorUi<'_, '_>) -> bool {
        let mut clone = self.to_string();
        let changed = if self.contains('\n') {
            ui.text_edit_multiline(&mut clone).changed()
        } else {
            ui.text_edit_singleline(&mut clone).changed()
        };

        if changed {
            *self = Cow::Owned(clone);
        }

        changed
    }

    fn ui_readonly(&self, ui: &mut egui::Ui, _: &dyn Any, _: egui::Id, _: InspectorUi<'_, '_>) {
        if self.contains('\n') {
            ui.text_edit_multiline(&mut self.as_str());
        } else {
            ui.text_edit_singleline(&mut self.as_str());
        }
    }
}

impl InspectorPrimitive for Duration {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _: &dyn Any,
        id: egui::Id,
        mut env: InspectorUi<'_, '_>,
    ) -> bool {
        let mut seconds = self.as_secs_f64();
        let options = NumberOptions {
            min: Some(0.0f64),
            suffix: "s".to_string(),
            ..Default::default()
        };

        let changed = env.ui_for_reflect_with_options(&mut seconds, ui, id, &options);
        if changed {
            *self = Duration::from_secs_f64(seconds);
        }
        changed
    }

    fn ui_readonly(
        &self,
        ui: &mut egui::Ui,
        _: &dyn Any,
        id: egui::Id,
        mut env: InspectorUi<'_, '_>,
    ) {
        let seconds = self.as_secs_f64();
        let options = NumberOptions {
            min: Some(0.0f64),
            suffix: "s".to_string(),
            ..Default::default()
        };
        env.ui_for_reflect_readonly_with_options(&seconds, ui, id, &options);
    }
}

impl InspectorPrimitive for Instant {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) -> bool {
        self.ui_readonly(ui, options, id, env);
        false
    }

    fn ui_readonly(&self, ui: &mut egui::Ui, _: &dyn Any, _: egui::Id, _: InspectorUi<'_, '_>) {
        let mut secs = self.elapsed().as_secs_f32();
        ui.horizontal(|ui| {
            ui.add_enabled(false, DragValue::new(&mut secs));
            ui.label("seconds ago");
        });
    }
}

impl<T: Reflect + TypePath + egui::emath::Numeric + InspectorOptionsType> InspectorPrimitive
    for std::ops::Range<T>
{
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) -> bool {
        let std::ops::Range { start, end } = self;
        display_range::<T>(ui, options, id, env, "..", Some(start), Some(end))
    }

    fn ui_readonly(
        &self,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) {
        let std::ops::Range { start, end } = self;
        display_range_readonly::<T>(ui, options, id, env, "..", Some(start), Some(end));
    }
}
fn display_range<T: egui::emath::Numeric + InspectorOptionsType>(
    ui: &mut egui::Ui,
    options: &dyn Any,
    id: egui::Id,
    mut env: InspectorUi<'_, '_>,

    // this is made to be generic but I'm currently just using it for a..b, not a..=b, ..a, a.., .., etc., because these types don't hand out mutable references
    symbol: &'static str,
    start: Option<&mut T>,
    end: Option<&mut T>,
) -> bool {
    let options = options.downcast_ref::<RangeOptions<T>>();

    let start_options = options.map(|a| &a.start as &dyn Any).unwrap_or(&());
    let end_options = options.map(|a| &a.end as &dyn Any).unwrap_or(&());

    let mut changed = false;
    ui.horizontal(|ui| {
        if let Some(start) = start {
            changed |= number_ui::<T>(start, ui, start_options, id, env.reborrow());
        }
        ui.label(symbol);
        if let Some(end) = end {
            changed |= number_ui::<T>(end, ui, end_options, id, env.reborrow());
        }
    });

    changed
}

fn display_range_readonly<T: egui::emath::Numeric + InspectorOptionsType>(
    ui: &mut egui::Ui,
    options: &dyn Any,
    id: egui::Id,
    mut env: InspectorUi<'_, '_>,

    symbol: &'static str,
    start: Option<&T>,
    end: Option<&T>,
) {
    let options = options.downcast_ref::<RangeOptions<T>>();

    let start_options = options.map(|a| &a.start as &dyn Any).unwrap_or(&());
    let end_options = options.as_ref().map(|a| &a.end as &dyn Any).unwrap_or(&());

    ui.horizontal(|ui| {
        if let Some(start) = start {
            number_ui_readonly::<T>(start, ui, start_options, id, env.reborrow());
        }
        ui.label(symbol);
        if let Some(end) = end {
            number_ui_readonly::<T>(end, ui, end_options, id, env.reborrow());
        }
    });
}

impl<T: Reflect + TypePath + egui::emath::Numeric + InspectorOptionsType> InspectorPrimitive
    for std::ops::RangeInclusive<T>
{
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) -> bool {
        let mut start = *self.start();
        let mut end = *self.end();

        let changed = display_range::<T>(
            ui,
            options,
            id,
            env,
            "..=",
            Some(&mut start),
            Some(&mut end),
        );

        if changed {
            *self = start..=end;
        }

        changed
    }

    fn ui_readonly(
        &self,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) {
        display_range_readonly::<T>(
            ui,
            options,
            id,
            env,
            "..",
            Some(self.start()),
            Some(self.end()),
        );
    }
}

impl InspectorPrimitive for PathBuf {
    fn ui(&mut self, ui: &mut egui::Ui, _: &dyn Any, _: egui::Id, _: InspectorUi<'_, '_>) -> bool {
        let mut str = self.to_string_lossy();
        let changed = ui.text_edit_singleline(&mut str).changed();

        if changed {
            *self = PathBuf::from(str.as_str());
        }

        changed
    }

    fn ui_readonly(&self, ui: &mut egui::Ui, _: &dyn Any, _: egui::Id, _: InspectorUi<'_, '_>) {
        ui.text_edit_singleline(&mut self.to_string_lossy());
    }
}
