use bevy::{
    prelude::*,
    state::{
        app::AppExtStates,
        state::{OnEnter, States},
    },
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::StateInspectorPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .insert_resource(ClearColor(Color::BLACK))
        .init_state::<AppState>()
        .register_type::<AppState>()
        .add_plugins(StateInspectorPlugin::<AppState>::default())
        .add_systems(OnEnter(AppState::A), set_color::<158, 228, 147>)
        .add_systems(OnEnter(AppState::B), set_color::<172, 200, 192>)
        .add_systems(OnEnter(AppState::C), set_color::<194, 148, 138>)
        .run();
}

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash, Reflect)]
enum AppState {
    #[default]
    A,
    B,
    C,
}

#[derive(Component)]
struct TheSquare;

fn set_color<const R: u8, const G: u8, const B: u8>(
    mut sprite: Query<&mut Sprite, With<TheSquare>>,
    mut commands: Commands,
) {
    let color = Color::srgb_u8(R, G, B);
    if let Ok(mut sprite) = sprite.single_mut() {
        sprite.color = color;
    } else {
        commands.spawn(Camera2d);

        commands.spawn((
            Sprite {
                custom_size: Some(Vec2::splat(100.)),
                color,
                ..default()
            },
            TheSquare,
        ));
    }
}
