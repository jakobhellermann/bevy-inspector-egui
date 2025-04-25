use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{EguiContext, EguiContextPass, EguiPlugin};
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_window::PrimaryWindow;

#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct Configuration {
    name: String,
    #[inspector(min = 0.0, max = 1.0)]
    option: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // if you don't use the `quick` plugins you need to add the `EguiPlugin` and the default inspector settings yourself
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(DefaultInspectorConfigPlugin)
        // insert and register resource
        .init_resource::<Configuration>()
        .register_type::<Configuration>()
        .add_systems(Startup, setup)
        // add the system showing the UI
        .add_systems(EguiContextPass, inspector_ui)
        .run();
}

fn inspector_ui(world: &mut World, mut disabled: Local<bool>) {
    let space_pressed = world
        .resource::<ButtonInput<KeyCode>>()
        .just_pressed(KeyCode::Space);
    if space_pressed {
        *disabled = !*disabled;
    }
    if *disabled {
        return;
    }

    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
    else {
        return;
    };
    let mut ctx = egui_context.clone();
    // the usual `ResourceInspector` code
    egui::Window::new("Resource Inspector").show(ctx.get_mut(), |ui| {
        egui::ScrollArea::both().show(ui, |ui| {
            bevy_inspector_egui::bevy_inspector::ui_for_resource::<Configuration>(world, ui);

            ui.separator();
            ui.label("Press space to toggle");
        });
    });
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_xyz(0.0, 0.5, 0.0),
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
