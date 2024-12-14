use crate::{
    minibuffer::{InspectorPlugins, Inspectors},
    quick::AssetInspectorPlugin,
    utils::pretty_type_name,
};
use bevy_app::{PluginGroup, PluginGroupBuilder};
use bevy_asset::Asset;
use bevy_ecs::{
    prelude::{Res, ResMut, Trigger},
    schedule::Condition,
};
use bevy_minibuffer::{prelude::*, prompt::PromptState};
use bevy_reflect::Reflect;
use bevy_state::prelude::in_state;

/// ## Adds the 'asset_inspector' act
///
/// This act toggles the visibility of added asset inspectors.
///
/// ## Usage
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_minibuffer::prelude::*;
/// use bevy_inspector_egui::minibuffer;
/// fn plugin(app: &mut App) {
///     app
///         .add_plugins(MinibufferPlugins)
///         .add_acts((
///             BasicActs::default(),
///             minibuffer::AssetInspectorActs::default()
///                 .add::<StandardMaterial>()
///         ));
/// }
/// ```
pub struct AssetInspectorActs {
    plugins: InspectorPlugins<Self>,
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
    /// Add an asset to be shown when prompted.
    pub fn add<A: Asset + Reflect>(mut self) -> Self {
        self.plugins
            .add_inspector(pretty_type_name::<A>(), Self::asset_inspector_plugin::<A>);
        self
    }

    fn asset_inspector_plugin<A: Asset + Reflect>(
        index: usize,
        inspector_plugins: &mut InspectorPlugins<Self>,
    ) {
        inspector_plugins.add_plugin(
            AssetInspectorPlugin::<A>::default().run_if(
                in_state(PromptState::Visible).and(InspectorPlugins::<Self>::visible(index)),
            ),
        );
    }
}

impl Default for AssetInspectorActs {
    fn default() -> Self {
        Self {
            plugins: InspectorPlugins::default(),
            acts: Acts::new([Act::new(asset_inspector)]),
        }
    }
}

fn asset_inspector(assets: Res<Inspectors<AssetInspectorActs>>, mut minibuffer: Minibuffer) {
    if !assets.visible.is_empty() {
        minibuffer
            .prompt_map("asset: ", assets.names.clone())
            .observe(
                |mut trigger: Trigger<Completed<usize>>,
                 mut assets: ResMut<Inspectors<AssetInspectorActs>>| {
                    if let Ok(index) = trigger.event_mut().take_result().unwrap() {
                        assets.visible[index] = !assets.visible[index];
                    }
                },
            );
    } else {
        minibuffer.message("No assets registered.");
    }
}

impl PluginGroup for AssetInspectorActs {
    fn build(self) -> PluginGroupBuilder {
        self.warn_on_unused_acts();
        self.plugins
            .warn_on_empty("No assets registered with `AssetInspectorActs`; consider adding some.");
        self.plugins.build()
    }
}
