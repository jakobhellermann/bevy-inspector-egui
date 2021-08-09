use bevy_inspector_egui::Inspectable;

#[derive(Inspectable)]
struct Foo<T> {
    value: T,
}

#[derive(Inspectable)]
enum Enum<T: Default, U: Default> {
    T(T),
    U(U),
}
