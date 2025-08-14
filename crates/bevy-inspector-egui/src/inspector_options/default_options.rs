use bevy_reflect::{TypeData, TypeInfo, TypeRegistry};

use crate::{
    InspectorOptions,
    inspector_options::{Target, std_options::NumberOptions},
    prelude::ReflectInspectorOptions,
};

#[allow(dead_code)]
fn insert_options_struct<T: 'static>(
    type_registry: &mut TypeRegistry,
    fields: &[(&'static str, &dyn TypeData)],
) {
    let Some(registration) = type_registry.get_mut(std::any::TypeId::of::<T>()) else {
        bevy_log::warn!(
            "Attempting to set default inspector options for {}, but it wasn't registered in the type registry.",
            std::any::type_name::<T>()
        );
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

#[allow(dead_code)]
fn insert_options_enum<T: 'static>(
    type_registry: &mut TypeRegistry,
    fields: &[(&'static str, &'static str, &dyn TypeData)],
) {
    let Some(registration) = type_registry.get_mut(std::any::TypeId::of::<T>()) else {
        bevy_log::warn!(
            "Attempting to set default inspector options for {}, but it wasn't registered in the type registry.",
            std::any::type_name::<T>()
        );
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
    insert_options_struct::<bevy_color::Srgba>(
        type_registry,
        &[
            ("red", &NumberOptions::<f32>::normalized()),
            ("green", &NumberOptions::<f32>::normalized()),
            ("blue", &NumberOptions::<f32>::normalized()),
            ("alpha", &NumberOptions::<f32>::normalized()),
        ],
    );
    insert_options_struct::<bevy_color::LinearRgba>(
        type_registry,
        &[
            ("red", &NumberOptions::<f32>::positive()),
            ("green", &NumberOptions::<f32>::positive()),
            ("blue", &NumberOptions::<f32>::positive()),
            ("alpha", &NumberOptions::<f32>::positive()),
        ],
    );
    insert_options_struct::<bevy_color::Hsla>(
        type_registry,
        &[
            ("hue", &NumberOptions::<f32>::between(0.0, 360.0)),
            ("saturation", &NumberOptions::<f32>::normalized()),
            ("lightness", &NumberOptions::<f32>::normalized()),
            ("alpha", &NumberOptions::<f32>::normalized()),
        ],
    );
    insert_options_struct::<bevy_color::Hsva>(
        type_registry,
        &[
            ("hue", &NumberOptions::<f32>::between(0.0, 360.0)),
            ("saturation", &NumberOptions::<f32>::normalized()),
            ("value", &NumberOptions::<f32>::normalized()),
            ("alpha", &NumberOptions::<f32>::normalized()),
        ],
    );
    insert_options_struct::<bevy_color::Hwba>(
        type_registry,
        &[
            ("hue", &NumberOptions::<f32>::between(0.0, 360.0)),
            ("whiteness", &NumberOptions::<f32>::normalized()),
            ("blackness", &NumberOptions::<f32>::normalized()),
            ("alpha", &NumberOptions::<f32>::normalized()),
        ],
    );
    insert_options_struct::<bevy_color::Laba>(
        type_registry,
        &[
            ("lightness", &NumberOptions::<f32>::between(0.0, 1.5)),
            ("a", &NumberOptions::<f32>::between(-1.5, 1.5)),
            ("b", &NumberOptions::<f32>::between(-1.5, 1.5)),
            ("alpha", &NumberOptions::<f32>::normalized()),
        ],
    );
    insert_options_struct::<bevy_color::Lcha>(
        type_registry,
        &[
            ("lightness", &NumberOptions::<f32>::between(0.0, 1.5)),
            ("chroma", &NumberOptions::<f32>::between(0.0, 1.5)),
            ("hue", &NumberOptions::<f32>::between(0.0, 360.0)),
            ("alpha", &NumberOptions::<f32>::normalized()),
        ],
    );
    insert_options_struct::<bevy_color::Oklaba>(
        type_registry,
        &[
            ("lightness", &NumberOptions::<f32>::normalized()),
            ("a", &NumberOptions::<f32>::between(-1.0, 1.0)),
            ("b", &NumberOptions::<f32>::between(-1.0, 1.0)),
            ("alpha", &NumberOptions::<f32>::normalized()),
        ],
    );
    insert_options_struct::<bevy_color::Oklcha>(
        type_registry,
        &[
            ("lightness", &NumberOptions::<f32>::normalized()),
            ("chroma", &NumberOptions::<f32>::normalized()),
            ("hue", &NumberOptions::<f32>::between(0.0, 360.0)),
            ("alpha", &NumberOptions::<f32>::normalized()),
        ],
    );
    insert_options_struct::<bevy_color::Xyza>(
        type_registry,
        &[
            ("x", &NumberOptions::<f32>::normalized()),
            ("y", &NumberOptions::<f32>::normalized()),
            ("z", &NumberOptions::<f32>::normalized()),
            ("alpha", &NumberOptions::<f32>::normalized()),
        ],
    );

    #[cfg(feature = "bevy_render")]
    {
        #[rustfmt::skip]
        insert_options_struct::<bevy_render::view::ColorGradingSection>(
            type_registry,
            &[
                ("saturation", &NumberOptions::<f32>::positive().with_speed(0.01)),
                ("contrast", &NumberOptions::<f32>::positive().with_speed(0.01)),
                ("gamma", &NumberOptions::<f32>::positive().with_speed(0.01)),
                ("gain", &NumberOptions::<f32>::positive().with_speed(0.01)),
                ("lift", &NumberOptions::<f32>::default().with_speed(0.01)),
            ],
        );

        #[rustfmt::skip]
        insert_options_struct::<bevy_render::view::ColorGradingGlobal>(
            type_registry,
            &[
                ("exposure", &NumberOptions::<f32>::default().with_speed(0.01)),
                ("temperature", &NumberOptions::<f32>::default().with_speed(0.01)),
                ("tint", &NumberOptions::<f32>::default().with_speed(0.01)),
                ("hue", &NumberOptions::<f32>::positive().with_speed(0.01)),
                ("post_saturation", &NumberOptions::<f32>::positive().with_speed(0.01)),
                ("midtones_range", &NumberOptions::<f32>::positive().with_speed(0.01)),
            ],
        );
    }

    #[cfg(feature = "bevy_pbr")]
    {
        #[rustfmt::skip]
        insert_options_struct::<bevy_light::AmbientLight>(
            type_registry,
            &[
                ("brightness", &NumberOptions::<f32>::positive()),
            ],
        );

        insert_options_struct::<bevy_light::PointLight>(
            type_registry,
            &[
                ("intensity", &NumberOptions::<f32>::positive()),
                ("range", &NumberOptions::<f32>::positive()),
                ("radius", &NumberOptions::<f32>::positive()),
            ],
        );

        #[rustfmt::skip]
        insert_options_struct::<bevy_light::DirectionalLight>(
            type_registry,
            &[
                ("illuminance", &NumberOptions::<f32>::positive()),
            ],
        );

        #[rustfmt::skip]
        insert_options_struct::<bevy_pbr::StandardMaterial>(
            type_registry,
            &[
                ("perceptual_roughness", &NumberOptions::<f32>::between(0.089, 1.0)),
                ("metallic", &NumberOptions::<f32>::normalized()),
                ("reflectance", &NumberOptions::<f32>::normalized()),
                ("depth_bias", &NumberOptions::<f32>::positive()),
            ],
        );

        #[rustfmt::skip]
        insert_options_enum::<bevy_light::cluster::ClusterConfig>(
            type_registry,
            &[
                ("FixedZ", "z_slices", &NumberOptions::<u32>::at_least(1)),
                ("XYZ", "dimensions", &NumberOptions::<bevy_math::UVec3>::at_least(bevy_math::UVec3::ONE)),
            ],
        );
    }

    #[rustfmt::skip]
    #[cfg(feature = "bevy_core_pipeline")]
    insert_options_enum::<bevy_camera::Camera3dDepthLoadOp>(
        type_registry,
        &[
            ("Clear", "0", &NumberOptions::<f32>::normalized()),
        ],
    );

    type_registry.register::<bevy_time::Virtual>();

    insert_options_struct::<bevy_time::Virtual>(
        type_registry,
        &[
            ("relative_speed", &NumberOptions::<f64>::positive()),
            ("effective_speed", &NumberOptions::<f64>::positive()),
        ],
    );
}
