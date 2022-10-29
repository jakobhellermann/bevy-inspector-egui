use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::{
    bevy_ecs_inspector::hierarchy::SelectedEntities, DefaultInspectorConfigPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(DefaultInspectorConfigPlugin)
        .add_startup_system(setup)
        .add_system(rotator_system)
        .add_system(inspector_ui)
        .run();
}

fn inspector_ui(
    world: &mut World,
    mut selected_entities: Local<SelectedEntities>,
    mut inactive: Local<bool>,
) {
    let input = world.resource::<Input<KeyCode>>();
    if input.just_pressed(KeyCode::Escape) {
        *inactive = !*inactive;
    }

    if *inactive {
        return;
    }

    let egui_context = world
        .resource_mut::<bevy_egui::EguiContext>()
        .ctx_mut()
        .clone();

    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = type_registry.read();

    egui::SidePanel::left("hierarchy")
        .default_width(200.0)
        .show(&egui_context, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Hierarchy");

                bevy_inspector_egui::bevy_ecs_inspector::hierarchy::hierarchy_ui(
                    world,
                    &type_registry,
                    ui,
                    &mut *selected_entities,
                );

                ui.label("Press escape to toggle UI");
                ui.allocate_space(ui.available_size());
            });
        });

    egui::SidePanel::right("inspector")
        .default_width(250.0)
        .show(&egui_context, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Inspector");

                let in_header = selected_entities.len() > 1;
                for entity in selected_entities.iter() {
                    bevy_inspector_egui::bevy_ecs_inspector::ui_for_entity(
                        world,
                        entity,
                        ui,
                        egui::Id::new(entity),
                        &type_registry,
                        in_header,
                    );
                }

                ui.allocate_space(ui.available_size());
            });
        });
}

#[derive(Component)]
struct Rotator;

fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<Rotator>>) {
    for mut transform in &mut query {
        transform.rotate_x(3.0 * time.delta_seconds());
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_handle = meshes.add(Mesh::from(shape::Cube { size: 2.0 }));
    let cube_material_handle = materials.add(StandardMaterial {
        base_color: Color::rgb(0.8, 0.7, 0.6),
        ..default()
    });

    // parent cube
    commands
        .spawn((
            PbrBundle {
                mesh: cube_handle.clone(),
                material: cube_material_handle.clone(),
                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                ..default()
            },
            Rotator,
        ))
        .with_children(|parent| {
            // child cube
            parent.spawn(PbrBundle {
                mesh: cube_handle,
                material: cube_material_handle,
                transform: Transform::from_xyz(0.0, 0.0, 3.0),
                ..default()
            });
        });
    // light
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(4.0, 5.0, -4.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(5.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
