use crate::quick::WorldInspectorPlugin;
use bevy_app::{App, Plugin};
use bevy_ecs::{
    prelude::{Res, ResMut},
    schedule::Condition,
};
use bevy_minibuffer::{prelude::*, prompt::PromptState};
use bevy_reflect::Reflect;
use bevy_state::app::AppExtStates;
use bevy_state::prelude::in_state;
use bevy_state::prelude::{NextState, State, States};

mod inspector_plugins;
pub(crate) use inspector_plugins::*;
mod resource_inspector;
pub use resource_inspector::*;
mod asset_inspector;
pub use asset_inspector::*;
mod state_inspector;
pub use state_inspector::*;
mod filter_query_inspector;
pub use filter_query_inspector::*;

/// Is the prompt visible?
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States, Reflect)]
pub enum WorldInspectorState {
    /// Invisible
    #[default]
    Invisible,
    /// Visible
    Visible,
}

pub struct WorldInspectorActs {
    pub acts: Acts,
}

impl ActsPlugin for WorldInspectorActs {
    fn acts(&self) -> &Acts {
        &self.acts
    }
    fn acts_mut(&mut self) -> &mut Acts {
        &mut self.acts
    }
}

fn world_inspector(
    state: Res<State<WorldInspectorState>>,
    mut next_state: ResMut<NextState<WorldInspectorState>>,
    mut minibuffer: Minibuffer,
) {
    use WorldInspectorState::*;
    let (state, msg) = match state.get() {
        Invisible => (Visible, "Show world inspector"),
        Visible => (Invisible, "Hide world inspector"),
    };
    next_state.set(state);
    minibuffer.message(msg);
}

impl Default for WorldInspectorActs {
    fn default() -> Self {
        Self {
            acts: Acts::new([Act::new(world_inspector)]),
        }
    }
}

impl Plugin for WorldInspectorActs {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            WorldInspectorPlugin::default()
                .run_if(in_state(PromptState::Visible).and(in_state(WorldInspectorState::Visible))),
        )
        .init_state::<WorldInspectorState>();
        self.warn_on_unused_acts();
    }
}
