mod impls;
mod inspectable_registry;
mod plugin;

pub use impls::InspectorQuery;
pub use inspectable_registry::InspectableRegistry;
pub use plugin::WorldInspectorPlugin;

use bevy::{
    ecs::{
        component::{ComponentFlags, ComponentId, ComponentInfo, StorageType},
        entity::EntityLocation,
        query::{FilterFetch, WorldQuery},
    },
    prelude::*,
    reflect::{TypeRegistryArc, TypeRegistryInternal},
    render::render_graph::base::MainPass,
    utils::{HashMap, HashSet},
};
use bevy_egui::egui;
use egui::CollapsingHeader;
use pretty_type_name::pretty_type_name_str;
use std::{any::TypeId, borrow::Cow};

use crate::{utils::sort_iter_if, Context};

/// Resource which controls the way the world inspector is shown.
#[derive(Debug, Clone)]
pub struct WorldInspectorParams {
    /// these components will be ignored
    pub ignore_components: HashSet<TypeId>,
    /// these components will be read only
    pub read_only_components: HashSet<TypeId>,
    /// Whether to sort the components alphabetically
    pub sort_components: bool,
    /// Controls whether the world inspector is shown
    pub enabled: bool,
}
struct WorldUIContext<'a> {
    world: &'a mut World,
    ui_ctx: &'a egui::CtxRef,
    entities: HashMap<Entity, EntityLocation>,
}
impl<'a> WorldUIContext<'a> {
    fn new(ui_ctx: &'a egui::CtxRef, world: &'a mut World) -> WorldUIContext<'a> {
        let mut entities: HashMap<Entity, EntityLocation> = HashMap::default();
        for archetype in world.archetypes().iter() {
            for (entity_index, entity) in archetype.entities().iter().enumerate() {
                let location = EntityLocation {
                    archetype_id: archetype.id(),
                    index: entity_index,
                };

                let entity_components = entities.entry(*entity).or_insert(location);
                assert_eq!(location.archetype_id, entity_components.archetype_id);
                assert_eq!(location.index, entity_components.index);
            }
        }

        WorldUIContext {
            world,
            ui_ctx,
            entities,
        }
    }
}

impl<'a> WorldUIContext<'a> {
    fn entity_name(&self, entity: Entity) -> Cow<'_, str> {
        match self.world.get::<Name>(entity) {
            Some(name) => name.as_str().into(),
            None => format!("Entity {}", entity.id()).into(),
        }
    }

    fn world_ui<F>(&mut self, ui: &mut egui::Ui, params: &WorldInspectorParams)
    where
        F: WorldQuery,
        F::Fetch: FilterFetch,
    {
        let mut root_entities = self.world.query_filtered::<Entity, (Without<Parent>, F)>();

        // the entities are unique themselves, because only one WorldInspector can exist
        let dummy_id = egui::Id::new(42);

        for entity in root_entities.iter(self.world) {
            self.entity_ui(ui, entity, params, dummy_id);
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
        let location = match self.entities.get(&entity) {
            Some(value) => *value,
            None => return drop(ui.label("Entity does not exist")),
        };
        let archetype = self.world.archetypes().get(location.archetype_id).unwrap();

        self.component_kind_ui(
            ui,
            archetype.table_components(),
            "Components",
            entity,
            location,
            params,
        );
        self.component_kind_ui(
            ui,
            archetype.sparse_set_components(),
            "Components (Sparse)",
            entity,
            location,
            params,
        );

        ui.separator();

        let children = self.world.get::<Children>(entity);
        if let Some(children) = children {
            ui.label("Children");
            for &child in children.iter() {
                self.entity_ui(ui, child, params, id);
            }
        } else {
            ui.label("No children");
        }
    }

    fn component_kind_ui(
        &self,
        ui: &mut egui::Ui,
        components: &[ComponentId],
        title: &str,
        entity: Entity,
        entity_location: EntityLocation,
        params: &WorldInspectorParams,
    ) {
        if !components.is_empty() {
            ui.label(title);

            let iter = components.iter().map(|component_id| {
                let component_info = self.world.components().get_info(*component_id).unwrap();
                let name = pretty_type_name_str(component_info.name());
                (name, component_info, component_id)
            });
            let iter = sort_iter_if(iter, params.sort_components, |a, b| a.0.cmp(&b.0));

            for (name, component_info, component_id) in iter {
                self.component_ui(
                    ui,
                    name,
                    entity,
                    entity_location,
                    component_info,
                    component_id,
                    params,
                );
            }
        }
    }

    fn component_ui(
        &self,
        ui: &mut egui::Ui,
        name: String,
        entity: Entity,
        entity_location: EntityLocation,
        component_info: &ComponentInfo,
        component_id: &ComponentId,
        params: &WorldInspectorParams,
    ) {
        let type_id = match component_info.type_id() {
            Some(id) => id,
            None => {
                ui.label("No type id");
                return;
            }
        };

        if params.should_ignore_component(type_id) {
            return;
        }

        let inspectable_registry = self.world.get_resource::<InspectableRegistry>().unwrap();
        let type_registry = self.world.get_resource::<TypeRegistryArc>().unwrap();
        let type_registry = &*type_registry.internal.read();

        CollapsingHeader::new(name)
            .id_source(component_id)
            .show(ui, |ui| {
                if params.is_read_only(type_id) {
                    ui.set_enabled(false);
                }

                let world_ptr = self.world as *const _ as *mut _;

                let context = unsafe {
                    Context::new_ptr(self.ui_ctx, world_ptr).with_id(component_id.index() as u64)
                };

                let could_display = unsafe {
                    generate(
                        &self.world,
                        entity,
                        entity_location,
                        *component_id,
                        type_id,
                        inspectable_registry,
                        type_registry,
                        ui,
                        &context,
                    )
                };

                if !could_display {
                    ui.label("Inspectable has not been defined for this component");
                }
            });
    }
}

/// Safety:
/// The `location` must point to a valid archetype and index,
/// and the function must have unique access to the components.
#[allow(unused_unsafe)]
pub(crate) unsafe fn generate(
    world: &World,
    entity: Entity,
    location: EntityLocation,
    component_id: ComponentId,
    type_id: TypeId,
    inspectable_registry: &InspectableRegistry,
    type_registry: &TypeRegistryInternal,
    ui: &mut egui::Ui,
    context: &Context,
) -> bool {
    let (ptr, flags) = get_component_and_flags(world, component_id, entity, location).unwrap();

    let flags = unsafe { &mut *flags };
    flags.insert(ComponentFlags::MUTATED);

    if let Some(f) = inspectable_registry.impls.get(&type_id) {
        f(ptr, ui, &context);
        return true;
    }

    let success = (|| {
        let registration = type_registry.get(type_id)?;
        let reflect_component = registration.data::<ReflectComponent>()?;
        let mut reflected =
            unsafe { reflect_component.reflect_component_unchecked_mut(world, entity)? };
        crate::reflect::ui_for_reflect(&mut *reflected, ui, &context);
        Some(())
    })();

    success.is_some()
}

// copied from bevy
pub unsafe fn get_component_and_flags(
    world: &World,
    component_id: ComponentId,
    entity: Entity,
    location: EntityLocation,
) -> Option<(*mut u8, *mut ComponentFlags)> {
    let archetype = world.archetypes().get_unchecked(location.archetype_id);
    let component_info = world.components().get_info_unchecked(component_id);
    match component_info.storage_type() {
        StorageType::Table => {
            // SAFE: tables stored in archetype always exist
            let table = world.storages().tables.get_unchecked(archetype.table_id());
            let components = table.get_column(component_id)?;
            let table_row = archetype.entity_table_row(location.index);
            // SAFE: archetypes only store valid table_rows and the stored component type is T
            Some((
                components.get_unchecked(table_row),
                components.get_flags_unchecked(table_row),
            ))
        }
        StorageType::SparseSet => world
            .storages()
            .sparse_sets
            .get(component_id)
            .and_then(|sparse_set| sparse_set.get_with_flags(entity)),
    }
}

impl WorldInspectorParams {
    fn empty() -> Self {
        WorldInspectorParams {
            ignore_components: HashSet::default(),
            read_only_components: HashSet::default(),
            sort_components: false,
            enabled: true,
        }
    }

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
        let mut params = WorldInspectorParams::empty();

        params.ignore_components = [
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
        params.read_only_components = [TypeId::of::<GlobalTransform>()].iter().copied().collect();

        params
    }
}
