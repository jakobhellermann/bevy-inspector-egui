use std::collections::HashMap;

use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};

#[derive(Inspectable)]
enum TextColor {
    White,
    Green,
    Blue,
}

#[derive(Resource, Inspectable)]
struct Data {
    #[inspectable(min = 10.0, max = 70.0)]
    font_size: f32,
    text: String,
    show_square: bool,
    text_color: TextColor,
    color: Color,
    #[inspectable(visual, min = Vec2::new(-200., -200.), max = Vec2::new(200., 200.))]
    position: Vec2,
    list: Vec<f32>,
    hash: HashMap<String, String>,
    #[inspectable(replacement = String::default as fn() -> _)]
    option: Option<String>,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            font_size: 50.0,
            text: "Hello World!".to_string(),
            show_square: true,
            text_color: TextColor::White,
            color: Color::BLUE,
            position: Vec2::default(),
            list: vec![0.0],
            hash: HashMap::from([
                ("one".to_string(), "two".to_string()),
                ("three".to_string(), "four".to_string()),
            ]),
            option: None,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<Data>::new())
        .add_startup_system(setup)
        .add_system(text_update_system)
        .add_system(shape_update_system)
        .run();
}

fn text_update_system(data: Res<Data>, mut query: Query<&mut Text>) {
    for mut text in query.iter_mut() {
        let text = &mut text.sections[0];
        text.value = data.text.clone();
        text.style.font_size = data.font_size;
        text.style.color = match &data.text_color {
            TextColor::White => Color::WHITE,
            TextColor::Green => Color::GREEN,
            TextColor::Blue => Color::BLUE,
        };
    }
}

fn shape_update_system(data: Res<Data>, mut query: Query<(&mut Sprite, &mut Transform)>) {
    for (mut sprite, mut transfrom) in query.iter_mut() {
        sprite.color = data.color;

        if !data.show_square {
            transfrom.translation.x = 1000000.0;
        } else {
            transfrom.translation.x = data.position.x;
            transfrom.translation.y = data.position.y;
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_handle = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands.spawn(Camera2dBundle::default());
    commands.spawn(TextBundle {
        style: Style {
            align_self: AlignSelf::FlexEnd,
            ..Default::default()
        },
        text: Text::from_section(
            "",
            TextStyle {
                font_size: 50.0,
                font: font_handle,
                ..Default::default()
            },
        ),
        ..Default::default()
    });
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(40.0, 40.0)),
            color: Color::BLUE,
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::default()),
        ..Default::default()
    });
}
