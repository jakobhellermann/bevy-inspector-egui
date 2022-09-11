use std::collections::HashSet;

use bevy_ecs::prelude::*;
use bevy_hierarchy::{Children, Parent};
use bevy_reflect::TypeRegistry;
use egui::{CollapsingHeader, RichText};

use super::guess_entity_name;

pub fn hierarchy_ui(
    world: &mut World,
    type_registry: &TypeRegistry,
    ui: &mut egui::Ui,
    selected: &mut SelectedEntities,
) {
    Hierarchy {
        world,
        type_registry,
        selected,
    }
    .show(ui);
}

struct Hierarchy<'a> {
    world: &'a mut World,
    type_registry: &'a TypeRegistry,
    selected: &'a mut SelectedEntities,
}

impl Hierarchy<'_> {
    fn show(&mut self, ui: &mut egui::Ui) {
        let mut root_query = self.world.query_filtered::<Entity, (Without<Parent>,)>();

        let always_open: HashSet<Entity> = self
            .selected
            .iter()
            .flat_map(|selected| {
                std::iter::successors(Some(selected), |&entity| {
                    self.world.get::<Parent>(entity).map(|parent| parent.get())
                })
                .skip(1)
            })
            .collect();

        let mut entities: Vec<_> = root_query.iter(self.world).collect();
        entities.sort();

        for &entity in &entities {
            self.entity_ui(ui, entity, &always_open, &entities);
        }
    }

    fn entity_ui(
        &mut self,
        ui: &mut egui::Ui,
        entity: Entity,
        always_open: &HashSet<Entity>,
        at_same_level: &[Entity],
    ) {
        let selected = self.selected.contains(entity);

        let entity_name = guess_entity_name::entity_name(self.world, self.type_registry, entity);
        let mut name = RichText::new(entity_name);
        if selected {
            name = name.strong();
        }

        let has_children = self
            .world
            .get::<Children>(entity)
            .map_or(false, |children| children.len() > 0);

        let open = if !has_children {
            Some(false)
        } else if always_open.contains(&entity) {
            Some(true)
        } else {
            None
        };

        #[allow(deprecated)] // the suggested replacement doesn't really work
        let response = CollapsingHeader::new(name)
            .id_source(entity)
            .icon(move |ui, openness, response| {
                if !has_children {
                    return;
                }
                paint_default_icon(ui, openness, response);
            })
            .selectable(true)
            .selected(selected)
            .open(open)
            .show(ui, |ui| {
                let children = self.world.get::<Children>(entity);
                if let Some(children) = children {
                    let children = children.to_vec();
                    for &child in children.iter() {
                        self.entity_ui(ui, child, always_open, &children);
                    }
                } else {
                    ui.label("No children");
                }
            });
        let header_response = response.header_response;

        if header_response.clicked() {
            let selection_mode = SelectionMode::from_ctrl_shift(
                ui.input().modifiers.ctrl,
                ui.input().modifiers.shift,
            );
            let extend_with = |from, to| {
                // PERF: this could be done in one scan
                let from_position = at_same_level.iter().position(|&entity| entity == from);
                let to_position = at_same_level.iter().position(|&entity| entity == to);
                from_position
                    .zip(to_position)
                    .map(|(from, to)| {
                        let (min, max) = if from < to { (from, to) } else { (to, from) };
                        at_same_level[min..=max].iter().copied()
                    })
                    .into_iter()
                    .flatten()
            };
            self.selected.select(selection_mode, entity, extend_with);
        }
    }
}

fn paint_default_icon(ui: &mut egui::Ui, openness: f32, response: &egui::Response) {
    let visuals = ui.style().interact(response);
    let stroke = visuals.fg_stroke;

    let rect = response.rect;

    // Draw a pointy triangle arrow:
    let rect = egui::Rect::from_center_size(
        rect.center(),
        egui::vec2(rect.width(), rect.height()) * 0.75,
    );
    let rect = rect.expand(visuals.expansion);
    let mut points = vec![rect.left_top(), rect.right_top(), rect.center_bottom()];
    use std::f32::consts::TAU;
    let rotation =
        egui::emath::Rot2::from_angle(egui::remap(openness, 0.0..=1.0, -TAU / 4.0..=0.0));
    for p in &mut points {
        *p = rect.center() + rotation * (*p - rect.center());
    }

    ui.painter().add(egui::Shape::closed_line(points, stroke));
}

#[derive(Default)]
pub struct SelectedEntities {
    entities: Vec<Entity>,
    last_action: Option<(SelectionMode, Entity)>,
}

#[derive(Clone, Copy)]
pub enum SelectionMode {
    Replace,
    Add,
    Extend,
}

impl SelectionMode {
    pub fn from_ctrl_shift(ctrl: bool, shift: bool) -> SelectionMode {
        match (ctrl, shift) {
            (true, _) => SelectionMode::Add,
            (false, true) => SelectionMode::Extend,
            (false, false) => SelectionMode::Replace,
        }
    }
}

impl SelectedEntities {
    pub fn select<I: IntoIterator<Item = Entity>>(
        &mut self,
        mode: SelectionMode,
        entity: Entity,
        extend_with: impl Fn(Entity, Entity) -> I,
    ) {
        match (self.len(), mode) {
            (0, _) => {
                self.insert(entity);
            }
            (_, SelectionMode::Replace) => {
                self.insert_replace(entity);
            }
            (_, SelectionMode::Add) => {
                self.toggle(entity);
            }
            (_, SelectionMode::Extend) => {
                match self.last_action {
                    None => self.insert(entity),
                    Some((last_mode, last_entity)) => {
                        if let SelectionMode::Add | SelectionMode::Replace = last_mode {
                            self.clear()
                        }
                        for entity in extend_with(entity, last_entity) {
                            self.insert(entity);
                        }

                        // extending doesn't update last action
                        return;
                    }
                };
            }
        }
        self.last_action = Some((mode, entity));
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.entities.contains(&entity)
    }
    fn insert(&mut self, entity: Entity) {
        if !self.contains(entity) {
            self.entities.push(entity);
        }
    }

    fn insert_replace(&mut self, entity: Entity) {
        self.entities.clear();
        self.entities.push(entity);
    }

    fn toggle(&mut self, entity: Entity) {
        if self.remove(entity).is_none() {
            self.entities.push(entity);
        }
    }

    pub fn remove(&mut self, entity: Entity) -> Option<Entity> {
        if let Some(idx) = self.entities.iter().position(|&e| e == entity) {
            Some(self.entities.remove(idx))
        } else {
            None
        }
    }

    pub fn last_action(&self) -> Option<(SelectionMode, Entity)> {
        self.last_action
    }

    pub fn clear(&mut self) {
        self.entities.clear();
    }
    pub fn retain(&mut self, f: impl Fn(Entity) -> bool) {
        self.entities.retain(|entity| f(*entity));
    }
    pub fn len(&self) -> usize {
        self.entities.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entities.len() == 0
    }
    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entities.iter().copied()
    }
}
