//! ```toml
//! [dependencies]
//! bevy-inspector-egui = "0.9"
//! bevy-inspector-egui-rapier = { version = "0.1", features = ["rapier3d"] }
//! ```
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_inspector_egui::WorldInspectorPlugin;
//! use bevy_inspector_egui_rapier::InspectableRapierPlugin;
//! use bevy_rapier3d::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(RapierDebugRenderPlugin::default())
//!         .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
//!         .add_plugin(InspectableRapierPlugin) // <--- register the inspectable UI functions for rapier types
//!         .add_plugin(WorldInspectorPlugin::default())
//!         .run();
//! }
//! ```

mod macros;

use bevy::prelude::{App, Plugin};
use bevy_inspector_egui::InspectableRegistry;

/// Plugin that will add register rapier components on the [`InspectableRegistry`]
pub struct InspectableRapierPlugin;

#[cfg(all(feature = "rapier2d", feature = "rapier3d"))]
compile_error!("enabling both `rapier2d` and `rapier3d` is not supported");

#[cfg_attr(
    all(not(feature = "rapier2d"), not(feature = "rapier3d")),
    allow(unreachable_code, unused_variables)
)]
impl Plugin for InspectableRapierPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(all(not(feature = "rapier2d"), not(feature = "rapier3d")))]
        panic!(
            "adding `InspectableRapierPlugin` but neither rapier2d nor rapier3d feature is enabled"
        );
        let mut inspectable_registry = app
            .world
            .get_resource_or_insert_with(InspectableRegistry::default);

        #[cfg(feature = "rapier2d")]
        rapier_2d::register(&mut inspectable_registry);
        #[cfg(feature = "rapier3d")]
        rapier_3d::register(&mut inspectable_registry);
    }
}

#[cfg(feature = "rapier2d")]
mod rapier_2d {
    use bevy_rapier2d as bevy_rapier;

    type Vect = bevy::math::Vec2;
    type VectAttributes = bevy_inspector_egui::options::Vec2dAttributes;

    include!("./rapier_impl.rs");
}

#[cfg(feature = "rapier3d")]
mod rapier_3d {
    use bevy_rapier3d as bevy_rapier;

    type Vect = bevy::math::Vec3;
    type VectAttributes = bevy_inspector_egui::options::NumberAttributes<Vect>;

    include!("./rapier_impl.rs");
}
