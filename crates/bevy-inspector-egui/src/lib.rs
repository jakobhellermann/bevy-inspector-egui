pub mod driver_egui;
pub mod options;

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
