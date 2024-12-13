use bevy_app::{App, Plugin, PluginGroup, PluginGroupBuilder};
use bevy_state::{state::FreelyMutableState, prelude::in_state};
use bevy_minibuffer::{prelude::*, prompt::PromptState};
use crate::{
    quick::{WorldInspectorPlugin, StateInspectorPlugin},
    utils::pretty_type_name,
    minibuffer::{InspectorPlugins, Inspectors},
};
use bevy_ecs::{prelude::{Resource, Res, ResMut, World, Trigger}, schedule::Condition};
use bevy_state::app::AppExtStates;
use bevy_state::prelude::{State, NextState, States};
use bevy_egui::EguiContext;
use bevy_reflect::Reflect;
use trie_rs::map::Trie;

pub struct StateInspectorActs {
    plugins: InspectorPlugins<Self>,
    acts: Acts,
}

impl PluginGroup for StateInspectorActs {
    fn build(mut self) -> PluginGroupBuilder {
        self.warn_on_unused_acts();
        self.plugins.build()
    }
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
        self.plugins
            .add_inspector(pretty_type_name::<S>(), Self::add_plugin::<S>);
        self
    }

    fn add_plugin<A: FreelyMutableState + Reflect>(index: usize, inspector_plugins: &mut InspectorPlugins<Self>) {
        inspector_plugins.add_plugin(StateInspectorPlugin::<A>::default()
                                     .run_if(in_state(PromptState::Visible).and(InspectorPlugins::<Self>::visible(index))));
    }
}

fn state_inspector(states: Res<Inspectors<StateInspectorActs>>,
                   mut minibuffer: Minibuffer) {
    if !states.visible.is_empty() {
        minibuffer.prompt_map("state: ", states.names.clone())
            .observe(|mut trigger: Trigger<Completed<usize>>,
                     mut states: ResMut<Inspectors<StateInspectorActs>>,
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
            plugins: InspectorPlugins::default(),
            acts: Acts::new([Act::new(state_inspector)])
            // acts: Acts::new([Act::new(InspectorPlugins::<StateInspectorActs>::inspector("state: ", "No states registered"))
            // .named("state_inspector")])
        }
    }
}
