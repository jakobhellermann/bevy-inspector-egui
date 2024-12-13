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

#[derive(Resource)]
pub struct ResourceInspectors {
    names: Trie<u8, usize>,
    visible: Vec<bool>,
}

pub struct ResourceInspectorActs {
    plugins: Option<PluginGroupBuilder>,
    acts: Acts,
    resource_names: Vec<String>,
}

impl ActsPluginGroup for ResourceInspectorActs {
    fn acts(&self) -> &Acts {
        &self.acts
    }

    fn acts_mut(&mut self) -> &mut Acts {
        &mut self.acts
    }
}

// impl PluginGroup for ResourceInspectorActs {
//     fn

// }

impl ResourceInspectorActs {
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

impl Default for ResourceInspectorActs {
    fn default() -> Self {
        Self {
            plugins: Some(PluginGroupBuilder::start::<Self>()),
            acts: Acts::new([Act::new(resource_inspector)]),
            resource_names: vec![],
        }
    }
}

fn resource_inspector(resources: Option<Res<ResourceInspectors>>,
                      mut minibuffer: Minibuffer) {
    if let Some(ref resources) = resources {
        minibuffer.prompt_map("resource: ", resources.names.clone())
            .observe(|mut trigger: Trigger<Completed<usize>>,
                     mut resources: ResMut<ResourceInspectors>,
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
        let builder = self.plugins.take().expect("plugin builder");
        // let resource_names = self.resource_names.drain(..);
        self.warn_on_unused_acts();
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
