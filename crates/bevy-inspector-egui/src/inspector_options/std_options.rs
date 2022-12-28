use super::InspectorOptionsType;

macro_rules! impl_options {
    ($ty:ty => $options:ty) => {
        impl InspectorOptionsType for $ty {
            type TypedOptions = $options;
            type Options = $options;
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
}

impl<T> Default for NumberOptions<T> {
    fn default() -> Self {
        Self {
            min: Default::default(),
            max: Default::default(),
            speed: 0.0,
            prefix: String::new(),
            suffix: String::new(),
        }
    }
}

impl<T> NumberOptions<T> {
    pub fn between(min: T, max: T) -> NumberOptions<T> {
        NumberOptions {
            min: Some(min),
            max: Some(max),
            speed: 0.0,
            prefix: String::new(),
            suffix: String::new(),
        }
    }
    pub fn at_least(min: T) -> NumberOptions<T> {
        NumberOptions {
            min: Some(min),
            max: None,
            speed: 0.0,
            prefix: String::new(),
            suffix: String::new(),
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
        }
    }

    pub fn normalized() -> Self {
        NumberOptions {
            min: Some(T::from_f64(0.0)),
            max: Some(T::from_f64(1.0)),
            speed: 0.01,
            prefix: String::new(),
            suffix: String::new(),
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
