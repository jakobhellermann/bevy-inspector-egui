use bevy::prelude::*;
use bevy_inspector_egui::minibuffer;
use bevy_inspector_egui::prelude::*;
use bevy_minibuffer::prelude::*;

#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct Configuration {
    name: String,
    #[inspector(min = 0.0, max = 1.0)]
    option: f32,
}

#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct Settings {
    name: String,
    #[inspector(min = 0.0, max = 1.0)]
    option: f32,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins,
                      MinibufferPlugins,
                      minibuffer::ResourceInspectorPlugins::default()
                      .add::<Configuration>()
                      .add::<Settings>()
        ))
        .init_resource::<Configuration>()
        .register_type::<Configuration>()
        .init_resource::<Settings>()
        .register_type::<Settings>()
        .add_acts((BasicActs::default(),
                   minibuffer::InspectorActs::default(),
        ))
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera2d);
        })
        .run();
}
