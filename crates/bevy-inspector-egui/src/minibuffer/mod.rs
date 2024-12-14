//! # bevy_minibuffer integration
//!
//! This module integrates with
//! [bevy_minibuffer](https://github.com/shanecelis/bevy_minibuffer) to provide
//! another means of invoking inspectors.
//!
//! ## Acts
//!
//! The minibuffer acts, i.e., commands this module makes available are:
//! - world_inspector
//! - resource_inspector
//! - asset_inspector
//! - state_inspector
//! - filter_query_inspector
//!
//! They may be used _a la carte_.
//!
//! ## Interaction
//!
//! Here is what interaction for 'world_insepctor' might look like:
//!
//! User types ':' or 'Alt-X', a black bar and prompt appear at the bottom of
//! the screen---that's the minibuffer. The user types 'world Tab Return' to tab
//! complete the 'world_inspector' act. The world inspector appears. If the user
//! hits the 'BackTick' (`) key, the minibuffer will disappear and so will the
//! inspector. Hit the 'BackTick' key again, and both reappear.
//!
//! ## Configuration
//!
//! The `WorldInspectorActs` provides 'world_inspector' act and it is the only
//! one that does not require any type registration.
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
//!
//! ### Type Registration
//!
//! Each of the other acts do require type registration. For instance, the
//! `AssetInspectorActs` provides 'asset_inspector' but requires registration of
//! what assets it should prompt for when the act is invoked.
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
//!             minibuffer::AssetInspectorActs::default()
//!                 .add::<StandardMaterial>()
//!         ));
//! }
//! ```
//!
//! There may be ways to automatically register various assets, resources, but I
//! would actually decline to do that. It can quickly make a mess, become
//! overwhelming, and takes control out of the user's hands.
//!
//! ## Visibility
//!
//! Each act toggles the visibility of an inspector. However, each inspector's
//! visibility is tied to minibuffer's visibility. When minibuffer is invisible
//! so are its inspectors and vice versa.
//!
//! NOTE: Any inspectors configured without this module are independent of
//! minibuffer's influence, so that's one escape hatch to this behavior.

mod inspector_plugins;
pub(crate) use inspector_plugins::*;
mod world_inspector;
pub use world_inspector::*;
mod resource_inspector;
pub use resource_inspector::*;
mod asset_inspector;
pub use asset_inspector::*;
mod state_inspector;
pub use state_inspector::*;
mod filter_query_inspector;
pub use filter_query_inspector::*;
