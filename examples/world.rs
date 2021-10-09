use bevy::prelude::*;
use bevy_inspector_egui::RegisterInspectable;
use bevy_inspector_egui::{widgets::ResourceInspector, Inspectable, InspectorPlugin};
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .insert_resource(WorldInspectorParams {
            despawnable_entities: true,
            highlight_changes: true,
            ..Default::default()
        })
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(InspectorPlugin::<Resources>::new())
        .register_type::<MyReflectedComponent>()
        .register_inspectable::<MyInspectableComponent>()
        .add_startup_system(setup.system())
        .add_system(movement.system())
        .run();
}

#[derive(Inspectable, Default)]
struct Resources {
    ambient_light: ResourceInspector<bevy::pbr::AmbientLight>,
    clear_color: ResourceInspector<ClearColor>,
}

#[derive(Component, Inspectable, Default)]
pub struct MyInspectableComponent {
    foo: f32,
    bar: usize,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MyReflectedComponent {
    str: String,
    list: Vec<f32>,
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn_bundle((
        MyInspectableComponent::default(),
        MyReflectedComponent {
            str: "str".to_string(),
            list: vec![2.0],
        },
        Name::new("Custom components"),
    ));
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_matrix(Mat4::face_toward(
                Vec3::new(-3.0, 5.0, 8.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            )),
            ..Default::default()
        })
        .insert(Name::new("Camera"));
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
            material: materials.add(Color::rgb_u8(80, 233, 54).into()),
            ..Default::default()
        })
        .insert(Name::new("Floor"));
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            ..Default::default()
        })
        .insert(Name::new("Cube"))
        .with_children(|commands| {
            commands
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
                    transform: Transform::from_xyz(0.0, 0.8, 0.0),
                    material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                    ..Default::default()
                })
                .insert(TeleportTarget)
                .insert(Name::new("Child"))
                .with_children(|commands| {
                    commands
                        .spawn_bundle(PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.2 })),
                            transform: Transform::from_xyz(0.0, 0.4, 0.0),
                            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                            ..Default::default()
                        })
                        .insert(Name::new("Child"));
                });
        });
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                subdivisions: 20,
                radius: 0.5,
            })),
            transform: Transform::from_xyz(1.5, 1.5, 1.5),
            material: materials.add(Color::RED.into()),
            ..Default::default()
        })
        .insert(RotateTarget)
        .insert(Name::new("Sphere"));
    commands
        .spawn_bundle(PointLightBundle {
            transform: Transform::from_xyz(10.3, 8.0, -2.3),
            point_light: PointLight {
                range: 20.0,
                intensity: 1237.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Name::new("Light"));
    commands
        .spawn_bundle(PointLightBundle {
            transform: Transform::from_xyz(-6.2, 8.0, 4.3),
            point_light: PointLight {
                range: 20.0,
                intensity: 245.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Name::new("Second Light"));
}

struct RotateTarget;

struct TeleportTarget;

struct TeleportState {
    timer: Timer,
    count: u32,
}

impl Default for TeleportState {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, true),
            count: 0,
        }
    }
}

/// simple movement in the scene
fn movement(
    time: Res<Time>,
    mut state: Local<TeleportState>,
    mut qs: QuerySet<(
        Query<&mut Transform, With<RotateTarget>>,
        Query<&mut Transform, With<TeleportTarget>>,
    )>,
) {
    for mut transform in qs.q0_mut().iter_mut() {
        // rotate around vertical axis through origin
        transform.translation =
            Quat::from_axis_angle(Vec3::Y, time.delta_seconds()) * transform.translation;
    }
    if state.timer.tick(time.delta()).just_finished() {
        for mut transform in qs.q1_mut().iter_mut() {
            // jump to new position to the left or right
            transform.translation += match state.count % 4 {
                0 | 3 => -Vec3::X,
                _ => Vec3::X,
            };
        }
        state.count += 1;
    }
}
