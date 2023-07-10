use bevy::prelude::*;
use bevy_egui::EguiContext;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
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
        .add_plugins(EguiPlugin)
        .add_plugins(DefaultInspectorConfigPlugin)
        // insert and register resource
        .init_resource::<Configuration>()
        .register_type::<Configuration>()
        .add_systems(Startup, setup)
        // add the system showing the UI
        .add_systems(Update, inspector_ui)
        .run();
}

fn inspector_ui(world: &mut World, mut disabled: Local<bool>) {
    let space_pressed = world
        .resource::<Input<KeyCode>>()
        .just_pressed(KeyCode::Space);
    if space_pressed {
        *disabled = !*disabled;
    }
    if *disabled {
        return;
    }

    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();

    // the usual `ResourceInspector` code
    egui::Window::new("Resource Inspector").show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
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
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
