mod macros;

use bevy::prelude::{App, Plugin};
use bevy_inspector_egui::{InspectableRegistry, WorldInspectorParams};

pub struct InspectableRapierPlugin;

#[cfg(all(not(feature = "rapier2d"), not(feature = "rapier3d")))]
compile_error!("please select either the rapier2d or the rapier3d feature of the crate bevy-inspector-egui-rapier");

impl Plugin for InspectableRapierPlugin {
    fn build(&self, app: &mut App) {
        #[allow(unused_mut)]
        let mut inspectable_registry = app
            .world
            .get_resource_or_insert_with(InspectableRegistry::default);

        #[cfg(feature = "rapier2d")]
        rapier_2d::register(&mut inspectable_registry);
        #[cfg(feature = "rapier3d")]
        rapier_3d::register(&mut inspectable_registry);

        #[allow(unused_mut)]
        let mut world_inspector_params = app
            .world
            .get_resource_or_insert_with(WorldInspectorParams::default);

        #[cfg(feature = "rapier2d")]
        rapier_2d::register_params(&mut world_inspector_params);
        #[cfg(feature = "rapier3d")]
        rapier_3d::register_params(&mut world_inspector_params);
    }
}

#[cfg(feature = "rapier2d")]
mod rapier_2d {
    use bevy_rapier2d::prelude::*;

    include!("./rapier_impl.rs");
}

#[cfg(feature = "rapier3d")]
mod rapier_3d {
    use bevy_rapier3d::prelude::*;

    include!("./rapier_impl.rs");
}
