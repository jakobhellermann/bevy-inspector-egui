use bevy::prelude::*;
use bevy_inspector_egui::quick::StateInspectorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .add_state(AppState::A)
        .register_type::<AppState>()
        .add_plugin(StateInspectorPlugin::<AppState>::default())
        .add_startup_system(setup)
        .add_system_set(SystemSet::on_enter(AppState::A).with_system(set_color::<158, 228, 147>))
        .add_system_set(SystemSet::on_enter(AppState::B).with_system(set_color::<171, 200, 192>))
        .add_system_set(SystemSet::on_enter(AppState::C).with_system(set_color::<194, 148, 138>))
        .run();
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Reflect)]
enum AppState {
    A,
    B,
    C,
}

#[derive(Component)]
struct TheSquare;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::splat(100.)),
                ..default()
            },
            ..default()
        })
        .insert(TheSquare);
}

fn set_color<const R: u8, const G: u8, const B: u8>(
    mut sprite: Query<&mut Sprite, With<TheSquare>>,
) {
    sprite.single_mut().color = Color::rgb_u8(R, G, B);
}
