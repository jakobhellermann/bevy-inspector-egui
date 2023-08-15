use bevy::input::common_conditions::input_toggle_active;
use bevy::input::mouse;
use bevy::prelude::*;
use bevy_inspector_egui::egui_mouse_check;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape)),
        )
        .init_resource::<egui_mouse_check::EguiMousePointerCheck>()
        .add_systems(
            Startup,
            (setup, egui_mouse_check::initialize_egui_mouse_check),
        )
        .add_systems(PreUpdate, egui_mouse_check::update_egui_mouse_check)
        .add_systems(
            Update,
            (camera_pan, camera_zoom).run_if(egui_mouse_check::mouse_pointer_valid()),
        )
        .run();
}

#[derive(Component)]
struct MainCamera;

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane::from_size(5.0))),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn((
        MainCamera,
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
    ));
}

fn camera_pan(
    mouse: Res<Input<MouseButton>>,
    mut motion_evr: EventReader<mouse::MouseMotion>,
    mut camera_q: Query<&mut Transform, With<MainCamera>>,
) {
    const SENSITIVITY: f32 = 0.005;
    let mut camera_transform = camera_q.single_mut();

    if mouse.any_pressed([MouseButton::Left, MouseButton::Right]) {
        for e in motion_evr.iter() {
            let y_rotation = Quat::from_rotation_y(e.delta.x * SENSITIVITY);
            camera_transform.rotate_around(Vec3::ZERO, y_rotation);
        }
    }
}

fn camera_zoom(
    mut scroll_evr: EventReader<mouse::MouseWheel>,
    mut camera_q: Query<&mut Transform, With<MainCamera>>,
) {
    let mut camera_transform = camera_q.single_mut();

    for e in scroll_evr.iter() {
        let dir = -e.y.signum();
        let n = camera_transform.translation.normalize() * dir * 2.;

        camera_transform.translation += n;
        camera_transform
            .translation
            .clamp(Vec3::splat(10.0), Vec3::splat(25.0));
    }
}
