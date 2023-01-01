use std::collections::VecDeque;

use crate::InspectorOptions;

use super::{InspectorOptionsType, Target};

macro_rules! impl_options {
    ($ty:ty => $options:ty) => {
        impl InspectorOptionsType for $ty {
            type DeriveOptions = $options;
            type Options = $options;

            fn options_from_derive(options: Self::DeriveOptions) -> Self::Options {
                options
            }
        }
    };
}

#[derive(Clone)]
pub struct NumberOptions<T> {
    pub min: Option<T>,
    pub max: Option<T>,
    pub speed: f32,
    pub prefix: String,
    pub suffix: String,
    pub display: NumberDisplay,
}

impl<T> Default for NumberOptions<T> {
    fn default() -> Self {
        Self {
            min: None,
            max: None,
            speed: 0.0,
            prefix: String::new(),
            suffix: String::new(),
            display: NumberDisplay::default(),
        }
    }
}

#[derive(Clone, Copy, Default)]
#[non_exhaustive]
pub enum NumberDisplay {
    #[default]
    Drag,
    Slider,
}

impl<T> NumberOptions<T> {
    pub fn between(min: T, max: T) -> NumberOptions<T> {
        NumberOptions {
            min: Some(min),
            max: Some(max),
            speed: 0.0,
            prefix: String::new(),
            suffix: String::new(),
            display: NumberDisplay::default(),
        }
    }
    pub fn at_least(min: T) -> NumberOptions<T> {
        NumberOptions {
            min: Some(min),
            max: None,
            speed: 0.0,
            prefix: String::new(),
            suffix: String::new(),
            display: NumberDisplay::default(),
        }
    }

    pub fn with_speed(self, speed: f32) -> NumberOptions<T> {
        NumberOptions { speed, ..self }
    }

    pub fn map<U>(&self, f: impl Fn(&T) -> U) -> NumberOptions<U> {
        NumberOptions {
            min: self.min.as_ref().map(|min| f(min)),
            max: self.max.as_ref().map(f),
            speed: self.speed,
            prefix: self.prefix.clone(),
            suffix: self.suffix.clone(),
            display: NumberDisplay::default(),
        }
    }
}
impl<T: egui::emath::Numeric> NumberOptions<T> {
    pub fn positive() -> NumberOptions<T> {
        NumberOptions {
            min: Some(T::from_f64(0.0)),
            max: None,
            speed: 0.0,
            prefix: String::new(),
            suffix: String::new(),
            display: NumberDisplay::default(),
        }
    }

    pub fn normalized() -> Self {
        NumberOptions {
            min: Some(T::from_f64(0.0)),
            max: Some(T::from_f64(1.0)),
            speed: 0.01,
            prefix: String::new(),
            suffix: String::new(),
            display: NumberDisplay::default(),
        }
    }
}

impl_options!(f32 => NumberOptions<f32>);
impl_options!(f64 => NumberOptions<f64>);
impl_options!(i8 => NumberOptions<i8>);
impl_options!(i16 => NumberOptions<i16>);
impl_options!(i32 => NumberOptions<i32>);
impl_options!(i64 => NumberOptions<i64>);
impl_options!(i128 => NumberOptions<i128>);
impl_options!(isize => NumberOptions<isize>);
impl_options!(u8 => NumberOptions<u8>);
impl_options!(u16 => NumberOptions<u16>);
impl_options!(u32 => NumberOptions<u32>);
impl_options!(u64 => NumberOptions<u64>);
impl_options!(u128 => NumberOptions<u128>);
impl_options!(usize => NumberOptions<usize>);

#[derive(Clone)]
pub struct QuatOptions {
    pub display: QuatDisplay,
}

#[derive(Copy, Clone, Debug)]
pub enum QuatDisplay {
    Raw,
    Euler,
    YawPitchRoll,
    AxisAngle,
}

impl Default for QuatOptions {
    fn default() -> Self {
        QuatOptions {
            display: QuatDisplay::Euler,
        }
    }
}

impl_options!(bevy_math::Quat => QuatOptions);

impl<T: InspectorOptionsType> InspectorOptionsType for Option<T> {
    type DeriveOptions = T::DeriveOptions;
    type Options = InspectorOptions;

    fn options_from_derive(options: Self::DeriveOptions) -> Self::Options {
        let inner_options = T::options_from_derive(options);

        let mut inspector_options = InspectorOptions::new();
        inspector_options.insert(
            Target::VariantField {
                variant_index: 1, // Some
                field_index: 0,
            },
            inner_options,
        );

        inspector_options
    }
}

macro_rules! impl_options_defer_generic {
    ($name:ident < $generic:ident >) => {
        impl<T: InspectorOptionsType> InspectorOptionsType for $name<$generic> {
            type DeriveOptions = $generic::DeriveOptions;
            type Options = $generic::Options;

            fn options_from_derive(options: Self::DeriveOptions) -> Self::Options {
                $generic::options_from_derive(options)
            }
        }
    };
}

impl_options_defer_generic!(Vec<T>);
impl_options_defer_generic!(VecDeque<T>);

impl<T: InspectorOptionsType, const N: usize> InspectorOptionsType for [T; N] {
    type DeriveOptions = T::DeriveOptions;
    type Options = T::Options;

    fn options_from_derive(options: Self::DeriveOptions) -> Self::Options {
        T::options_from_derive(options)
    }
}
