use bevy::prelude::*;
use bevy_inspector_egui::{options::QuatDisplay, Inspectable, InspectorPlugin};

#[derive(Inspectable, Default)]
struct Data {
    #[inspectable(display = QuatDisplay::Euler)]
    euler: Quat,
    #[inspectable(display = QuatDisplay::YawPitchRoll)]
    ypr: Quat,
    #[inspectable(display = QuatDisplay::AxisAngle)]
    axis_angle: Quat,
    #[inspectable(display = QuatDisplay::Raw)]
    raw: Quat,
    which_one: WhichOne,
}

#[derive(Inspectable)]
enum WhichOne {
    Euler,
    YawPitchRoll,
    AxisAngle,
    Raw,
}
impl Default for WhichOne {
    fn default() -> Self {
        WhichOne::Euler
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<Data>::new())
        .add_startup_system(setup)
        .add_system(update)
        .run();
}

fn update(data: Res<Data>, mut query: Query<(&Cube, &mut Transform)>) {
    for (_, mut transform) in query.iter_mut() {
        match data.which_one {
            WhichOne::Euler => transform.rotation = data.euler,
            WhichOne::Raw => transform.rotation = data.raw,
            WhichOne::YawPitchRoll => transform.rotation = data.ypr,
            WhichOne::AxisAngle => transform.rotation = data.axis_angle,
        }
    }
}

#[derive(Component)]
struct Cube;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        })
        .insert(Cube);
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}
