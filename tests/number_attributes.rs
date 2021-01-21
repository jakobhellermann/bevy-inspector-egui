use bevy::math::prelude::*;
use bevy_inspector_egui::Inspectable;

#[derive(Inspectable)]
struct IntegerAttribues {
    #[inspectable(min = 2, max = 2, speed = 0.1)]
    a: u8,
    b: u16,
    c: u32,
    d: u64,
    e: i8,
    f: i16,
    #[inspectable(min = -2)]
    g: i32,
    h: i64,
    i: f32,
    #[inspectable(min = 0.1, max = 0.2)]
    j: f64,
    #[inspectable(min = Vec2::new(0.1, 0.2))]
    k: Vec2,
    l: Vec3,
}
