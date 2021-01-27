use bevy::prelude::*;
use bevy_inspector_egui::{widgets::ReflectedUI, Inspectable, InspectorPlugin};

#[derive(Inspectable, Default, Debug)]
struct Data {
    // it works for custom reflect types
    custom: ReflectedUI<MyComponent>,
    // also for builtin implementations
    color: ReflectedUI<Color>,
    // and for most of bevy's types
    timer: ReflectedUI<Timer>,
}

#[derive(Reflect, Default, Debug)]
struct MyComponent {
    a: f32,
    b: Vec2,
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<Data>::new())
        .run();
}
