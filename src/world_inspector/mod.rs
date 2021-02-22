mod impls;
mod inspectable_registry;
mod plugin;

use egui::CollapsingHeader;
pub use inspectable_registry::InspectableRegistry;
pub use plugin::WorldInspectorPlugin;
use pretty_type_name::pretty_type_name_str;

pub use impls::InspectorQuery;

use bevy::ecs::{Location, ResourceRef};
use bevy::reflect::TypeRegistryArc;
use bevy::render::render_graph::base::MainPass;
use bevy::utils::{HashMap, HashSet};
use bevy::{ecs::TypeInfo, prelude::*};
use bevy_egui::egui;
use std::{any::TypeId, borrow::Cow};

use crate::Context;

/// Resource which controls the way the world inspector is shown.
#[derive(Debug, Clone)]
pub struct WorldInspectorParams {
    /// these components will be ignored
    pub ignore_components: HashSet<TypeId>,
    /// these components will be read only
    pub read_only_components: HashSet<TypeId>,
    /// if this option is enabled, the inspector will cluster the entities by archetype
    pub cluster_by_archetype: bool,
    /// Whether to sort the components alphabetically
    pub sort_components: bool,
    /// Controls whether the world inspector is shown
    pub enabled: bool,
}

struct WorldUIContext<'a> {
    world: &'a World,
    resources: &'a Resources,
    inspectable_registry: ResourceRef<'a, InspectableRegistry>,
    type_registry: ResourceRef<'a, TypeRegistryArc>,
    components: HashMap<Entity, (Location, Vec<TypeInfo>)>,
    ui_ctx: &'a egui::CtxRef,
}
impl<'a> WorldUIContext<'a> {
    fn new(
        ui_ctx: &'a egui::CtxRef,
        world: &'a World,
        resources: &'a Resources,
    ) -> WorldUIContext<'a> {
        let mut components: HashMap<Entity, (Location, Vec<TypeInfo>)> = HashMap::default();
        for (archetype_index, archetype) in world.archetypes().enumerate() {
            for (entity_index, entity) in archetype.iter_entities().enumerate() {
                let location = Location {
                    archetype: archetype_index as u32,
                    index: entity_index,
                };

                let entity_components = components
                    .entry(*entity)
                    .or_insert_with(|| (location, Vec::new()));

                assert_eq!(location.archetype, entity_components.0.archetype);
                assert_eq!(location.index, entity_components.0.index);

                entity_components.1.extend(archetype.types());
            }
        }

        let inspectable_registry = resources.get::<InspectableRegistry>().unwrap();
        let type_registry = resources.get::<TypeRegistryArc>().unwrap();

        WorldUIContext {
            ui_ctx,
            world,
            resources,
            inspectable_registry,
            type_registry,
            components,
        }
    }

    fn entity_name(&self, entity: Entity) -> Cow<'_, str> {
        match self.world.get::<Name>(entity) {
            Ok(name) => name.as_str().into(),
            Err(_) => format!("Entity {}", entity.id()).into(),
        }
    }
}

impl WorldUIContext<'_> {
    fn ui(&self, ui: &mut egui::Ui, params: &WorldInspectorParams) {
        if params.cluster_by_archetype {
            self.ui_split_archetypes(ui, params);
        } else {
            self.ui_all_entities(ui, params);
        }
    }

    fn ui_all_entities(&self, ui: &mut egui::Ui, params: &WorldInspectorParams) {
        let root_entities = self.world.query_filtered::<Entity, Without<Parent>>();

        // the entities are unique themselves, because only one WorldInspector can exist
        let dummy_id = egui::Id::new(42);

        for entity in root_entities {
            self.entity_ui(ui, entity, params, dummy_id);
        }
    }

    fn ui_split_archetypes(&self, ui: &mut egui::Ui, params: &WorldInspectorParams) {
        let root_entities = self.world.query_filtered::<Entity, Without<Parent>>();

        // the entities are unique themselves, because only one WorldInspector can exist
        let dummy_id = egui::Id::new(42);

        let mut archetypes: Vec<u32> = Vec::new();
        let entities: Vec<_> = root_entities
            .map(|entity| {
                let (location, _) = &self.components[&entity];
                if !archetypes.contains(&location.archetype) {
                    archetypes.push(location.archetype);
                }
                (*location, entity)
            })
            .collect();

        for archetype in archetypes {
            let archetype_label = format!("Archetype {}", archetype);
            ui.collapsing(archetype_label, |ui| {
                for (location, entity) in &entities {
                    if location.archetype == archetype {
                        self.entity_ui(ui, *entity, params, dummy_id);
                    }
                }
            });
        }
    }

    fn entity_ui(
        &self,
        ui: &mut egui::Ui,
        entity: Entity,
        params: &WorldInspectorParams,
        id: egui::Id,
    ) {
        CollapsingHeader::new(self.entity_name(entity))
            .id_source(id.with(entity))
            .show(ui, |ui| {
                self.entity_ui_inner(ui, entity, params, id);
            });
    }

    fn entity_ui_inner(
        &self,
        ui: &mut egui::Ui,
        entity: Entity,
        params: &WorldInspectorParams,
        id: egui::Id,
    ) {
        let (location, components) = match &self.components.get(&entity) {
            Some(value) => value,
            None => {
                ui.label("Entity does not exist");
                return;
            }
        };

        ui.label("Components");

        let components = components
            .iter()
            .map(|type_info| (pretty_type_name_str(type_info.type_name()), type_info));
        let components = sort_iter_if(components, params.sort_components);

        for (short_name, type_info) in components {
            if params.should_ignore_component(type_info.id()) {
                continue;
            }

            ui.collapsing(short_name, |ui| {
                if params.is_read_only(type_info.id()) {
                    ui.set_enabled(false);
                }

                let context = Context::new(self.ui_ctx, self.world, self.resources)
                    .with_id(entity.id() as u64);

                // SAFETY: we have unique access to the world
                let could_display = unsafe {
                    self.inspectable_registry.generate(
                        self.world,
                        *location,
                        type_info,
                        &*self.type_registry.read(),
                        ui,
                        &context,
                    )
                };

                if !could_display {
                    ui.label("Inspectable has not been defined for this component");
                }
            });
        }

        ui.separator();

        let children = self.world.get::<Children>(entity);
        if let Ok(children) = children {
            ui.label("Children");
            for &child in children.iter() {
                self.entity_ui(ui, child, params, id);
            }
        } else {
            ui.label("No children");
        }
    }
}

impl WorldInspectorParams {
    /// Add `T` to component ignore list
    pub fn ignore_component<T: 'static>(&mut self) {
        self.ignore_components.insert(TypeId::of::<T>());
    }

    fn should_ignore_component(&self, type_id: TypeId) -> bool {
        self.ignore_components.contains(&type_id)
    }

    fn is_read_only(&self, type_id: TypeId) -> bool {
        self.read_only_components.contains(&type_id)
    }
}

impl Default for WorldInspectorParams {
    fn default() -> Self {
        let ignore_components = [
            TypeId::of::<Name>(),
            TypeId::of::<Children>(),
            TypeId::of::<Parent>(),
            TypeId::of::<PreviousParent>(),
            TypeId::of::<MainPass>(),
            TypeId::of::<Draw>(),
            TypeId::of::<RenderPipelines>(),
        ]
        .iter()
        .copied()
        .collect();
        let read_only_components = [TypeId::of::<GlobalTransform>()].iter().copied().collect();

        WorldInspectorParams {
            ignore_components,
            read_only_components,
            cluster_by_archetype: false,
            sort_components: false,
            enabled: true,
        }
    }
}

/// Sorts an iterator if a condition is met.
/// This avoids collecting the iterator
/// if it shouldn't be sorted.
// Overenginereed? Yes. Had fun implementing? Also yes.
fn sort_iter_if<T, I>(iter: I, sort: bool) -> impl Iterator<Item = T>
where
    I: Iterator<Item = T>,
    T: Ord,
{
    if sort {
        let mut items: Vec<_> = iter.collect();
        items.sort();
        TwoIter::I(items.into_iter())
    } else {
        TwoIter::J(iter)
    }
}

enum TwoIter<I, J> {
    I(I),
    J(J),
}
impl<T, I, J> Iterator for TwoIter<I, J>
where
    I: Iterator<Item = T>,
    J: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            TwoIter::I(i) => i.next(),
            TwoIter::J(j) => j.next(),
        }
    }
}
