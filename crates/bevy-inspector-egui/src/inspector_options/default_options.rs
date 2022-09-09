use std::borrow::Cow;

use bevy_reflect::{TypeData, TypeInfo, TypeRegistry};

use crate::{
    inspector_options::{std_options::NumberOptions, Target},
    prelude::ReflectInspectorOptions,
    InspectorOptions,
};

fn insert_options_struct<T: 'static>(
    type_registry: &mut TypeRegistry,
    fields: &[(&'static str, &dyn TypeData)],
) {
    let registration = type_registry.get_mut(std::any::TypeId::of::<T>()).unwrap();
    if !registration.data::<ReflectInspectorOptions>().is_some() {
        let mut options = InspectorOptions::new();
        for (field, data) in fields {
            let info = match registration.type_info() {
                TypeInfo::Struct(info) => info,
                _ => unreachable!(),
            };
            let field_index = info.index_of(field).unwrap();
            options.insert_boxed(Target::Field(field_index), TypeData::clone_type_data(*data));
        }
        registration.insert(ReflectInspectorOptions(options));
    }
}

fn insert_options_enum<T: 'static>(
    type_registry: &mut TypeRegistry,
    fields: &[(&'static str, &'static str, &dyn TypeData)],
) {
    let registration = type_registry.get_mut(std::any::TypeId::of::<T>()).unwrap();
    if !registration.data::<ReflectInspectorOptions>().is_some() {
        let mut options = InspectorOptions::new();
        for (variant, field, data) in fields {
            let info = match registration.type_info() {
                TypeInfo::Enum(info) => info,
                _ => unreachable!(),
            };
            let field_index = match info.variant(variant).unwrap() {
                bevy_reflect::VariantInfo::Struct(strukt) => strukt.index_of(field).unwrap(),
                bevy_reflect::VariantInfo::Tuple(_) => todo!(),
                bevy_reflect::VariantInfo::Unit(_) => todo!(),
            };
            options.insert_boxed(
                Target::VariantField(Cow::Borrowed(variant), field_index),
                TypeData::clone_type_data(*data),
            );
        }
        registration.insert(ReflectInspectorOptions(options));
    }
}

pub fn register_default_options(type_registry: &mut TypeRegistry) {
    insert_options_enum::<bevy_render::color::Color>(
        type_registry,
        &[
            ("Rgba", "red", &NumberOptions::<f32>::normalized()),
            ("Rgba", "green", &NumberOptions::<f32>::normalized()),
            ("Rgba", "blue", &NumberOptions::<f32>::normalized()),
            ("Rgba", "alpha", &NumberOptions::<f32>::normalized()),
            ("RgbaLinear", "red", &NumberOptions::<f32>::normalized()),
            ("RgbaLinear", "green", &NumberOptions::<f32>::normalized()),
            ("RgbaLinear", "blue", &NumberOptions::<f32>::normalized()),
            ("RgbaLinear", "alpha", &NumberOptions::<f32>::normalized()),
            ("Hsla", "hue", &NumberOptions::<f32>::between(0.0, 360.0)),
            ("Hsla", "saturation", &NumberOptions::<f32>::normalized()),
            ("Hsla", "lightness", &NumberOptions::<f32>::normalized()),
            ("Hsla", "alpha", &NumberOptions::<f32>::normalized()),
        ],
    );

    insert_options_struct::<bevy_pbr::AmbientLight>(
        type_registry,
        &[("brightness", &NumberOptions::<f32>::normalized())],
    );
    insert_options_struct::<bevy_pbr::StandardMaterial>(
        type_registry,
        &[
            (
                "perceptual_roughness",
                &NumberOptions::<f32>::between(0.089, 1.0),
            ),
            ("metallic", &NumberOptions::<f32>::normalized()),
            ("reflectance", &NumberOptions::<f32>::normalized()),
            ("depth_bias", &NumberOptions::<f32>::positive()),
        ],
    );
}
