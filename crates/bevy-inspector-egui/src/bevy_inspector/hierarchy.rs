use std::collections::HashSet;

use crate::bevy_inspector::{EntityFilter, Filter};
use crate::utils::guess_entity_name;
use bevy_ecs::{prelude::*, query::QueryFilter};
use egui::{CollapsingHeader, RichText};

/// Display UI of the entity hierarchy.
///
/// Returns `true` if a new entity was selected.
pub fn hierarchy_ui(world: &mut World, ui: &mut egui::Ui, selected: &mut SelectedEntities) -> bool {
    Hierarchy {
        world,
        selected,
        context_menu: None,
        shortcircuit_entity: None,
        extra_state: &mut (),
    }
    .show::<()>(ui)
}

/// Display UI of the entity hierarchy with a [QueryFilter].
///
/// Returns `true` if a new entity was selected.
pub fn hierarchy_ui_filtered<QF>(
    world: &mut World,
    ui: &mut egui::Ui,
    selected: &mut SelectedEntities,
) -> bool
where
    QF: QueryFilter,
{
    Hierarchy {
        world,
        selected,
        context_menu: None,
        shortcircuit_entity: None,
        extra_state: &mut (),
    }
    .show::<QF>(ui)
}

pub struct Hierarchy<'a, T = ()> {
    pub world: &'a mut World,
    pub selected: &'a mut SelectedEntities,
    pub context_menu: Option<&'a mut dyn FnMut(&mut egui::Ui, Entity, &mut World, &mut T)>,
    pub shortcircuit_entity:
        Option<&'a mut dyn FnMut(&mut egui::Ui, Entity, &mut World, &mut T) -> bool>,
    pub extra_state: &'a mut T,
}

impl<T> Hierarchy<'_, T> {
    pub fn default_children_getter() -> impl Fn(&World, Entity) -> Option<Vec<Entity>> {
        |world: &World, entity: Entity| {
            world
                .get::<Children>(entity)
                .map(|children| children.iter().collect::<Vec<_>>())
        }
    }

    pub fn default_parent_getter() -> impl Fn(&World, Entity) -> Option<Entity> {
        |world: &World, entity: Entity| world.get::<ChildOf>(entity).map(|c| c.parent())
    }

    pub fn show<QF>(&mut self, ui: &mut egui::Ui) -> bool
    where
        QF: QueryFilter,
    {
        let filter: Filter = Filter::all();
        self._show::<QF, _, _, _>(
            ui,
            filter,
            Self::default_children_getter(),
            Self::default_parent_getter(),
        )
    }
    pub fn show_with_default_filter<QF>(&mut self, ui: &mut egui::Ui) -> bool
    where
        QF: QueryFilter,
    {
        let filter: Filter = Filter::from_ui(ui, egui::Id::new("default_hierarchy_filter"));
        self._show::<QF, _, _, _>(
            ui,
            filter,
            Self::default_children_getter(),
            Self::default_parent_getter(),
        )
    }
    pub fn show_with_filter<QF, F>(&mut self, ui: &mut egui::Ui, filter: F) -> bool
    where
        QF: QueryFilter,
        F: EntityFilter,
    {
        self._show::<QF, F, _, _>(
            ui,
            filter,
            Self::default_children_getter(),
            Self::default_parent_getter(),
        )
    }

    pub fn show_with_filter_and_getters<QF, F, CG, PG>(
        &mut self,
        ui: &mut egui::Ui,
        filter: F,
        children_getter: CG,
        parent_getter: PG,
    ) -> bool
    where
        QF: QueryFilter,
        F: EntityFilter,
        CG: Fn(&World, Entity) -> Option<Vec<Entity>>,
        PG: Fn(&World, Entity) -> Option<Entity>,
    {
        self._show::<QF, F, CG, PG>(ui, filter, children_getter, parent_getter)
    }

    fn _show<QF, F, CG, PG>(
        &mut self,
        ui: &mut egui::Ui,
        filter: F,
        children_getter: CG,
        parent_getter: PG,
    ) -> bool
    where
        QF: QueryFilter,
        F: EntityFilter,
        CG: Fn(&World, Entity) -> Option<Vec<Entity>>,
        PG: Fn(&World, Entity) -> Option<Entity>,
    {
        let always_open: HashSet<Entity> = self
            .selected
            .iter()
            .flat_map(|selected| {
                std::iter::successors(Some(selected), |&entity| parent_getter(self.world, entity))
                    .skip(1)
            })
            .collect();

        let mut root_query = self.world.query_filtered::<Entity, QF>();
        let mut entities: Vec<_> = root_query
            .iter(self.world)
            .filter(|&entity| parent_getter(self.world, entity).is_none())
            .collect();

        filter.filter_entities(self.world, &mut entities);
        entities.sort();

        let mut selected = false;
        for &entity in &entities {
            selected |= self.entity_ui(
                ui,
                entity,
                &always_open,
                &entities,
                &filter,
                &children_getter,
            );
        }
        selected
    }

    fn entity_ui<F, CG>(
        &mut self,
        ui: &mut egui::Ui,
        entity: Entity,
        always_open: &HashSet<Entity>,
        at_same_level: &[Entity],
        filter: &F,
        get_children: &CG,
    ) -> bool
    where
        F: EntityFilter,
        CG: Fn(&World, Entity) -> Option<Vec<Entity>>,
    {
        let mut new_selection = false;
        let selected = self.selected.contains(entity);

        let entity_name = guess_entity_name::guess_entity_name(self.world, entity);
        let mut name = RichText::new(entity_name);
        if selected {
            name = name.strong();
        }

        let has_children = get_children(self.world, entity)
            .as_ref()
            .is_some_and(|children| !children.is_empty());

        let open = if !has_children {
            Some(false)
        } else if always_open.contains(&entity) {
            Some(true)
        } else {
            None
        };

        if let Some(shortcircuit_entity) = self.shortcircuit_entity.as_mut()
            && shortcircuit_entity(ui, entity, self.world, self.extra_state)
        {
            return false;
        }

        let response = CollapsingHeader::new(name)
            .id_salt(entity)
            .icon(move |ui, openness, response| {
                if !has_children {
                    return;
                }
                paint_default_icon(ui, openness, response);
            })
            .open(open)
            .show(ui, |ui| {
                if let Some(mut children) = get_children(self.world, entity) {
                    filter.filter_entities(self.world, &mut children);
                    for &child in &children {
                        new_selection |=
                            self.entity_ui(ui, child, always_open, &children, filter, get_children);
                    }
                } else {
                    ui.label("No children");
                }
            });
        let header_response = response.header_response;

        if header_response.clicked() {
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
            let selection_mode = ui.input(|input| input.modifiers.into());
            self.selected.select(selection_mode, entity, extend_with);
            new_selection = true;
        }

        if let Some(context_menu) = self.context_menu.as_mut() {
            header_response
                .context_menu(|ui| context_menu(ui, entity, self.world, self.extra_state));
        }

        new_selection
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

/// Collection of currently selected entities
#[derive(Default, Debug)]
pub struct SelectedEntities {
    entities: Vec<Entity>,
    last_action: Option<(SelectionMode, Entity)>,
}

/// Kind of selection modifier
#[derive(Debug, Clone, Copy)]
pub enum SelectionMode {
    /// No modifiers
    Replace,
    /// `Ctrl`
    Add,
    /// `Shift`
    Extend,
}

impl From<egui::Modifiers> for SelectionMode {
    fn from(modifiers: egui::Modifiers) -> Self {
        match (modifiers.ctrl, modifiers.shift) {
            (true, _) => SelectionMode::Add,
            (false, true) => SelectionMode::Extend,
            (false, false) => SelectionMode::Replace,
        }
    }
}

impl SelectedEntities {
    pub fn select_replace(&mut self, entity: Entity) {
        self.insert_replace(entity);
        self.last_action = Some((SelectionMode::Replace, entity));
    }

    pub fn select_maybe_add(&mut self, entity: Entity, add: bool) {
        let mode = match add {
            true => SelectionMode::Add,
            false => SelectionMode::Replace,
        };
        self.select(mode, entity, |_, _| std::iter::empty());
    }

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
    pub fn as_slice(&self) -> &[Entity] {
        self.entities.as_slice()
    }
}
