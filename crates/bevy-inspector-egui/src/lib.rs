#![warn(
    clippy::undocumented_unsafe_blocks,
    clippy::missing_safety_doc,
    unsafe_op_in_unsafe_fn
)]
#![allow(clippy::needless_lifetimes)]

mod bevy_default_options;
pub mod bevy_ecs_inspector;
pub mod driver_egui;
pub mod options;

pub fn setup_default_inspector_config(world: &bevy_ecs::world::World) {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let type_registry = world.resource::<bevy_app::AppTypeRegistry>();
        let mut type_registry = type_registry.write();

        bevy_default_options::register_default_options(&mut type_registry);
        driver_egui::inspector_egui_impls::register_default_impls(&mut type_registry);
    });
}

mod egui_utils;

pub use bevy_inspector_egui_derive::InspectorOptions;
pub use options::InspectorOptions;

#[doc(hidden)]
pub mod __macro_exports {
    pub use bevy_reflect;
}

pub mod prelude {
    // for `#[derive(Reflect)] #[reflect(InspectorOptions)]
    pub use crate::options::ReflectInspectorOptions;
    pub use crate::InspectorOptions;
}
