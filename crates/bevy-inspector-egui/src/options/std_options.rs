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

impl_options!(f32 => NumberOptions<f32>);
impl_options!(usize => NumberOptions<usize>);

impl<T> InspectorOptionsType for Option<T> {
    type TypedOptions = ();
    type Options = ();
}
