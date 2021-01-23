use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};

// #[derive(Inspectable, Debug)]
#[derive(Inspectable, Default, Debug)]
struct Data {
    material: Handle<StandardMaterial>,
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<Data>::thread_local())
        .add_startup_system(setup.system())
        .run();
}

struct Cube;

fn setup(
    commands: &mut Commands,
    mut data: ResMut<Data>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = materials.add(Color::rgb(0.8, 0.7, 0.6).into());
    data.material = material.clone();

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..Default::default()
        })
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material,
            transform: Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
            ..Default::default()
        })
        .with(Cube)
        .spawn(LightBundle {
            transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
            ..Default::default()
        })
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(-2.0, 2.5, 5.0))
                .looking_at(Vec3::default(), Vec3::unit_y()),
            ..Default::default()
        });
}
