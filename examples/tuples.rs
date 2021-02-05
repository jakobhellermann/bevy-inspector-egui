use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};

#[derive(Inspectable, Default)]
struct Data {
    a: (),
    b: (u8,),
    c: (u8, u8),
    d: (u8, u8, u8),
    e: (u8, u8, u8, u8),
    #[inspectable(0 = (), 2 = ((), (), ()))]
    u_u: ((), ((), ()), ((), (), ())),
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<Data>::new())
        .run();
}
