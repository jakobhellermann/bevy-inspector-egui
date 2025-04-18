use std::ops::DerefMut;

use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_inspector_egui::{
    bevy_egui::{EguiContext, EguiContextPass, EguiPlugin},
    bevy_inspector::hierarchy::SelectedEntities,
    DefaultInspectorConfigPlugin,
};
use bevy_window::PrimaryWindow;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(DefaultInspectorConfigPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, rotator_system)
        .add_systems(
            EguiContextPass,
            inspector_ui.run_if(input_toggle_active(true, KeyCode::Escape)),
        )
        .run();
}

fn inspector_ui(world: &mut World, mut selected_entities: Local<SelectedEntities>) {
    let Ok(mut ctx) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single_mut(world)
    else {
        return;
    };

    let mut egui_context = ctx.deref_mut().clone();
    egui::SidePanel::left("hierarchy")
        .default_width(200.0)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.heading("Hierarchy");

                bevy_inspector_egui::bevy_inspector::hierarchy::hierarchy_ui(
                    world,
                    ui,
                    &mut selected_entities,
                );

                ui.label("Press escape to toggle UI");
                ui.allocate_space(ui.available_size());
            });
        });

    egui::SidePanel::right("inspector")
        .default_width(250.0)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.heading("Inspector");

                match selected_entities.as_slice() {
                    &[entity] => {
                        bevy_inspector_egui::bevy_inspector::ui_for_entity(world, entity, ui);
                    }
                    entities => {
                        bevy_inspector_egui::bevy_inspector::ui_for_entities_shared_components(
                            world, entities, ui,
                        );
                    }
                }

                ui.allocate_space(ui.available_size());
            });
        });
}

#[derive(Component)]
struct Rotator;

fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<Rotator>>) {
    for mut transform in &mut query {
        transform.rotate_x(3.0 * time.delta_secs());
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_handle = meshes.add(Cuboid::new(2.0, 2.0, 2.0));
    let cube_material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.7, 0.6),
        ..default()
    });

    // parent cube
    commands
        .spawn((
            Mesh3d(cube_handle.clone()),
            MeshMaterial3d(cube_material_handle.clone()),
            Transform::from_xyz(0.0, 0.0, 1.0),
            Rotator,
        ))
        .with_children(|parent| {
            // child cube
            parent.spawn((
                Mesh3d(cube_handle),
                MeshMaterial3d(cube_material_handle),
                Transform::from_xyz(0.0, 0.0, 3.0),
            ));
        });
    // light
    commands.spawn((PointLight::default(), Transform::from_xyz(4.0, 5.0, -4.0)));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(5.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
