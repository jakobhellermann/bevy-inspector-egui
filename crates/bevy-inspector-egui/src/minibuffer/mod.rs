use bevy_app::{App, Plugin, PluginGroup, PluginGroupBuilder};
use bevy_state::prelude::in_state;
use bevy_minibuffer::{prelude::*, prompt::PromptState};
use crate::{
    quick::{WorldInspectorPlugin, ResourceInspectorPlugin},
    utils::pretty_type_name,
};
use bevy_ecs::{prelude::{Res, ResMut, Resource, World, Trigger}, schedule::Condition};
use bevy_state::app::AppExtStates;
use bevy_state::prelude::{State, NextState, States};
use bevy_egui::EguiContext;
use bevy_reflect::Reflect;
use trie_rs::map::Trie;

mod inspector_plugins;
pub use inspector_plugins::*;
mod resource_inspector;
pub use resource_inspector::*;
mod asset_inspector;
pub use asset_inspector::*;
mod state_inspector;
pub use state_inspector::*;
mod filter_query_inspector;
pub use filter_query_inspector::*;

/// Is the prompt visible?
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States, Reflect)]
pub enum WorldInspectorState {
    /// Invisible
    #[default]
    Invisible,
    /// Visible
    Visible,
}

#[derive(Resource)]
pub struct ResourceInspectors {
    names: Trie<u8, usize>,
    visible: Vec<bool>,
}

pub struct ResourceInspectorPlugins {
    plugins: Option<PluginGroupBuilder>,
    resource_names: Vec<String>,
}

impl ResourceInspectorPlugins {
    pub fn add<R: Resource + Reflect>(mut self) -> Self {
        let index = self.resource_names.len();
        self.resource_names.push(pretty_type_name::<R>());
        self.add_plugin(resource_inspector_plugin::<R>(index));
        self
    }

    fn add_plugin<T: Plugin>(&mut self, plugin: T) {
        let builder = self.plugins.take().expect("plugin builder");
        self.plugins = Some(builder.add(plugin));
    }
}

impl Default for ResourceInspectorPlugins {
    fn default() -> Self {
        Self {
            plugins: Some(PluginGroupBuilder::start::<Self>()),
            resource_names: vec![],
        }
    }
}

impl PluginGroup for ResourceInspectorPlugins {
    fn build(mut self) -> PluginGroupBuilder {
        let builder = self.plugins.take().expect("plugin builder");
        // let resource_names = self.resource_names.drain(..);
        builder.add(move |app: &mut App| {
            let count = self.resource_names.len();
            let trie = Trie::from_iter(self.resource_names.clone().into_iter().enumerate().map(|(index, name)| (name, index)));
            app.insert_resource(ResourceInspectors {
                names: trie,
                visible: vec![false; count],
            });
        })
    }
}

pub struct InspectorActs {
    pub acts: Acts,
}

fn resource_visible(index: usize) -> impl Fn(Res<ResourceInspectors>) -> bool {
    move |inspectors: Res<ResourceInspectors>| {
        inspectors.visible.get(index).copied().unwrap_or(false)
    }
}

fn resource_inspector_plugin<R: Resource + Reflect>(index: usize) -> impl Fn(&mut App) {
    move |app: &mut App| {
        app.add_plugins(ResourceInspectorPlugin::<R>::default()
                    .run_if(in_state(PromptState::Visible).and(resource_visible(index)))
        );
    }
}

impl ActsPlugin for InspectorActs {
    fn acts(&self) -> &Acts {
        &self.acts
    }
    fn acts_mut(&mut self) -> &mut Acts {
        &mut self.acts
    }
}

fn world_inspector(state: Res<State<WorldInspectorState>>,
                   mut next_state: ResMut<NextState<WorldInspectorState>>,
                   mut minibuffer: Minibuffer) {
    use WorldInspectorState::*;
    let (state, msg) = match state.get() {
        Invisible => (Visible, "Show world inspector"),
        Visible => (Invisible, "Hide world inspector"),
    };
    next_state.set(state);
    minibuffer.message(msg);
}

impl Default for InspectorActs {
    fn default() -> Self {
        Self {
            acts: Acts::new([
                Act::new(world_inspector),
                ]),
        }
    }
}

impl Plugin for InspectorActs {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorldInspectorPlugin::default().run_if(in_state(PromptState::Visible).and(in_state(WorldInspectorState::Visible))))
            .init_state::<WorldInspectorState>();
        self.warn_on_unused_acts();
    }
}
