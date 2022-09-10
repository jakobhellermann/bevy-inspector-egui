#![warn(
    clippy::undocumented_unsafe_blocks,
    clippy::missing_safety_doc,
    unsafe_op_in_unsafe_fn
)]
#![allow(clippy::needless_lifetimes)]

pub mod bevy_ecs_inspector;
pub mod egui_reflect_inspector;
pub mod inspector_options;

mod egui_utils;
mod inspector_egui_impls;
pub(crate) mod split_world_permission;

pub struct DefaultInspectorConfigPlugin;
impl bevy_app::Plugin for DefaultInspectorConfigPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        let type_registry = app.world.resource::<bevy_app::AppTypeRegistry>();
        let mut type_registry = type_registry.write();

        inspector_options::default_options::register_default_options(&mut type_registry);
        inspector_egui_impls::register_default_impls(&mut type_registry);
    }
}

pub use bevy_inspector_egui_derive::InspectorOptions;
pub use inspector_options::InspectorOptions;

#[doc(hidden)]
pub mod __macro_exports {
    pub use bevy_reflect;
}

pub mod prelude {
    // for `#[derive(Reflect)] #[reflect(InspectorOptions)]
    pub use crate::inspector_options::ReflectInspectorOptions;
    pub use crate::InspectorOptions;
}
