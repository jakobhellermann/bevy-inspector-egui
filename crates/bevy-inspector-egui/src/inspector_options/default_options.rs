use bevy_reflect::{TypeData, TypeInfo, TypeRegistry};

use crate::{
    inspector_options::{std_options::NumberOptions, Target},
    prelude::ReflectInspectorOptions,
    InspectorOptions,
};

#[allow(dead_code)]
fn insert_options_struct<T: 'static>(
    type_registry: &mut TypeRegistry,
    fields: &[(&'static str, &dyn TypeData)],
) {
    let Some(registration) = type_registry.get_mut(std::any::TypeId::of::<T>()) else {
        bevy_log::warn!("Attempting to set default inspector options for {}, but it wasn't registered in the type registry.", std::any::type_name::<T>());
        return;
    };
    if registration.data::<ReflectInspectorOptions>().is_none() {
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
    let Some(registration) = type_registry.get_mut(std::any::TypeId::of::<T>()) else {
        bevy_log::warn!("Attempting to set default inspector options for {}, but it wasn't registered in the type registry.", std::any::type_name::<T>());
        return;
    };
    if registration.data::<ReflectInspectorOptions>().is_none() {
        let mut options = InspectorOptions::new();
        for (variant, field, data) in fields {
            let info = match registration.type_info() {
                TypeInfo::Enum(info) => info,
                _ => unreachable!(),
            };
            let variant_index = info.index_of(variant).unwrap();
            let field_index = match info.variant_at(variant_index).unwrap() {
                bevy_reflect::VariantInfo::Struct(strukt) => strukt.index_of(field).unwrap(),
                bevy_reflect::VariantInfo::Tuple(_) => field.parse().unwrap(),
                bevy_reflect::VariantInfo::Unit(_) => unreachable!(),
            };
            options.insert_boxed(
                Target::VariantField {
                    variant_index,
                    field_index,
                },
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

    insert_options_struct::<bevy_render::view::ColorGrading>(
        type_registry,
        &[
            (
                "exposure",
                &NumberOptions::<f32>::positive().with_speed(0.01),
            ),
            ("gamma", &NumberOptions::<f32>::positive().with_speed(0.01)),
            (
                "pre_saturation",
                &NumberOptions::<f32>::positive().with_speed(0.01),
            ),
            (
                "post_saturation",
                &NumberOptions::<f32>::positive().with_speed(0.01),
            ),
        ],
    );

    #[cfg(feature = "bevy_pbr")]
    {
        insert_options_struct::<bevy_pbr::AmbientLight>(
            type_registry,
            &[("brightness", &NumberOptions::<f32>::normalized())],
        );
        insert_options_struct::<bevy_pbr::PointLight>(
            type_registry,
            &[
                ("intensity", &NumberOptions::<f32>::positive()),
                ("range", &NumberOptions::<f32>::positive()),
                ("radius", &NumberOptions::<f32>::positive()),
            ],
        );
        insert_options_struct::<bevy_pbr::DirectionalLight>(
            type_registry,
            &[("illuminance", &NumberOptions::<f32>::positive())],
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
        insert_options_enum::<bevy_pbr::ClusterConfig>(
            type_registry,
            &[
                ("FixedZ", "z_slices", &NumberOptions::<u32>::at_least(1)),
                (
                    "XYZ",
                    "dimensions",
                    &NumberOptions::<bevy_math::UVec3>::at_least(bevy_math::UVec3::ONE),
                ),
            ],
        );
    }

    insert_options_enum::<bevy_core_pipeline::core_3d::Camera3dDepthLoadOp>(
        type_registry,
        &[("Clear", "0", &NumberOptions::<f32>::normalized())],
    );
}
