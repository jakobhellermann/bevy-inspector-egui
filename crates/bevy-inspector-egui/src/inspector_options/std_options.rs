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
}
impl NumberOptions<f32> {
    pub fn normalized() -> Self {
        NumberOptions {
            min: Some(0.0),
            max: Some(1.0),
            speed: 0.01,
            prefix: String::new(),
            suffix: String::new(),
        }
    }
}
impl NumberOptions<f64> {
    pub fn normalized() -> Self {
        NumberOptions {
            min: Some(0.0),
            max: Some(1.0),
            speed: 0.01,
            prefix: String::new(),
            suffix: String::new(),
        }
    }
}

impl_options!(f32 => NumberOptions<f32>);
impl_options!(usize => NumberOptions<usize>);

impl<T> InspectorOptionsType for Option<T> {
    type TypedOptions = ();
    type Options = ();
}

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
