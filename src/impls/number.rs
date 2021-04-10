use crate::egui::{self, widgets};
use crate::Context;
use crate::Inspectable;

#[derive(Debug, Clone)]
pub struct NumberAttributes<T> {
    pub min: Option<T>,
    pub max: Option<T>,
    /// How much the value changes when dragged one logical pixel.
    pub speed: f32,
    pub prefix: String,
    pub suffix: String,
}
impl<T> Default for NumberAttributes<T> {
    fn default() -> Self {
        NumberAttributes {
            min: None,
            max: None,
            speed: 0.0,
            prefix: "".to_string(),
            suffix: "".to_string(),
        }
    }
}

impl<T> NumberAttributes<T> {
    pub(crate) fn map<U>(&self, f: impl Fn(&T) -> U) -> NumberAttributes<U> {
        NumberAttributes {
            min: self.min.as_ref().map(|v| f(v)),
            max: self.max.as_ref().map(f),
            speed: self.speed,
            prefix: self.prefix.clone(),
            suffix: self.suffix.clone(),
        }
    }

    pub fn min(min: T) -> Self {
        NumberAttributes {
            min: Some(min),
            max: None,
            ..Default::default()
        }
    }

    pub fn between(min: T, max: T) -> Self {
        NumberAttributes {
            min: Some(min),
            max: Some(max),
            ..Default::default()
        }
    }

    pub(crate) fn speed(self, speed: f32) -> Self {
        NumberAttributes { speed, ..self }
    }
}
impl NumberAttributes<f32> {
    pub(crate) fn positive() -> Self {
        NumberAttributes::min(0.0)
    }

    pub(crate) fn normalized() -> Self {
        NumberAttributes::between(0.0, 1.0).speed(0.1)
    }
}

pub trait Num: emath::Numeric {
    fn default_speed() -> Option<f32> {
        None
    }
}

impl Num for f32 {
    fn default_speed() -> Option<f32> {
        Some(0.1)
    }
}
impl Num for f64 {
    fn default_speed() -> Option<f32> {
        Some(0.1)
    }
}
impl Num for i8 {}
impl Num for u8 {}
impl Num for i16 {}
impl Num for u16 {}
impl Num for i32 {}
impl Num for u32 {}
impl Num for i64 {}
impl Num for u64 {}
impl Num for isize {}
impl Num for usize {}

impl<T: Num> Inspectable for T {
    type Attributes = NumberAttributes<T>;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, _: &Context) {
        let mut widget = widgets::DragValue::new(self);

        if !options.prefix.is_empty() {
            widget = widget.prefix(options.prefix);
        }
        if !options.suffix.is_empty() {
            widget = widget.suffix(options.suffix);
        }

        match (options.min, options.max) {
            (Some(min), Some(max)) => widget = widget.clamp_range(min.to_f64()..=max.to_f64()),
            (Some(min), None) => widget = widget.clamp_range(min.to_f64()..=f64::MAX),
            (None, Some(max)) => widget = widget.clamp_range(f64::MIN..=max.to_f64()),
            (None, None) => {}
        }

        if options.speed != 0.0 {
            widget = widget.speed(options.speed);
        } else if let Some(default_speed) = T::default_speed() {
            widget = widget.speed(default_speed);
        }

        ui.add(widget);
    }
}
