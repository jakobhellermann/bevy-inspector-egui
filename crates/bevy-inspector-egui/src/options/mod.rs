use std::{any::Any, borrow::Cow, collections::HashMap};

use bevy_reflect::{FromType, TypeData};

pub mod std_options;

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum Target {
    Field(usize),
    VariantField(Cow<'static, str>, usize),
}

#[derive(Default)]
pub struct InspectorOptions {
    options: HashMap<Target, Box<dyn TypeData>>,
}
impl Clone for InspectorOptions {
    fn clone(&self) -> Self {
        Self {
            options: self
                .options
                .iter()
                .map(|(target, data)| (target.clone(), TypeData::clone_type_data(&**data)))
                .collect(),
        }
    }
}
impl InspectorOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<T: TypeData>(&mut self, target: Target, options: T) {
        self.options.insert(target, Box::new(options));
    }
    pub fn insert_boxed(&mut self, target: Target, options: Box<dyn TypeData>) {
        self.options.insert(target, options);
    }
    pub fn get(&self, target: Target) -> Option<&dyn Any> {
        self.options.get(&target).map(|value| value.as_any())
    }
}

#[derive(Clone)]
pub struct ReflectInspectorOptions(pub InspectorOptions);

impl<T> FromType<T> for ReflectInspectorOptions
where
    InspectorOptions: FromType<T>,
{
    fn from_type() -> Self {
        ReflectInspectorOptions(InspectorOptions::from_type())
    }
}

pub trait InspectorOptionsType {
    type TypedOptions: Default;
    type Options: From<Self::TypedOptions>;
}
