use bevy_app::{App, Plugin, PluginGroup, PluginGroupBuilder};
use bevy_state::prelude::in_state;
use bevy_minibuffer::{prelude::*, prompt::PromptState};
use crate::{
    quick::{WorldInspectorPlugin, AssetInspectorPlugin},
    utils::pretty_type_name,
};
use bevy_asset::Asset;
use bevy_ecs::{prelude::{Resource, Res, ResMut, World, Trigger}, schedule::Condition};
use bevy_state::app::AppExtStates;
use bevy_state::prelude::{State, NextState, States};
use bevy_egui::EguiContext;
use bevy_reflect::Reflect;
use trie_rs::map::Trie;

#[derive(Resource)]
pub(crate) struct AssetInspectors {
    pub(crate) names: Trie<u8, usize>,
    pub(crate) visible: Vec<bool>,
}

pub struct AssetInspectorActs {
    plugins: Option<PluginGroupBuilder>,
    asset_names: Vec<String>,
    acts: Acts,
}

impl ActsPluginGroup for AssetInspectorActs {
    fn acts(&self) -> &Acts {
        &self.acts
    }

    fn acts_mut(&mut self) -> &mut Acts {
        &mut self.acts
    }
}

impl AssetInspectorActs {
    pub fn add<R: Asset + Reflect>(mut self) -> Self {
        let index = self.asset_names.len();
        self.asset_names.push(pretty_type_name::<R>());
        self.add_plugin(asset_inspector_plugin::<R>(index));
        self
    }

    fn add_plugin<T: Plugin>(&mut self, plugin: T) {
        let builder = self.plugins.take().expect("plugin builder");
        self.plugins = Some(builder.add(plugin));
    }
}

impl Default for AssetInspectorActs {
    fn default() -> Self {
        Self {
            plugins: Some(PluginGroupBuilder::start::<Self>()),
            asset_names: vec![],
            acts: Acts::new([Act::new(asset_inspector)]),
        }
    }
}

fn asset_inspector(assets: Res<AssetInspectors>,
                   mut minibuffer: Minibuffer) {
    if !assets.visible.is_empty() {
        minibuffer.prompt_map("asset: ", assets.names.clone())
            .observe(|mut trigger: Trigger<Completed<usize>>,
                     mut assets: ResMut<AssetInspectors>,
                     mut minibuffer: Minibuffer| {
                         if let Ok(index) = trigger.event_mut().take_result().unwrap() {
                             assets.visible[index] = !assets.visible[index];
                         }
            });
    } else {
        minibuffer.message("No assets registered.");
    }
}


impl PluginGroup for AssetInspectorActs {
    fn build(mut self) -> PluginGroupBuilder {
        let builder = self.plugins.take().expect("plugin builder");
        self.warn_on_unused_acts();
        builder.add(move |app: &mut App| {
            let count = self.asset_names.len();
            let trie = Trie::from_iter(self.asset_names.clone().into_iter().enumerate().map(|(index, name)| (name, index)));
            app.insert_resource(AssetInspectors {
                names: trie,
                visible: vec![false; count],
            });
        })
    }
}

fn asset_visible(index: usize) -> impl Fn(Res<AssetInspectors>) -> bool {
    move |inspectors: Res<AssetInspectors>| {
        inspectors.visible.get(index).copied().unwrap_or(false)
    }
}

fn asset_inspector_plugin<A: Asset + Reflect>(index: usize) -> impl Fn(&mut App) {
    move |app: &mut App| {
        app.add_plugins(AssetInspectorPlugin::<A>::default()
                    .run_if(in_state(PromptState::Visible).and(asset_visible(index)))
        );
    }
}
