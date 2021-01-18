use bevy_inspector_egui::Inspectable;

#[derive(Inspectable, Debug, PartialEq)]
enum Test {
    A,
    B,
    C,
}
