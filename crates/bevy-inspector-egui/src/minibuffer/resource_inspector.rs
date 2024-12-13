use bevy_app::{App, Plugin, PluginGroup, PluginGroupBuilder};
use bevy_state::prelude::in_state;
use bevy_minibuffer::{prelude::*, prompt::PromptState};
use crate::{
    quick::{WorldInspectorPlugin, ResourceInspectorPlugin},
    utils::pretty_type_name,
    minibuffer::{InspectorPlugins, Inspectors},
};
use bevy_ecs::{prelude::{Res, ResMut, Resource, World, Trigger}, schedule::Condition};
use bevy_state::app::AppExtStates;
use bevy_state::prelude::{State, NextState, States};
use bevy_egui::EguiContext;
use bevy_reflect::Reflect;
use trie_rs::map::Trie;

pub struct ResourceInspectorActs {
    plugins: InspectorPlugins<Self>,
    acts: Acts,
}

impl ActsPluginGroup for ResourceInspectorActs {
    fn acts(&self) -> &Acts {
        &self.acts
    }

    fn acts_mut(&mut self) -> &mut Acts {
        &mut self.acts
    }
}

impl ResourceInspectorActs {
    pub fn add<R: Resource + Reflect>(mut self) -> Self {
        self.plugins.add_inspector(pretty_type_name::<R>(), Self::resource_inspector_plugin::<R>);
        self
    }

    fn resource_inspector_plugin<R: Resource + Reflect>(index: usize, inspector_plugins: &mut InspectorPlugins<Self>) {
        inspector_plugins.add_plugin(ResourceInspectorPlugin::<R>::default()
                                     .run_if(in_state(PromptState::Visible).and(InspectorPlugins::<Self>::visible(index))));
    }
}

impl Default for ResourceInspectorActs {
    fn default() -> Self {
        Self {
            plugins: InspectorPlugins::default(),
            acts: Acts::new([Act::new(resource_inspector)]),
        }
    }
}

fn resource_inspector(resources: Res<Inspectors<ResourceInspectorActs>>,
                      mut minibuffer: Minibuffer) {

    if !resources.visible.is_empty() {
        minibuffer.prompt_map("resource: ", resources.names.clone())
            .observe(|mut trigger: Trigger<Completed<usize>>,
                     mut resources: ResMut<Inspectors<ResourceInspectorActs>>,
                     mut minibuffer: Minibuffer| {
                         if let Ok(index) = trigger.event_mut().take_result().unwrap() {
                             resources.visible[index] = !resources.visible[index];
                         }
            });
    } else {
        minibuffer.message("No resource inspectors available.");
    }
}


impl PluginGroup for ResourceInspectorActs {
    fn build(mut self) -> PluginGroupBuilder {
        self.warn_on_unused_acts();
        self.plugins.build()
    }
}
