use bevy_app::{App, Plugin, PluginGroup, PluginGroupBuilder};
use bevy_state::{state::FreelyMutableState, prelude::in_state};
use bevy_minibuffer::{prelude::*, prompt::PromptState};
use crate::{
    quick::{WorldInspectorPlugin, StateInspectorPlugin},
    utils::pretty_type_name,
};
use bevy_ecs::{prelude::{Resource, Res, ResMut, World, Trigger}, schedule::Condition};
use bevy_state::app::AppExtStates;
use bevy_state::prelude::{State, NextState, States};
use bevy_egui::EguiContext;
use bevy_reflect::Reflect;
use trie_rs::map::Trie;

#[derive(Resource)]
pub(crate) struct StateInspectors {
    pub(crate) names: Trie<u8, usize>,
    pub(crate) visible: Vec<bool>,
}

pub struct StateInspectorActs {
    plugins: Option<PluginGroupBuilder>,
    state_names: Vec<String>,
    acts: Acts,
}

impl ActsPluginGroup for StateInspectorActs {
    fn acts(&self) -> &Acts {
        &self.acts
    }

    fn acts_mut(&mut self) -> &mut Acts {
        &mut self.acts
    }
}

impl StateInspectorActs {
    pub fn add<S: FreelyMutableState + Reflect>(mut self) -> Self {
        let index = self.state_names.len();
        self.state_names.push(pretty_type_name::<S>());
        self.add_plugin(state_inspector_plugin::<S>(index));
        self
    }

    fn add_plugin<T: Plugin>(&mut self, plugin: T) {
        let builder = self.plugins.take().expect("plugin builder");
        self.plugins = Some(builder.add(plugin));
    }
}

fn state_inspector(states: Res<StateInspectors>,
                      mut minibuffer: Minibuffer) {
    if !states.visible.is_empty() {
        minibuffer.prompt_map("state: ", states.names.clone())
            .observe(|mut trigger: Trigger<Completed<usize>>,
                     mut states: ResMut<StateInspectors>,
                     mut minibuffer: Minibuffer| {
                         if let Ok(index) = trigger.event_mut().take_result().unwrap() {
                             states.visible[index] = !states.visible[index];
                         }
            });
    } else {
        minibuffer.message("No states registered.");
    }
}

impl Default for StateInspectorActs {
    fn default() -> Self {
        Self {
            plugins: Some(PluginGroupBuilder::start::<Self>()),
            state_names: vec![],
            acts: Acts::new([Act::new(state_inspector)])
        }
    }
}

impl PluginGroup for StateInspectorActs {
    fn build(mut self) -> PluginGroupBuilder {
        let builder = self.plugins.take().expect("plugin builder");
        self.warn_on_unused_acts();
        builder.add(move |app: &mut App| {
            let count = self.state_names.len();
            let trie = Trie::from_iter(self.state_names.clone().into_iter().enumerate().map(|(index, name)| (name, index)));
            app.insert_resource(StateInspectors {
                names: trie,
                visible: vec![false; count],
            });
        })
    }
}

fn state_visible(index: usize) -> impl Fn(Res<StateInspectors>) -> bool {
    move |inspectors: Res<StateInspectors>| {
        inspectors.visible.get(index).copied().unwrap_or(false)
    }
}

fn state_inspector_plugin<A: FreelyMutableState + Reflect>(index: usize) -> impl Fn(&mut App) {
    move |app: &mut App| {
        app.add_plugins(StateInspectorPlugin::<A>::default()
                    .run_if(in_state(PromptState::Visible).and(state_visible(index)))
        );
    }
}
