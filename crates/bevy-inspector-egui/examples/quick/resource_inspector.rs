use bevy::{
    input::common_conditions::input_toggle_active, platform::collections::HashSet, prelude::*,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, prelude::*, quick::ResourceInspectorPlugin};

#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct Configuration {
    name: String,
    #[inspector(min = 0.0, max = 1.0)]
    option: f32,
    set: HashSet<String>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(
            ResourceInspectorPlugin::<GizmoConfigStore>::default()
                .run_if(input_toggle_active(true, KeyCode::Escape)),
        )
        //.init_resource::<Configuration>()
        .insert_resource(Configuration {
            set: [
                "Einar".to_string(),
                "Olaf".to_string(),
                "Harald".to_string(),
            ]
            .into_iter()
            .collect(),
            ..default()
        })
        .register_type::<Configuration>()
        .add_systems(Startup, setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(0.5, 0.5))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    // light
    commands.spawn((
        PointLight {
            intensity: 2_000_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
