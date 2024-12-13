use bevy_app::{App, Plugin, PluginGroup, PluginGroupBuilder};
use bevy_state::prelude::in_state;
use bevy_minibuffer::{prelude::*, prompt::PromptState};
use crate::{
    quick::{WorldInspectorPlugin, AssetInspectorPlugin, FilterQueryInspectorPlugin},
    utils::pretty_type_name,
    minibuffer::{Inspectors, InspectorPlugins},
};
use bevy_asset::Asset;
use bevy_ecs::{prelude::{Resource, Res, ResMut, World, Trigger}, schedule::Condition, query::QueryFilter};
use bevy_state::app::AppExtStates;
use bevy_state::prelude::{State, NextState, States};
use bevy_egui::EguiContext;
use bevy_reflect::Reflect;
use trie_rs::map::Trie;

pub struct FilterQueryInspectorActs {
    plugins: InspectorPlugins<Self>,
    acts: Acts,
}

impl ActsPluginGroup for FilterQueryInspectorActs {
    fn acts(&self) -> &Acts {
        &self.acts
    }

    fn acts_mut(&mut self) -> &mut Acts {
        &mut self.acts
    }
}

impl FilterQueryInspectorActs {
    pub fn add<A: QueryFilter + 'static>(mut self) -> Self {
        self.plugins.add_inspector(pretty_type_name::<A>(), Self::filter_query_inspector_plugin::<A>);
        self
    }

    fn filter_query_inspector_plugin<A: QueryFilter + 'static>(index: usize, inspector_plugins: &mut InspectorPlugins<Self>) {
        inspector_plugins.add_plugin(FilterQueryInspectorPlugin::<A>::default()
                                     .run_if(in_state(PromptState::Visible).and(InspectorPlugins::<Self>::visible(index)))
        );
    }
}

impl Default for FilterQueryInspectorActs {
    fn default() -> Self {
        Self {
            plugins: InspectorPlugins::default(),
            acts: Acts::new([Act::new(filter_query_inspector)]),
        }
    }
}

fn filter_query_inspector(assets: Res<Inspectors<FilterQueryInspectorActs>>,
                   mut minibuffer: Minibuffer) {
    if !assets.visible.is_empty() {
        minibuffer.prompt_map("filter query: ", assets.names.clone())
            .observe(|mut trigger: Trigger<Completed<usize>>,
                     mut assets: ResMut<Inspectors<FilterQueryInspectorActs>>,
                     mut minibuffer: Minibuffer| {
                         if let Ok(index) = trigger.event_mut().take_result().unwrap() {
                             assets.visible[index] = !assets.visible[index];
                         }
            });
    } else {
        minibuffer.message("No filter queries registered.");
    }
}

impl PluginGroup for FilterQueryInspectorActs {
    fn build(mut self) -> PluginGroupBuilder {
        self.warn_on_unused_acts();
        self.plugins.build()
    }
}
