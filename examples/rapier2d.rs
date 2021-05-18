use bevy::prelude::*;
use bevy_inspector_egui::{widgets::InspectorQuery, InspectorPlugin};
use bevy_rapier2d::physics::{RapierConfiguration, RapierPhysicsPlugin, RigidBodyHandleComponent};
use bevy_rapier2d::rapier::dynamics::RigidBodyBuilder;
use bevy_rapier2d::rapier::geometry::ColliderBuilder;
use bevy_rapier2d::rapier::na::Vector2;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<
            InspectorQuery<&'static mut RigidBodyHandleComponent>,
        >::new())
        .add_plugin(RapierPhysicsPlugin)
        .add_startup_system(spawn_player.system())
        .run();
}

// The float value is the player movemnt speed in 'pixels/second'.
struct Player(f32);

fn spawn_player(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    // Set gravity to 0.0 and spawn camera.
    rapier_config.gravity = Vector2::zeros();
    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d());

    let sprite_size_x = 40.0;
    let sprite_size_y = 40.0;

    // While we want our sprite to look ~40 px square, we want to keep the physics units smaller
    // to prevent float rounding problems. To do this, we set the scale factor in RapierConfiguration
    // and divide our sprite_size by the scale.
    rapier_config.scale = 20.0;
    let collider_size_x = sprite_size_x / rapier_config.scale;
    let collider_size_y = sprite_size_y / rapier_config.scale;

    // Spawn entity with `Player` struct as a component for access in movement query.
    commands
        .spawn()
        .insert_bundle(SpriteBundle {
            material: materials.add(Color::rgb(0.0, 0.0, 0.0).into()),
            sprite: Sprite::new(Vec2::new(sprite_size_x, sprite_size_y)),
            ..Default::default()
        })
        .insert(RigidBodyBuilder::new_dynamic())
        .insert(ColliderBuilder::cuboid(
            collider_size_x / 2.0,
            collider_size_y / 2.0,
        ))
        .insert(Player(300.0));
}
