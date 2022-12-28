use bevy_inspector_egui::{
    inspector_options::{
        std_options::{QuatDisplay, QuatOptions},
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
    }

    let options = <InspectorOptions as FromType<Test>>::from_type();
    assert_eq!(options.iter().count(), 1);

    let quat_options = options
        .get(Target::Field(0))
        .unwrap()
        .downcast_ref::<QuatOptions>()
        .unwrap();
    assert!(matches!(quat_options.display, QuatDisplay::Euler));
}
