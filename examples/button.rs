use bevy::prelude::*;
use bevy_inspector_egui::{widgets::InspectableButton, Context, Inspectable, InspectorPlugin};

fn handler(data: &mut Data, _context: &Context) {
    dbg!();
}

#[derive(Inspectable)]
struct Data {
    // #[inspectable(on_click = handler)]
    button: InspectableButton,
}
impl Default for Data {
    fn default() -> Self {
        Data {
            button: InspectableButton::new(handler),
        }
    }
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<Data>::new())
        .add_startup_system(setup.system())
        // .add_system(update.system())
        .run();
}

/*fn update(data: Res<Data>, mut query: Query<(&Cube, &mut Transform)>) {
    for (_, mut transform) in query.iter_mut() {
        *transform = data.transform;
    }
}*/

fn setup(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..Default::default()
        })
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
            ..Default::default()
        })
        .spawn(LightBundle {
            transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
            ..Default::default()
        })
        .spawn(PerspectiveCameraBundle {
            transform: Transform::from_translation(Vec3::new(-2.0, 2.5, 5.0))
                .looking_at(Vec3::default(), Vec3::unit_y()),
            ..Default::default()
        });
}
