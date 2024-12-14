use crate::{
    minibuffer::{InspectorPlugins, Inspectors},
    quick::StateInspectorPlugin,
    utils::pretty_type_name,
};
use bevy_app::{PluginGroup, PluginGroupBuilder};
use bevy_ecs::{
    prelude::{Res, ResMut, Trigger},
    schedule::Condition,
};
use bevy_minibuffer::{prelude::*, prompt::PromptState};
use bevy_reflect::Reflect;
use bevy_state::{prelude::in_state, state::FreelyMutableState};

/// ## Adds the 'state_inspector' act
///
/// This act toggles the visibility of registered state inspectors.
///
/// ## Usage
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_minibuffer::prelude::*;
/// use bevy_inspector_egui::minibuffer;
/// #[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash, Reflect)]
/// enum AppState {
///     #[default]
///     A,
///     B,
///     C,
/// }
/// fn plugin(app: &mut App) {
///     app
///         .add_plugins(MinibufferPlugins)
///         .add_acts((
///             BasicActs::default(),
///             minibuffer::StateInspectorActs::default()
///                 .add::<AppState>()
///         ));
/// }
/// ```
pub struct StateInspectorActs {
    plugins: InspectorPlugins<Self>,
    acts: Acts,
}

impl PluginGroup for StateInspectorActs {
    fn build(self) -> PluginGroupBuilder {
        self.warn_on_unused_acts();
        self.plugins.warn_on_empty("No states registered with `StateInspectorActs`; consider adding some.");
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
    /// Add a state to the list of inspectors when prompted.
    pub fn add<S: FreelyMutableState + Reflect>(mut self) -> Self {
        self.plugins
            .add_inspector(pretty_type_name::<S>(), Self::add_plugin::<S>);
        self
    }

    fn add_plugin<A: FreelyMutableState + Reflect>(
        index: usize,
        inspector_plugins: &mut InspectorPlugins<Self>,
    ) {
        inspector_plugins.add_plugin(
            StateInspectorPlugin::<A>::default().run_if(
                in_state(PromptState::Visible).and(InspectorPlugins::<Self>::visible(index)),
            ),
        );
    }
}

fn state_inspector(states: Res<Inspectors<StateInspectorActs>>, mut minibuffer: Minibuffer) {
    if !states.visible.is_empty() {
        minibuffer
            .prompt_map("state: ", states.names.clone())
            .observe(
                |mut trigger: Trigger<Completed<usize>>,
                 mut states: ResMut<Inspectors<StateInspectorActs>>| {
                    if let Ok(index) = trigger.event_mut().take_result().unwrap() {
                        states.visible[index] = !states.visible[index];
                    }
                },
            );
    } else {
        minibuffer.message("No states registered.");
    }
}

impl Default for StateInspectorActs {
    fn default() -> Self {
        Self {
            plugins: InspectorPlugins::default(),
            acts: Acts::new([Act::new(state_inspector)]),
            // acts: Acts::new([Act::new(InspectorPlugins::<StateInspectorActs>::inspector("state: ", "No states registered"))
            // .named("state_inspector")])
        }
    }
}
