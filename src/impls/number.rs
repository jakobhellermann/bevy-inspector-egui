use crate::egui::{self, widgets};
use crate::Inspectable;

#[derive(Debug, Clone)]
pub struct NumberAttributes<T> {
    pub min: T,
    pub max: T,
    /// How much the value changes when dragged one logical pixel.
    pub speed: f32,
    pub prefix: String,
    pub suffix: String,
}
impl<T: Default> Default for NumberAttributes<T> {
    fn default() -> Self {
        NumberAttributes {
            min: T::default(),
            max: T::default(),
            speed: 0.0,
            prefix: "".into(),
            suffix: "".into(),
        }
    }
}
impl<T> NumberAttributes<T> {
    pub(crate) fn map<U>(&self, f: impl Fn(&T) -> U) -> NumberAttributes<U> {
        NumberAttributes {
            min: f(&self.min),
            max: f(&self.max),
            speed: self.speed,
            prefix: self.prefix.clone(),
            suffix: self.suffix.clone(),
        }
    }
}

macro_rules! impl_for_num {
    ($ty:ident $(default_speed=$default_speed:expr)? ) => {
        impl Inspectable for $ty {
            type Attributes = NumberAttributes<$ty>;

            fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes) {
                let mut widget = widgets::DragValue::$ty(self);

                if !options.prefix.is_empty() {
                    widget = widget.prefix(options.prefix);
                }
                if !options.suffix.is_empty() {
                    widget = widget.suffix(options.suffix);
                }

                if options.min != options.max {
                    widget = widget.range(options.min as f32..=options.max as f32);
                }

                if options.speed != 0.0 {
                    widget = widget.speed(options.speed);
                } $(else {
                    widget = widget.speed($default_speed);
                })?

                ui.add(widget);
            }
        }
    };
}

macro_rules! impl_for_num_delegate_f64 {
    ($ty:ty) => {
        impl Inspectable for $ty {
            type Attributes = NumberAttributes<$ty>;

            fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes) {
                let mut options_f64 = options.map(|val| *val as f64);
                    if options_f64.speed == 0.0 {
                        options_f64.speed = 1.0;
                    }

                let mut value = *self as f64;
                <f64 as Inspectable>::ui(&mut value, ui, options_f64);

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

impl_for_num_delegate_f64!(u16, u32, u64);
impl_for_num_delegate_f64!(i8, i16, i64);
