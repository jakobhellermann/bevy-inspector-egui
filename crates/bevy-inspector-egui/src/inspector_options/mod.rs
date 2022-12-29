//! Way of associating options to fields using [`struct@InspectorOptions`]

use std::{any::Any, collections::HashMap};

use bevy_reflect::{FromType, TypeData};

pub(crate) mod default_options;

/// Options for dealing with common types such as numbers or quaternions
pub mod std_options;

/// Descriptor of a path into a struct/enum. Either a `Field` (`.foo`) or a `VariantField` (`RGBA.r`)
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Target {
    Field(usize),
    VariantField {
        variant_index: usize,
        field_index: usize,
    },
}

pub use bevy_inspector_egui_derive::InspectorOptions;

/// Map of [`Target`]s to arbitrary [`TypeData`] used to control how the value is displayed, e.g. [`NumberOptions`](crate::inspector_options::std_options::NumberOptions).
///
/// Comes with a [derive macro](derive@InspectorOptions), which generates a `FromType<T> for InspectorOptions` impl:
/// ```rust
/// use bevy_inspector_egui::prelude::*;
/// use bevy_reflect::Reflect;
///
/// #[derive(Reflect, Default, InspectorOptions)]
/// #[reflect(InspectorOptions)]
/// struct Config {
///     #[inspector(min = 10.0, max = 70.0)]
///     font_size: f32,
///     option: Option<f32>,
/// }
/// ```
/// will expand roughly to
/// ```rust
/// # use bevy_inspector_egui::inspector_options::{InspectorOptions, Target, std_options::NumberOptions};
/// let mut options = InspectorOptions::default();
/// let mut field_options = NumberOptions { min: 10.0.into(), max: 70.0.into(), ..Default::default() };
/// options.insert(Target::Field(0usize), field_options);
/// ```
#[derive(Default)]
pub struct InspectorOptions {
    options: HashMap<Target, Box<dyn TypeData>>,
}

impl std::fmt::Debug for InspectorOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut options = f.debug_struct("InspectorOptions");
        for entry in self.options.keys() {
            options.field(&format!("{entry:?}"), &"..");
        }
        options.finish()
    }
}

impl Clone for InspectorOptions {
    fn clone(&self) -> Self {
        Self {
            options: self
                .options
                .iter()
                .map(|(target, data)| (*target, TypeData::clone_type_data(&**data)))
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

    pub fn iter(&self) -> impl Iterator<Item = (Target, &dyn TypeData)> + '_ {
        self.options.iter().map(|(target, data)| (*target, &**data))
    }
}

/// Wrapper of [`struct@InspectorOptions`] to be stored in the [`TypeRegistry`](bevy_reflect::TypeRegistry)
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

/// Helper trait for the [`derive@InspectorOptions`] macro.
///
/// ```skip
///     #[inspector(min = 0.0, max = 1.0)]
///     field: f32
/// ```
/// will expand to this:
/// ```rust
/// # use std::convert::Into;
/// # use bevy_inspector_egui::inspector_options::{InspectorOptions, InspectorOptionsType, Target};
/// let mut options = InspectorOptions::default();
/// let mut field_options =  <f32 as InspectorOptionsType>::DeriveOptions::default();
/// field_options.min = Into::into(2.0);
/// field_options.max = Into::into(3.0);
/// options.insert(
///     Target::Field(0usize),
///     <f32 as InspectorOptionsType>::options_from_derive(field_options),
/// );
pub trait InspectorOptionsType {
    type DeriveOptions: Default;
    type Options;

    fn options_from_derive(options: Self::DeriveOptions) -> Self::Options;
}
