use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::{bevy_ecs_inspector::ui_for_state, DefaultInspectorConfigPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .add_state(AppState::A)
        .add_plugin(EguiPlugin)
        .add_plugin(DefaultInspectorConfigPlugin)
        // alternatively, just add
        // .add_plugin(bevy_inspector_egui::quick::StateInspectorPlugin::<AppState>::default())
        .add_startup_system(setup)
        .add_system(ui)
        .add_system_set(SystemSet::on_enter(AppState::A).with_system(set_color::<158, 228, 147>))
        .add_system_set(SystemSet::on_enter(AppState::B).with_system(set_color::<171, 200, 192>))
        .add_system_set(SystemSet::on_enter(AppState::C).with_system(set_color::<194, 148, 138>))
        .register_type::<AppState>()
        .run();
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Reflect)]
enum AppState {
    A,
    B,
    C,
}

fn ui(world: &mut World) {
    let egui_context = world
        .resource_mut::<bevy_egui::EguiContext>()
        .ctx_mut()
        .clone();

    let type_registry = world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    egui::Window::new("UI")
        .title_bar(false)
        .resizable(false)
        .show(&egui_context, |ui| {
            ui.heading("AppState");
            ui_for_state::<AppState>(world, ui, &type_registry);
        });
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
