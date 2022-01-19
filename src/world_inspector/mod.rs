pub(crate) mod impls;
mod inspectable_registry;
mod plugin;

use bevy::{
    ecs::archetype::Archetype, reflect::TypeRegistration, render::camera::Camera, ui::FocusPolicy,
    window::WindowId,
};
pub use inspectable_registry::InspectableRegistry;
pub use plugin::WorldInspectorPlugin;

use bevy::{
    ecs::{
        component::{ComponentId, ComponentTicks, StorageType},
        entity::EntityLocation,
        query::{FilterFetch, WorldQuery},
        world::EntityRef,
    },
    prelude::*,
    reflect::{TypeRegistryArc, TypeRegistryInternal},
    utils::HashSet,
};
use bevy_egui::egui::{self, Color32};
use egui::CollapsingHeader;
use pretty_type_name::pretty_type_name_str;
use std::{any::TypeId, cell::Cell};

use crate::{utils::ui::label_button, Context};
use impls::EntityAttributes;
use inspectable_registry::InspectCallback;

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
    /// Whether entities can be despawned
    pub despawnable_entities: bool,
    /// The window the inspector should be displayed on
    pub window: WindowId,
    /// Filter entities by name.
    pub name_filter: Option<String>,
    /// Highlight when components are changed in the world
    pub highlight_changes: bool,
}

impl WorldInspectorParams {
    fn empty() -> Self {
        WorldInspectorParams {
            ignore_components: HashSet::default(),
            read_only_components: HashSet::default(),
            sort_components: false,
            enabled: true,
            despawnable_entities: false,
            window: WindowId::primary(),
            name_filter: Some(String::new()),
            highlight_changes: false,
        }
    }

    /// Add `T` to component ignore list
    pub fn ignore_component<T: 'static>(&mut self) {
        self.ignore_components.insert(TypeId::of::<T>());
    }

    /// Filter entities by name.
    pub fn filter_by_name<S: Into<String>>(&mut self, filter: S) {
        self.name_filter = Some(filter.into());
    }

    fn should_ignore_component(&self, type_id: TypeId) -> bool {
        self.ignore_components.contains(&type_id)
    }

    fn is_read_only(&self, type_id: TypeId) -> bool {
        self.read_only_components.contains(&type_id)
    }

    fn entity_options(&self) -> EntityAttributes {
        EntityAttributes {
            despawnable: self.despawnable_entities,
        }
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
        ]
        .iter()
        .copied()
        .collect();
        params.read_only_components = [TypeId::of::<GlobalTransform>()].iter().copied().collect();

        #[cfg(feature = "rapier")]
        {
            params
                .ignore_components
                .insert(TypeId::of::<bevy_rapier3d::prelude::RigidBodyIds>());
            params
                .ignore_components
                .insert(TypeId::of::<bevy_rapier3d::prelude::ColliderBroadPhaseData>());
        }
        #[cfg(feature = "rapier2d")]
        {
            params
                .ignore_components
                .insert(TypeId::of::<bevy_rapier2d::prelude::RigidBodyIds>());
            params
                .ignore_components
                .insert(TypeId::of::<bevy_rapier2d::prelude::ColliderBroadPhaseData>());
        }

        params
    }
}

/// Context for providing the [`WorldInspectorPlugin`](crate::WorldInspectorPlugin)'s ui
pub struct WorldUIContext<'a> {
    world: &'a mut World,
    ui_ctx: Option<&'a egui::CtxRef>,
    delete_entity: Cell<Option<Entity>>,
}

impl Drop for WorldUIContext<'_> {
    fn drop(&mut self) {
        if let Some(entity) = self.delete_entity.get() {
            despawn_with_children_recursive(self.world, entity);
        }
    }
}

impl<'a> WorldUIContext<'a> {
    /// Create a new world ui context.
    pub fn new(world: &'a mut World, ui_ctx: Option<&'a egui::CtxRef>) -> WorldUIContext<'a> {
        WorldUIContext {
            world,
            ui_ctx,
            delete_entity: Cell::new(None),
        }
    }

    /// Displays the world inspector UI.
    pub fn world_ui<F>(&mut self, ui: &mut egui::Ui, params: &mut WorldInspectorParams) -> bool
    where
        F: WorldQuery,
        F::Fetch: FilterFetch,
    {
        let mut root_entities = self.world.query_filtered::<Entity, (Without<Parent>, F)>();

        // the entities are unique themselves, because only one WorldInspector can exist
        let dummy_id = egui::Id::new(42);
        let entity_options = params.entity_options();

        let mut changed = false;

        if let Some(filter) = params.name_filter.as_mut() {
            ui.horizontal(|ui| {
                ui.label("Named");
                ui.text_edit_singleline(filter);
            });
        }

        let entites = root_entities.iter(self.world).collect::<Vec<_>>();
        for entity in entites {
            changed |= self.entity_ui(ui, entity, params, dummy_id.with(entity), &entity_options);
        }

        changed
    }

    /// Displays an entity and its children in a collapsing header.
    pub fn entity_ui(
        &mut self,
        ui: &mut egui::Ui,
        entity: Entity,
        params: &WorldInspectorParams,
        id: egui::Id,
        entity_options: &EntityAttributes,
    ) -> bool {
        let entity_name = entity_name(self.world, entity);
        let show_entity = params
            .name_filter
            .as_ref()
            .map(|filter| {
                filter.is_empty() || entity_name.to_lowercase().contains(&filter.to_lowercase())
            })
            .unwrap_or(true);

        if show_entity {
            CollapsingHeader::new(entity_name.to_string())
                .id_source(id.with(entity))
                .show(ui, |ui| {
                    self.entity_ui_inner(ui, entity, params, id, entity_options)
                })
                .body_returned
                .unwrap_or(false)
        } else {
            let children = self.world.get::<Children>(entity).cloned();
            if let Some(children) = children {
                for &child in children.iter() {
                    self.entity_ui(ui, child, params, id.with(child), entity_options);
                }
            }
            false
        }
    }

    /// Displays an entity and its children.
    pub fn entity_ui_inner(
        &mut self,
        ui: &mut egui::Ui,
        entity: Entity,
        params: &WorldInspectorParams,
        id: egui::Id,
        entity_options: &EntityAttributes,
    ) -> bool {
        let mut changed = false;

        changed |= self.component_kind_ui(
            ui,
            |archetype| archetype.table_components(),
            "Components",
            entity,
            params,
            id,
        );
        changed |= self.component_kind_ui(
            ui,
            |archetype| archetype.sparse_set_components(),
            "Components (Sparse)",
            entity,
            params,
            id,
        );

        ui.separator();

        let children = self.world.get::<Children>(entity).cloned();
        if let Some(children) = children {
            ui.label("Children");
            for &child in children.iter() {
                self.entity_ui(ui, child, params, id.with(child), entity_options);
            }
        } else {
            ui.label("No children");
        }

        if entity_options.despawnable {
            if label_button(ui, "✖ Despawn", Color32::RED) {
                self.delete_entity.set(Some(entity));
                changed = true;
            }
        }

        changed
    }

    /// Safety:
    /// `entity_location` must point to a valid archetype and index.
    fn component_kind_ui(
        &mut self,
        ui: &mut egui::Ui,
        components: impl Fn(&Archetype) -> &[ComponentId],
        title: &str,
        entity: Entity,
        params: &WorldInspectorParams,
        id: egui::Id,
    ) -> bool {
        let (entity_location, components) = {
            let entity_ref = match self.world.get_entity(entity) {
                Some(entity_ref) => entity_ref,
                None => {
                    ui.label("Entity does not exist");
                    return false;
                }
            };
            let entity_location = entity_ref.location();
            let archetype = entity_ref.archetype();

            let components = components(archetype).to_vec();
            (entity_location, components)
        };

        if !components.is_empty() {
            ui.label(title);

            let mut components: Vec<_> = components
                .iter()
                .map(|component_id| {
                    let component_info = self.world.components().get_info(*component_id).unwrap();
                    let name = pretty_type_name_str(component_info.name());
                    (
                        name,
                        component_info.id(),
                        component_info.type_id(),
                        component_info.layout().size(),
                    )
                })
                .collect();

            if params.sort_components {
                components.sort_by(|a, b| a.0.cmp(&b.0));
            }

            let mut changed = false;
            for (name, component_id, component_type_id, size) in components {
                let is_zst = size == 0;
                changed |= unsafe {
                    self.component_ui(
                        ui,
                        name,
                        entity,
                        entity_location,
                        component_id,
                        component_type_id,
                        is_zst,
                        params,
                        id,
                    )
                }
            }
            changed
        } else {
            false
        }
    }

    /// Safety:
    /// `entity_location` must point to a valid archetype and index.
    unsafe fn component_ui(
        &mut self,
        ui: &mut egui::Ui,
        name: String,
        entity: Entity,
        entity_location: EntityLocation,
        component_id: ComponentId,
        component_type_id: Option<TypeId>,
        is_zst: bool,
        params: &WorldInspectorParams,
        id: egui::Id,
    ) -> bool {
        let type_id = match component_type_id {
            Some(id) => id,
            None => {
                ui.label("No type id");
                return false;
            }
        };

        if params.should_ignore_component(type_id) {
            return false;
        }

        // Safety: according to this function's contract, entity_location is valid,
        // and self.world gives us exclusive access.
        let (component_ptr, component_ticks) = {
            let (ptr, ticks) =
                get_component_and_ticks(self.world, component_id, entity, entity_location).unwrap();
            (ptr, { &mut *ticks })
        };

        if params.highlight_changes
            && component_ticks
                .is_changed(self.world.last_change_tick(), self.world.read_change_tick())
        {
            ui.style_mut().visuals.collapsing_header_frame = true;
            ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke {
                width: 1.0,
                color: egui::Color32::GOLD,
            };
            ui.style_mut().visuals.widgets.active.bg_stroke = egui::Stroke {
                width: 1.0,
                color: egui::Color32::GOLD,
            };
            ui.style_mut().visuals.widgets.hovered.bg_stroke = egui::Stroke {
                width: 1.0,
                color: egui::Color32::GOLD,
            };
            ui.style_mut().visuals.widgets.noninteractive.bg_stroke = egui::Stroke {
                width: 1.0,
                color: egui::Color32::GOLD,
            };
        }

        let id = id.with(component_id);
        let changed = CollapsingHeader::new(name)
            .id_source(id)
            .show(ui, |ui| {
                ui.reset_style();

                if params.is_read_only(type_id) {
                    ui.set_enabled(false);
                }

                let result = self.world
                    .resource_scope(|world, inspectable_registry: Mut<InspectableRegistry>| {
                        world.resource_scope(|world, type_registry: Mut<TypeRegistryArc>| {
                        let type_registry = &*type_registry.internal.read();

                        // Safety: according to this function's contract, entity_location (and therefore component_ptr) are valid
                        try_display(
                            world,
                            entity,
                            component_ptr,
                            is_zst,
                            type_id,
                            &*inspectable_registry,
                            type_registry,
                            ui,
                            self.ui_ctx,
                            id,
                        )
                    })
                });

                if result.is_err() {
                    no_inspectable_error_message(ui);
                }

                result.unwrap_or(false)
            })
            .body_returned
            .unwrap_or(false);

        if changed {
            component_ticks.set_changed(self.world.change_tick());
        }

        ui.reset_style();
        changed
    }
}

/// Safety:
/// `component_ptr` must point to a valid component, and the function must have unique access.
pub(crate) unsafe fn try_display(
    world: &mut World,
    entity: Entity,
    component_ptr: *mut u8,
    is_zst: bool,
    type_id: TypeId,
    inspectable_registry: &InspectableRegistry,
    type_registry: &TypeRegistryInternal,
    ui: &mut egui::Ui,
    ui_ctx: Option<&egui::CtxRef>,
    id: egui::Id,
) -> Result<bool, ()> {
    if let Some(inspect_callback) = inspectable_registry.impls.get(&type_id) {
        let id = id_to_u64(&id);
        let mut context = Context::new_world_access(ui_ctx, world);
        let mut context = context.with_id(id);

        let changed =
            display_by_inspectable_registry(inspect_callback, component_ptr, ui, &mut context);
        return Ok(changed);
    }

    if is_zst {
        return Ok(false);
    }

    if let Ok(changed) = display_by_reflection(
        type_registry,
        inspectable_registry,
        type_id,
        world,
        entity,
        ui,
        ui_ctx,
        id,
    ) {
        return Ok(changed);
    }

    Err(())
}

/// Safety:
/// `component_ptr` must point to a valid component to which the the caller has unique access
unsafe fn display_by_inspectable_registry(
    inspect_callback: &InspectCallback,
    component_ptr: *mut u8,
    ui: &mut egui::Ui,
    context: &mut Context,
) -> bool {
    inspect_callback(component_ptr, ui, context)
}

fn display_by_reflection(
    type_registry: &TypeRegistryInternal,
    inspectable_registry: &InspectableRegistry,
    type_id: TypeId,
    world: &mut World,
    entity: Entity,
    ui: &mut egui::Ui,
    ui_ctx: Option<&egui::CtxRef>,
    id: egui::Id,
) -> Result<bool, ()> {
    let registration = type_registry.get(type_id).ok_or(())?;

    let reflect_component = match registration.data::<ReflectComponent>() {
        Some(reflect_component) => reflect_component,
        None => {
            reflection_error_message(ui, registration);

            return Ok(false);
        }
    };

    let mut reflected = {
        let world = unsafe { &mut *(world as *mut _) };
        reflect_component
            .reflect_component_mut(world, entity)
            .ok_or(())?
    };

    let id = id_to_u64(&id);
    let mut context = Context::new_world_access(ui_ctx, world);
    let mut context = context.with_id(id);

    Ok(crate::reflect::ui_for_reflect_with_registry(
        &mut *reflected,
        ui,
        &mut context,
        Some(inspectable_registry),
    ))
}

macro_rules! layout_job {
    ( $($kind:ident $text:expr),* $(,)?) => {{
        let mut job = egui::epaint::text::LayoutJob::default();
        $(
            job.append(
                $text,
                0.0,
                egui::TextFormat {
                    style: egui::TextStyle::$kind,
                    ..Default::default()
                },
            );
        )*
        job
    }}
}

fn reflection_error_message(ui: &mut egui::Ui, registration: &TypeRegistration) {
    let job = layout_job!(
        Monospace registration.short_name(), Body " implements ", Monospace "Reflect",
        Body ", but does not have a ", Monospace "#[reflect(Component)]", Body " attribute.",
    );

    ui.label(job);
}

fn no_inspectable_error_message(ui: &mut egui::Ui) {
    let job = layout_job!(
        Body "This component is neither ", Monospace "Reflect", Body " nor ", Monospace "Inspectable",
        Body ".\nMake sure to also call ", Monospace "app.register_type", Body " or ", Monospace "app.register_inspectable", Body ".",
    );

    ui.label(job);
}

// copied from bevy
#[inline]
unsafe fn get_component_and_ticks(
    world: &mut World,
    component_id: ComponentId,
    entity: Entity,
    location: EntityLocation,
) -> Option<(*mut u8, *mut ComponentTicks)> {
    let archetype = &world.archetypes()[location.archetype_id];
    let component_info = world.components().get_info_unchecked(component_id);
    match component_info.storage_type() {
        StorageType::Table => {
            let table = &world.storages().tables[archetype.table_id()];
            let components = table.get_column(component_id)?;
            let table_row = archetype.entity_table_row(location.index);
            // SAFE: archetypes only store valid table_rows and the stored component type is T
            Some((
                components.get_data_unchecked(table_row),
                components.get_ticks_mut_ptr_unchecked(table_row),
            ))
        }
        StorageType::SparseSet => world
            .storages()
            .sparse_sets
            .get(component_id)
            .and_then(|sparse_set| sparse_set.get_with_ticks(entity)),
    }
}

macro_rules! is_bundle {
    ($entity:ident: $($ty:ty),* $(,)?) => {
        $( $entity.contains::<$ty>() && )* true
    };
}

/// Guesses an appropriate entity name like `Light (6)` or falls back to `Entity (8)`
pub fn entity_name(world: &World, entity: Entity) -> String {
    match world.get_entity(entity) {
        Some(entity) => guess_entity_name_inner(entity),
        None => format!("Entity {} (inexistent)", entity.id()),
    }
}

fn guess_entity_name_inner(entity: EntityRef) -> String {
    if let Some(name) = entity.get::<Name>() {
        return name.as_str().to_string();
    }

    let id = entity.id().id();

    if let Some(camera) = entity.get::<Camera>() {
        match &camera.name {
            Some(name) => return name.to_string(),
            None => return format!("Camera ({})", id),
        }
    }

    if is_bundle!(entity: PointLight, Transform, GlobalTransform) {
        return format!("Light ({})", id);
    }

    if is_bundle!(entity: DirectionalLight, Transform, GlobalTransform) {
        return format!("Directional Light ({})", id);
    }

    if is_bundle!(
        entity: Handle<Mesh>,
        Handle<StandardMaterial>,
        Transform,
        GlobalTransform
    ) {
        return format!("Pbr Mesh ({})", id);
    }

    if is_bundle!(
        entity: Node,
        Style,
        Text,
        CalculatedSize,
        FocusPolicy,
        Transform,
        GlobalTransform
    ) {
        return format!("Test ({})", id);
    }
    if is_bundle!(
        entity: Text,
        Transform,
        GlobalTransform,
        bevy::text::Text2dSize
    ) {
        return format!("Text2d ({})", id);
    }
    if is_bundle!(
        entity: Node,
        Style,
        UiColor,
        UiImage,
        Transform,
        GlobalTransform
    ) {
        return format!("Node ({})", id);
    }

    format!("Entity ({:?})", id)
}

fn id_to_u64(id: &egui::Id) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hash;
    use std::hash::Hasher;
    let mut hasher = DefaultHasher::default();
    id.hash(&mut hasher);
    hasher.finish()
}
