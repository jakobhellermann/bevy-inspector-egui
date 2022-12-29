use bevy_inspector_egui::{
    inspector_options::{
        std_options::{NumberOptions, QuatDisplay, QuatOptions},
        Target,
    },
    InspectorOptions,
};
use bevy_math::Quat;
use bevy_reflect::{FromType, Reflect};

#[test]
fn expr_attribute() {
    #[derive(Reflect, InspectorOptions)]
    struct Test {
        #[inspector(display = QuatDisplay::Euler)]
        b: Quat,

        #[inspector(min = 0.0)]
        option: Option<f32>,
    }

    let options = <InspectorOptions as FromType<Test>>::from_type();

    let quat_options = options
        .get(Target::Field(0))
        .unwrap()
        .downcast_ref::<QuatOptions>()
        .unwrap();
    assert!(matches!(quat_options.display, QuatDisplay::Euler));

    let option_options = options
        .get(Target::Field(1))
        .unwrap()
        .downcast_ref::<InspectorOptions>()
        .unwrap();
    let option_inner_options = option_options
        .get(Target::VariantField {
            variant_index: 1,
            field_index: 0,
        })
        .unwrap()
        .downcast_ref::<NumberOptions<f32>>()
        .unwrap();
    assert_eq!(option_inner_options.min, Some(0.0));
}
