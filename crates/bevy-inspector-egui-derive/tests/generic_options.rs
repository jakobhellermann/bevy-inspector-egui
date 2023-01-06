use std::marker::PhantomData;

use bevy_inspector_egui::{
    inspector_options::{std_options::NumberOptions, Target},
    InspectorOptions,
};
use bevy_reflect::{FromType, Reflect};

#[test]
fn generic_without_options() {
    #[derive(Reflect, InspectorOptions)]
    struct Generic<T: Reflect> {
        b: T,

        #[inspector(min = 0.0)]
        other: f32,
    }

    let options = <InspectorOptions as FromType<Generic<f32>>>::from_type();

    let options = options
        .get(Target::Field(1))
        .unwrap()
        .downcast_ref::<NumberOptions<f32>>()
        .unwrap();
    assert_eq!(options.min, Some(0.0));
}

#[test]
fn phantom_data() {
    #[derive(Reflect, InspectorOptions)]
    struct Generic<T: 'static> {
        #[reflect(ignore)]
        _marker: PhantomData<fn() -> T>,

        #[inspector(min = 0.0)]
        other: f32,
    }

    let options = <InspectorOptions as FromType<Generic<f32>>>::from_type();

    let options = options
        .get(Target::Field(0))
        .unwrap()
        .downcast_ref::<NumberOptions<f32>>()
        .unwrap();
    assert_eq!(options.min, Some(0.0));
}
