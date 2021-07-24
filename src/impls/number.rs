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

macro_rules! impl_num {
    ($ty:ty $(| $default_speed:literal)?) => {
        impl Inspectable for $ty {
            type Attributes = NumberAttributes<$ty>;

            fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, _: &Context) -> bool {
                num_ui(self, options, ui, None)
            }
        }
    };
}

impl_num!(i8);
impl_num!(i16);
impl_num!(i32);
impl_num!(i64);
impl_num!(isize);
impl_num!(u8);
impl_num!(u16);
impl_num!(u32);
impl_num!(u64);
impl_num!(usize);

impl Inspectable for f32 {
    type Attributes = NumberAttributes<f32>;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, _: &Context) -> bool {
        num_ui(self, options, ui, Some(0.1))
    }
}
impl Inspectable for f64 {
    type Attributes = NumberAttributes<f64>;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, _: &Context) -> bool {
        num_ui(self, options, ui, Some(0.1))
    }
}

fn num_ui<T: emath::Numeric>(
    value: &mut T,
    options: NumberAttributes<T>,
    ui: &mut egui::Ui,
    default_speed: Option<f32>,
) -> bool {
    let mut widget = widgets::DragValue::new(value);
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
    } else if let Some(default_speed) = default_speed {
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
