#![warn(
    clippy::undocumented_unsafe_blocks,
    clippy::missing_safety_doc,
    unsafe_op_in_unsafe_fn
)]
#![allow(clippy::needless_lifetimes)]

pub mod bevy_ecs_inspector;
pub mod egui_reflect_inspector;
pub mod inspector_egui_impls;
pub mod inspector_options;

mod egui_utils;
pub(crate) mod split_world_permission;

/// [`bevy_app::Plugin`] used to register default [`struct@InspectorOptions`] and [`InspectorEguiImpl`](crate::inspector_egui_impls::InspectorEguiImpl)s
pub struct DefaultInspectorConfigPlugin;
impl bevy_app::Plugin for DefaultInspectorConfigPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        if app.is_plugin_added::<Self>() {
            return;
        }

        let type_registry = app.world.resource::<bevy_app::AppTypeRegistry>();
        let mut type_registry = type_registry.write();

        inspector_options::default_options::register_default_options(&mut type_registry);
        inspector_egui_impls::register_std_impls(&mut type_registry);
        inspector_egui_impls::register_glam_impls(&mut type_registry);
        inspector_egui_impls::register_bevy_impls(&mut type_registry);
    }
}

#[doc(inline)]
pub use inspector_options::InspectorOptions;

#[doc(hidden)]
pub mod __macro_exports {
    pub use bevy_reflect;
}

/// Reexports of commonly used types
pub mod prelude {
    // for `#[derive(Reflect)] #[reflect(InspectorOptions)]
    pub use crate::inspector_options::InspectorOptions;
    pub use crate::inspector_options::ReflectInspectorOptions;
}
