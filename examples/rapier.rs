use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};
use bevy_rapier3d::physics::{RapierPhysicsPlugin, RigidBodyHandleComponent};
use bevy_rapier3d::rapier::{dynamics::RigidBodyBuilder, geometry::ColliderBuilder};

#[derive(Inspectable, Default)]
struct Data {
    #[inspectable(deletable = false)]
    handle: Option<RigidBodyHandleComponent>,
}

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<Data>::new())
        .add_plugin(RapierPhysicsPlugin)
        .add_startup_system(setup.system())
        .add_system(set_rigidbody_handle.system())
        .run();
}

fn set_rigidbody_handle(
    mut data: ResMut<Data>,
    query: Query<(&Cube, &RigidBodyHandleComponent), Added<RigidBodyHandleComponent>>,
) {
    for (_, handle) in query.iter() {
        if data.handle.is_none() {
            data.handle = Some(handle.handle().into());
        }
    }
}

struct Cube;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_size = 1.0;
    let floor_size = 6.0;

    let floor = PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: floor_size })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    };

    // Static rigid-body with a cuboid shape.
    let rigid_body1 = RigidBodyBuilder::new_static();
    let collider1 = ColliderBuilder::cuboid(floor_size / 2.0, 0.1, floor_size / 2.0);
    commands
        .spawn_bundle((rigid_body1, collider1))
        .insert_bundle(floor);

    let cube = PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: cube_size })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        ..Default::default()
    };

    // Dynamic rigid-body with ball shape.
    let rigid_body2 = RigidBodyBuilder::new_dynamic().translation(0.0, 3.0, 0.0);
    let collider2 = ColliderBuilder::cuboid(cube_size / 2.0, cube_size / 2.0, cube_size / 2.0);
    commands
        .spawn_bundle((rigid_body2, collider2, Cube))
        .insert_bundle(cube);

    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::default(), Vec3::Y),
        ..Default::default()
    });
}
