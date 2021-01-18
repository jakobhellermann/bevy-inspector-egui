use bevy::prelude::Color;
use bevy_inspector_egui::Inspectable;

#[derive(Inspectable)]
struct Struct {
    #[inspectable(min = 1.0, max = -100.0)]
    x: f32,
    #[inspectable(alpha = true)]
    color: Color,
}
