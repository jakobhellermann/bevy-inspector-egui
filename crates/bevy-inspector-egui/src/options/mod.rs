use std::{any::Any, borrow::Cow, collections::HashMap};

pub mod std_options;

#[derive(Hash, PartialEq, Eq, Clone)]
pub enum Target {
    Field(usize),
    VariantField(Cow<'static, str>, usize),
}

#[derive(Default)]
pub struct InspectorOptions {
    options: HashMap<Target, Box<dyn Any>>,
}
impl InspectorOptions {
    pub fn insert<T: Any>(&mut self, target: Target, options: T) {
        self.options.insert(target, Box::new(options));
    }

    pub fn get(&self, target: Target) -> Option<&dyn Any> {
        self.options.get(&target).map(|value| &**value)
    }
}

pub trait InspectorOptionsType {
    type TypedOptions: Default;
    type Options: From<Self::TypedOptions>;
}
