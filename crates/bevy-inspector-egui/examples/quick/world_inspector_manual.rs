use bevy::prelude::*;
use bevy_egui::{EguiContext, EguiPrimaryContextPass, PrimaryEguiContext};
use bevy_inspector_egui::{DefaultInspectorConfigPlugin, bevy_egui::EguiPlugin, bevy_inspector};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(DefaultInspectorConfigPlugin)
        .add_systems(EguiPrimaryContextPass, custom_world_inspector_ui)
        .add_systems(Startup, setup)
        .run();
}

fn custom_world_inspector_ui(world: &mut World) {
    let egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single(world);

    let Ok(egui_context) = egui_context else {
        return;
    };
    let mut egui_context = egui_context.clone();

    egui::Window::new("World Inspector")
        .default_size((400., 300.))
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                bevy_inspector::ui_for_world(world, ui);
                ui.allocate_space(ui.available_size());
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
        Name::new("Plane"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));
    // cube
    commands.spawn((
        Name::new("My Cube"),
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgba(255., 181. / 255., 0., 102. / 255.))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    // light
    commands.spawn((
        Name::new("Light"),
        PointLight {
            intensity: 2_000_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // camera
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
