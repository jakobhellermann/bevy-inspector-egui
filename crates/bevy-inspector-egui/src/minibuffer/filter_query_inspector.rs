//! # filter_query_inspector act
//!
//! ## Usage
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_minibuffer::prelude::*;
//! use bevy_inspector_egui::minibuffer;
//! fn plugin(app: &mut App) {
//!     app
//!         .add_plugins(MinibufferPlugins)
//!         .add_acts((
//!             BasicActs::default(),
//!             minibuffer::FilterQueryInspectorActs::default()
//!                 .add::<With<Transform>>()
//!                 .add::<With<Mesh3d>>()
//!         ));
//! }
//! ```
use crate::{
    minibuffer::{InspectorPlugins, Inspectors},
    quick::FilterQueryInspectorPlugin,
    utils::pretty_type_name,
};
use bevy_app::{PluginGroup, PluginGroupBuilder};
use bevy_ecs::{
    prelude::{Res, ResMut, Trigger},
    query::QueryFilter,
    schedule::Condition,
};
use bevy_minibuffer::{prelude::*, prompt::PromptState};
use bevy_state::prelude::in_state;

/// Adds the 'filter_query_inspector' act which toggles the visibility of the
/// added filter query inspectors.
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
        self.plugins.add_inspector(
            pretty_type_name::<A>(),
            Self::filter_query_inspector_plugin::<A>,
        );
        self
    }

    fn filter_query_inspector_plugin<A: QueryFilter + 'static>(
        index: usize,
        inspector_plugins: &mut InspectorPlugins<Self>,
    ) {
        inspector_plugins.add_plugin(
            FilterQueryInspectorPlugin::<A>::default().run_if(
                in_state(PromptState::Visible).and(InspectorPlugins::<Self>::visible(index)),
            ),
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

fn filter_query_inspector(
    assets: Res<Inspectors<FilterQueryInspectorActs>>,
    mut minibuffer: Minibuffer,
) {
    if !assets.visible.is_empty() {
        minibuffer
            .prompt_map("filter query: ", assets.names.clone())
            .observe(
                |mut trigger: Trigger<Completed<usize>>,
                 mut assets: ResMut<Inspectors<FilterQueryInspectorActs>>| {
                    if let Ok(index) = trigger.event_mut().take_result().unwrap() {
                        assets.visible[index] = !assets.visible[index];
                    }
                },
            );
    } else {
        minibuffer.message("No filter queries registered.");
    }
}

impl PluginGroup for FilterQueryInspectorActs {
    fn build(self) -> PluginGroupBuilder {
        self.warn_on_unused_acts();
        self.plugins.build()
    }
}
