use bevy_inspector_egui::Inspectable;

#[derive(Inspectable)]
struct Struct {
    #[inspectable(label = "X")]
    x: f32,
    snake_case: f32,
}
