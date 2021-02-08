use crate::egui::{self, widgets};
use crate::Context;
use crate::Inspectable;

#[derive(Debug, Default, Clone)]
pub struct NumberAttributes<T> {
    pub min: Option<T>,
    pub max: Option<T>,
    /// How much the value changes when dragged one logical pixel.
    pub speed: f32,
    pub prefix: String,
    pub suffix: String,
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
            speed: 0.0,
            prefix: "".into(),
            suffix: "".into(),
        }
    }
}

macro_rules! impl_for_num {
    ($ty:ident $(default_speed=$default_speed:expr)? ) => {
        impl Inspectable for $ty {
            type Attributes = NumberAttributes<$ty>;

            fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, _: &Context) {
                let mut widget = widgets::DragValue::$ty(self);

                if !options.prefix.is_empty() {
                    widget = widget.prefix(options.prefix);
                }
                if !options.suffix.is_empty() {
                    widget = widget.suffix(options.suffix);
                }

                match (options.min, options.max) {
                    (Some(min), Some(max)) => widget = widget.clamp_range(min as f32..=max as f32),
                    (Some(min), None) => widget = widget.clamp_range(min as f32..=f32::MAX),
                    (None, Some(max)) => widget = widget.clamp_range(f32::MIN..=max as f32),
                    (None, None) => {},
                }

                if options.speed != 0.0 {
                    widget = widget.speed(options.speed);
                } $(else {
                    widget = widget.speed($default_speed);
                })?

                ui.add(widget);


                if let Some(min) = options.min {
                    *self = (*self).max(min);
                }
                if let Some(max) = options.max {
                    *self = (*self).min(max);
                }
            }
        }
    };
}

macro_rules! impl_for_num_delegate_f64 {
    ($ty:ty) => {
        impl Inspectable for $ty {
            type Attributes = NumberAttributes<$ty>;

            fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context) {
                let mut options_f64 = options.map(|val| *val as f64);
                    if options_f64.speed == 0.0 {
                        options_f64.speed = 1.0;
                    }

                let mut value = *self as f64;
                <f64 as Inspectable>::ui(&mut value, ui, options_f64, context);

                *self = value as $ty;
            }
        }
    };

    ( $($ty:ty),* ) => {
        $( impl_for_num_delegate_f64!($ty); )*
    }
}

impl_for_num!(f32 default_speed = 0.1);
impl_for_num!(f64 default_speed = 0.1);

impl_for_num!(u8);
impl_for_num!(i32);

impl_for_num_delegate_f64!(u16, u32, u64, usize);
impl_for_num_delegate_f64!(i8, i16, i64, isize);
