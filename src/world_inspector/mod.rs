mod inspectable_registry;

use std::{any::TypeId, borrow::Cow};

pub use inspectable_registry::InspectableRegistry;

use bevy::{ecs::ResourceRef, utils::HashMap};
use bevy::{ecs::TypeInfo, prelude::*};
use bevy::{reflect::TypeRegistryArc, render::render_graph::base::MainPass};
use bevy_egui::{egui, EguiContext, EguiPlugin};

/// Plugin for displaying an inspector window of all entites in the world and their components.
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_inspector_egui::WorldInspectorPlugin;
///
/// fn main() {
///     App::build()
///         .add_plugins(DefaultPlugins)
///         .add_plugin(WorldInspectorPlugin)
///         .add_startup_system(setup.system())
///         .run();
/// }
///
/// fn setup(commands: &mut Commands) {
///   // setup your scene
///   // adding `Name` components will make the inspector more readable
/// }
/// ```
#[allow(missing_debug_implementations)]
pub struct WorldInspectorPlugin;

impl Plugin for WorldInspectorPlugin {
    fn build(&self, app: &mut AppBuilder) {
        if app.resources().get::<EguiContext>().is_none() {
            app.add_plugin(EguiPlugin);
        }

        app.init_resource::<InspectableRegistry>()
            .add_system(world_inspector_ui.system());
    }
}

fn world_inspector_ui(world: &mut World, resources: &mut Resources) {
    let egui_context = resources.get::<EguiContext>().expect("EguiContext");
    let ctx = &egui_context.ctx;

    egui::Window::new("World").scroll(true).show(ctx, |ui| {
        let ui_context = WorldUIContext::new(world, resources);
        ui_context.ui(ui);
    });
}
struct ComponentInfo {
    entity_index: usize,
    archetype_index: usize,
    types: Vec<TypeInfo>,
}

struct WorldUIContext<'a> {
    world: &'a mut World,
    resources: &'a Resources,
    inspectable_registry: ResourceRef<'a, InspectableRegistry>,
    type_registry: ResourceRef<'a, TypeRegistryArc>,
    components: HashMap<Entity, Vec<ComponentInfo>>,
}
impl<'a> WorldUIContext<'a> {
    fn new(world: &'a mut World, resources: &'a Resources) -> WorldUIContext<'a> {
        let mut components = HashMap::default();
        for (archetype_index, archetype) in world.archetypes().enumerate() {
            for (entity_index, entity) in archetype.iter_entities().enumerate() {
                let cs = components.entry(*entity).or_insert_with(Vec::new);
                let types = archetype.types().iter().cloned().collect::<Vec<_>>();
                let component_info = ComponentInfo {
                    entity_index,
                    archetype_index,
                    types,
                };
                cs.push(component_info);
            }
        }

        let inspectable_registry = resources.get::<InspectableRegistry>().unwrap();
        let type_registry = resources.get::<TypeRegistryArc>().unwrap();

        WorldUIContext {
            world,
            resources,
            inspectable_registry,
            type_registry,
            components,
        }
    }

    fn components_of(
        &self,
        entity: Entity,
    ) -> impl Iterator<Item = (usize, usize, &TypeInfo)> + '_ {
        self.components[&entity].iter().flat_map(|info| {
            info.types
                .iter()
                .map(move |type_info| (info.entity_index, info.archetype_index, type_info))
        })
    }

    fn entity_name(&self, entity: Entity) -> Cow<'_, str> {
        match self.world.get::<Name>(entity) {
            Ok(name) => name.as_str().into(),
            Err(_) => format!("Entity {}", entity.id()).into(),
        }
    }
}

impl WorldUIContext<'_> {
    fn ui(&self, ui: &mut egui::Ui) {
        let root_entities = self.world.query_filtered::<Entity, Without<Parent>>();

        for entity in root_entities {
            self.entity_ui(ui, entity);
        }
    }

    fn entity_ui(&self, ui: &mut egui::Ui, entity: Entity) {
        ui.collapsing(self.entity_name(entity), |ui| {
            ui.label("Components");

            for (entity_index, archetype_index, type_info) in self.components_of(entity) {
                if ignore_component(type_info) {
                    continue;
                }

                let type_name = type_info.type_name();
                let short_name = short_name(type_name);

                ui.collapsing(short_name, |ui| {
                    let could_display = self.inspectable_registry.generate(
                        &self.world,
                        &self.resources,
                        archetype_index,
                        entity_index,
                        type_info,
                        &*self.type_registry.read(),
                        ui,
                    );

                    if !could_display {
                        ui.label("Inspectable has not been defined for this component");
                    }
                });
            }

            ui.separator();

            let children = self.world.get::<Children>(entity);
            if let Some(children) = children.ok() {
                ui.label("Children");
                for &child in children.iter() {
                    self.entity_ui(ui, child);
                }
            } else {
                ui.label("No children");
            }
        });
    }
}

fn ignore_component(type_info: &TypeInfo) -> bool {
    let type_id = type_info.id();
    [
        TypeId::of::<Name>(),
        TypeId::of::<Children>(),
        TypeId::of::<Parent>(),
        TypeId::of::<PreviousParent>(),
        TypeId::of::<MainPass>(),
        TypeId::of::<Draw>(),
        TypeId::of::<RenderPipelines>(),
    ]
    .contains(&type_id)
}

fn short_name(type_name: &str) -> String {
    match type_name.find('<') {
        // no generics
        None => type_name.rsplit("::").next().unwrap_or(type_name).into(),
        // generics a::b::c<d>
        Some(angle_open) => {
            let angle_close = type_name.rfind('>').unwrap();

            let before_generics = &type_name[..angle_open];
            let after = &type_name[angle_close + 1..];
            let in_between = &type_name[angle_open + 1..angle_close];

            let before_generics = match before_generics.rfind("::") {
                None => before_generics,
                Some(i) => &before_generics[i + 2..],
            };

            let in_between = short_name(in_between);

            format!("{}<{}>{}", before_generics, in_between, after)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::short_name;

    #[test]
    fn shorten_name_basic() {
        assert_eq!(short_name("path::to::some::Type"), "Type".to_string());
    }
    #[test]
    fn shorten_name_generic() {
        assert_eq!(
            short_name("bevy::ecs::Handle<bevy::render::StandardMaterial>"),
            "Handle<StandardMaterial>".to_string()
        );
    }
    #[test]
    fn shorten_name_nested_generic() {
        assert_eq!(
            short_name("foo::bar::quux<qaax<p::t::b>>"),
            "quux<qaax<b>>".to_string()
        );
    }
}
