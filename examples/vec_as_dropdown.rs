use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};
use std::fmt::{Debug, Display};
pub struct VecAsDropdown<T>
where
    T: Clone + Display + PartialEq,
{
    from: Vec<T>,
    selected: usize,
}

impl<T> VecAsDropdown<T>
where
    T: Clone + Display + PartialEq,
{
    pub fn new(from_vec: Vec<T>) -> Self {
        Self {
            from: from_vec,
            selected: 0,
        }
    }

    pub fn selected_value(&self) -> T {
        self.from[self.selected].clone()
    }
}

impl<T> Default for VecAsDropdown<T>
where
    T: Clone + Display + PartialEq,
{
    fn default() -> Self {
        Self {
            from: Vec::new(),
            selected: 0,
        }
    }
}

impl<T> Inspectable for VecAsDropdown<T>
where
    T: Clone + Display + PartialEq + Debug + Default,
{
    type Attributes = Vec<T>;

    fn ui(
        &mut self,
        ui: &mut bevy_inspector_egui::egui::Ui,
        _: Self::Attributes,
        _: &bevy_inspector_egui::Context,
    ) -> bool {
        let mut display = T::default();
        if !self.from.is_empty() {
            display = self.from[self.selected].clone();
        }
        let hash = format!("{:?}", self.from);

        bevy_inspector_egui::egui::ComboBox::from_id_source(hash)
            .selected_text(format!("{}", display))
            .show_ui(ui, |ui| {
                for (index, value) in self.from.iter().enumerate() {
                    ui.selectable_value(&mut self.selected, index, format!("{}", value));
                }
            });
        true
    }

    fn setup(_: &mut App) {
        // eprintln!("Running setup code...");

        // app.init_resource::<WhateverYouNeed>();
    }
}

#[derive(Inspectable)]
struct Data {
    pub vec_of_ints: VecAsDropdown<u32>,
    pub vec_of_strings: VecAsDropdown<String>,
}

impl Default for Data {
    fn default() -> Self {
        let vec_of_ints = VecAsDropdown::new(vec![100, 200, 300, 400]);
        let vec_of_strings = VecAsDropdown::new(
            vec!["Some", "Thing", "And Another"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        );
        Data {
            vec_of_ints,
            vec_of_strings,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<Data>::new())
        .add_startup_system(setup.system())
        .run();
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let color = materials.add(Color::BLUE.into());

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
    commands.spawn_bundle(SpriteBundle {
        material: color,
        sprite: Sprite {
            size: Vec2::new(40.0, 40.0),
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::default()),
        ..Default::default()
    });
}
