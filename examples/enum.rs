use bevy::prelude::*;

use bevy_inspector_egui::{widgets::InspectableButton, Inspectable, InspectorPlugin};

#[derive(Default)]
struct Print;

#[derive(Default, Inspectable)]
struct Inspector {
    shape: Shape,
    #[inspectable(label = "", text = "Print")]
    print: InspectableButton<Print>,
}

#[derive(Inspectable, Debug)]
enum Shape {
    Box {
        size: Vec3,
    },
    Icosphere {
        #[inspectable(min = 1)]
        subdivisions: usize,
        #[inspectable(default = 5.0, min = 0.1)]
        radius: f32,
    },
    Line(Vec2, Vec2),
    UnitSphere,
}
impl Default for Shape {
    fn default() -> Self {
        Shape::Box {
            size: Default::default(),
        }
    }
}

fn main() {
    App::build()
        .insert_resource(ClearColor(Color::rgb(0.039, 0.055, 0.078)))
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<Inspector>::new())
        .add_system(print_inspector.system())
        .run();
}

fn print_inspector(inspector: Res<Inspector>, mut events: EventReader<Print>) {
    for _ in events.iter() {
        println!("{:?}", inspector.shape);
    }
}
