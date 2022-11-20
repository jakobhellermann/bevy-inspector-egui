use bevy::prelude::*;
use bevy::render::mesh::shape;
use bevy_inspector_egui::{widgets::InspectorQuerySingle, InspectorPlugin};
use bevy_inspector_egui_rapier::InspectableRapierPlugin;
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(InspectorPlugin::<InspectorQuerySingle<Entity, With<Cube>>>::new())
        .add_plugin(InspectableRapierPlugin)
        .add_startup_system(setup)
        .run();
}

#[derive(Component)]
struct Cube;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_size = 1.0;
    let floor_size = 20.0;

    let _floor = PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: floor_size })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    };

    commands
        // .spawn(floor)
        .spawn_empty()
        .insert(Name::new("Floor"))
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(floor_size / 2.0, 0.1, floor_size / 2.0));

    let _cube = PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: cube_size })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        ..Default::default()
    };

    commands
        // .spawn(cube)
        .spawn_empty()
        .insert(Cube)
        .insert(Name::new("Cube"))
        .insert(Collider::cuboid(
            cube_size / 2.0,
            cube_size / 2.0,
            cube_size / 2.0,
        ))
        .insert(RigidBody::Dynamic)
        .insert(CollisionGroups::default())
        .insert(SolverGroups::default())
        .insert(Transform::from_xyz(0.0, 2.0, 0.0));

    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::default(), Vec3::Y),
        ..Default::default()
    });
}
