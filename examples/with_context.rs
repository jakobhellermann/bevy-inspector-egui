use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};

// #[derive(Inspectable, Debug)]
#[derive(Inspectable, Debug)]
struct Data {
    material: Handle<StandardMaterial>,
}

impl FromResources for Data {
    fn from_resources(resources: &Resources) -> Self {
        let mut materials = resources.get_mut::<Assets<StandardMaterial>>().unwrap();
        let asset_server = resources.get::<AssetServer>().unwrap();

        let texture_handle = asset_server.load("texture-128.png");
        let material = materials.add(StandardMaterial {
            albedo: Color::WHITE,
            albedo_texture: Some(texture_handle),
            ..Default::default()
        });

        Data { material }
    }
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<Data>::new())
        .add_startup_system(setup.system())
        .run();
}

struct Cube;

fn setup(
    commands: &mut Commands,
    data: ResMut<Data>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..Default::default()
        })
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: data.material.clone(),
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
