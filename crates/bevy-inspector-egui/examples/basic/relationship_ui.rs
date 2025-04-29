use bevy::prelude::*;
use bevy_egui::{EguiContext, EguiContextPass, EguiPlugin};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_window::PrimaryWindow;
use egui::{Align2, Vec2};

#[derive(Component, Debug, Reflect)]
#[relationship(relationship_target = Container)]
struct ContainedIn(Entity);

#[derive(Component, Debug, Reflect)]
#[relationship_target(relationship = ContainedIn)]
struct Container(Vec<Entity>);

fn main() {
    App::new()
        .register_type::<ContainedIn>()
        .register_type::<Container>()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(DefaultInspectorConfigPlugin)
        .add_systems(Startup, setup)
        .add_systems(EguiContextPass, show_ui_system)
        .run();
}

fn show_ui_system(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
    else {
        return;
    };
    let mut egui_context: EguiContext = egui_context.clone();

    egui::Window::new("Entities in Children hierarchy")
        .anchor(Align2::LEFT_TOP, Vec2::new(10., 10.))
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                bevy_inspector_egui::bevy_inspector::ui_for_entities_with_relationship::<Children>(
                    world, ui, false,
                );
            });
        });

    egui::Window::new("Container hierarchy entities only")
        .anchor(Align2::CENTER_TOP, Vec2::new(0., 10.))
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                bevy_inspector_egui::bevy_inspector::ui_for_entities_with_relationship::<Container>(
                    world, ui, true,
                );
            });
        });

    egui::Window::new("All entities in Container hierarchy")
        .anchor(Align2::RIGHT_TOP, Vec2::new(-10., 10.))
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                bevy_inspector_egui::bevy_inspector::ui_for_entities_with_relationship::<Container>(
                    world, ui, false,
                );
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
    ));
    // cube
    let cube = commands
        .spawn((
            Name::new("My Cube"),
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(Color::srgba(255., 181. / 255., 0., 102. / 255.))),
            Transform::from_xyz(0.0, 0.5, 0.0),
            children![(
                Name::new("Cube Nose"),
                Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.1))),
                MeshMaterial3d(materials.add(Color::srgba(55., 51. / 255., 0., 52. / 255.))),
                Transform::from_xyz(-0.5, 0.0, 0.0),
            )],
        ))
        .id();

    // sword
    commands.spawn((
        Name::new("Sword"),
        Mesh3d(meshes.add(Cuboid::new(0.85, 0.05, 0.05))),
        MeshMaterial3d(materials.add(Color::srgba(
            20. / 255.,
            20. / 255.,
            20. / 255.,
            200. / 255.,
        ))),
        Transform::from_xyz(-0.3, 0.5, -0.2),
        ContainedIn(cube),
    ));

    // shield
    let shield = commands
        .spawn((
            Name::new("Shield"),
            Mesh3d(meshes.add(Cuboid::new(0.85, 0.8, 0.05))),
            MeshMaterial3d(materials.add(Color::srgba(
                20. / 255.,
                20. / 255.,
                20. / 255.,
                200. / 255.,
            ))),
            Transform::from_xyz(0.0, 0.5, 0.7),
            ContainedIn(cube),
        ))
        .id();

    // hidden letter
    commands.spawn((Name::new("Hidden Letter"), ContainedIn(shield)));

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
