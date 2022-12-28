use bevy_inspector_egui::{
    inspector_options::{std_options::NumberOptions, Target},
    InspectorOptions,
};
use bevy_reflect::{FromType, Reflect};

#[test]
fn check_options_ignore_struct() {
    #[derive(Reflect, InspectorOptions)]
    struct Test {
        #[reflect(ignore)]
        _a: f32,
        #[inspector(min = 2.0, max = 3.0)]
        b: f32,
    }

    let options = <InspectorOptions as FromType<Test>>::from_type();
    assert_eq!(options.iter().count(), 1);

    let b_options = options
        .get(Target::Field(0))
        .unwrap()
        .downcast_ref::<NumberOptions<f32>>()
        .unwrap();
    assert_eq!(b_options.min, Some(2.0));
}

#[test]
fn check_options_ignore_enum() {
    #[derive(Reflect, InspectorOptions)]
    enum Test {
        Variant {
            #[reflect(ignore)]
            _ignored: f32,
            #[inspector(min = 0.0)]
            no_ignored: f32,
        },
    }

    let options = <InspectorOptions as FromType<Test>>::from_type();
    assert_eq!(options.iter().count(), 1);

    let field_options = options
        .get(Target::VariantField {
            variant_index: 0,
            field_index: 0,
        })
        .unwrap()
        .downcast_ref::<NumberOptions<f32>>()
        .unwrap();
    assert_eq!(field_options.min, Some(0.0));
}
