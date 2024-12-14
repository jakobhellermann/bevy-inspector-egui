//! # resource_inspector act
//!
//! ## Usage
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_minibuffer::prelude::*;
//! use bevy_inspector_egui::minibuffer;
//! #[derive(Resource, Reflect)]
//! struct R1;
//! #[derive(Resource, Reflect)]
//! struct R2;
//! fn plugin(app: &mut App) {
//!     app
//!         .add_plugins(MinibufferPlugins)
//!         .add_acts((
//!             BasicActs::default(),
//!             minibuffer::ResourceInspectorActs::default()
//!                 .add::<R1>()
//!                 .add::<R2>()
//!         ));
//! }
//! ```
use crate::{
    minibuffer::{InspectorPlugins, Inspectors},
    quick::ResourceInspectorPlugin,
    utils::pretty_type_name,
};
use bevy_app::{PluginGroup, PluginGroupBuilder};
use bevy_ecs::{
    prelude::{Res, ResMut, Resource, Trigger},
    schedule::Condition,
};
use bevy_minibuffer::{prelude::*, prompt::PromptState};
use bevy_reflect::Reflect;
use bevy_state::prelude::in_state;

/// Adds the 'resource_inspector' act which toggles the visibility of resource
/// inspectors that were added.
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
    /// Add a resource to the list of resources when prompted.
    pub fn add<R: Resource + Reflect>(mut self) -> Self {
        self.plugins.add_inspector(
            pretty_type_name::<R>(),
            Self::resource_inspector_plugin::<R>,
        );
        self
    }

    fn resource_inspector_plugin<R: Resource + Reflect>(
        index: usize,
        inspector_plugins: &mut InspectorPlugins<Self>,
    ) {
        inspector_plugins.add_plugin(
            ResourceInspectorPlugin::<R>::default().run_if(
                in_state(PromptState::Visible).and(InspectorPlugins::<Self>::visible(index)),
            ),
        );
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

fn resource_inspector(
    resources: Res<Inspectors<ResourceInspectorActs>>,
    mut minibuffer: Minibuffer,
) {
    if !resources.visible.is_empty() {
        minibuffer
            .prompt_map("resource: ", resources.names.clone())
            .observe(
                |mut trigger: Trigger<Completed<usize>>,
                 mut resources: ResMut<Inspectors<ResourceInspectorActs>>| {
                    if let Ok(index) = trigger.event_mut().take_result().unwrap() {
                        resources.visible[index] = !resources.visible[index];
                    }
                },
            );
    } else {
        minibuffer.message("No resource inspectors available.");
    }
}

impl PluginGroup for ResourceInspectorActs {
    fn build(self) -> PluginGroupBuilder {
        self.warn_on_unused_acts();
        self.plugins.build()
    }
}
