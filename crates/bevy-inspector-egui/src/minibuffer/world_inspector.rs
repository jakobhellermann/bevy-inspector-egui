//! # world_inspector act
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
//!             minibuffer::WorldInspectorActs::default(),
//!         ));
//! }
//! ```
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

/// Is the prompt visible?
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States, Reflect)]
enum WorldInspectorState {
    /// Invisible
    #[default]
    Invisible,
    /// Visible
    Visible,
}

/// Provides the 'world_inspector' act that toggles the visibility of the world
/// inspector.
pub struct WorldInspectorActs {
    acts: Acts,
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
