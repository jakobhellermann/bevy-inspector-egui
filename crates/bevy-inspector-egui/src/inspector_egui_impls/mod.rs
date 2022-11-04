//! UI implementations for leaf types

use crate::egui_reflect_inspector::InspectorUi;
use bevy_reflect::TypeRegistry;
use bevy_utils::Instant;
use std::{
    any::{Any, TypeId},
    borrow::Cow,
};

mod color;
mod glam_impls;
mod image;
mod std_impls;

type InspectorEguiImplFn = fn(&mut dyn Any, &mut egui::Ui, &dyn Any, InspectorUi<'_, '_>) -> bool;
type InspectorEguiImplFnReadonly = fn(&dyn Any, &mut egui::Ui, &dyn Any, InspectorUi<'_, '_>);

/// Function pointers for displaying a concrete type, to be registered in the [`TypeRegistry`].
///
/// This can used for leaf types like `u8` or `String`, as well as people who want to completely customize the way
/// to display a certain type.
#[derive(Clone)]
pub struct InspectorEguiImpl {
    fn_mut: InspectorEguiImplFn,
    fn_readonly: InspectorEguiImplFnReadonly,
}

impl InspectorEguiImpl {
    /// Create a new [`InspectorEguiImpl`] from functions displaying a type
    pub fn new(fn_mut: InspectorEguiImplFn, fn_readonly: InspectorEguiImplFnReadonly) -> Self {
        InspectorEguiImpl {
            fn_mut,
            fn_readonly,
        }
    }

    pub fn execute<'a, 'c: 'a>(
        &'a self,
        value: &mut dyn Any,
        ui: &mut egui::Ui,
        options: &dyn Any,
        env: InspectorUi<'_, '_>,
    ) -> bool {
        (self.fn_mut)(value, ui, options, env)
    }
    pub fn execute_readonly<'a, 'c: 'a>(
        &'a self,
        value: &dyn Any,
        ui: &mut egui::Ui,
        options: &dyn Any,
        env: InspectorUi<'_, '_>,
    ) {
        (self.fn_readonly)(value, ui, options, env)
    }
}

fn add<T: 'static>(
    type_registry: &mut TypeRegistry,
    fn_mut: InspectorEguiImplFn,
    fn_readonly: InspectorEguiImplFnReadonly,
) {
    type_registry
        .get_mut(TypeId::of::<T>())
        .unwrap_or_else(|| panic!("{} not registered", std::any::type_name::<T>()))
        .insert(InspectorEguiImpl::new(fn_mut, fn_readonly));
}

/// Register [`InspectorEguiImpl`]s for primitive rust types as well as standard library types
#[rustfmt::skip]
pub fn register_std_impls(type_registry: &mut TypeRegistry) {
    add::<f32>(type_registry, std_impls::number_ui_subint::<f32>, std_impls::number_ui_readonly::<f32>);
    add::<f64>(type_registry, std_impls::number_ui_subint::<f64>, std_impls::number_ui_readonly::<f64>);
    add::<i8>(type_registry, std_impls::number_ui::<i8>, std_impls::number_ui_readonly::<i8>);
    add::<i16>(type_registry, std_impls::number_ui::<i16>, std_impls::number_ui_readonly::<i16>);
    add::<i32>(type_registry, std_impls::number_ui::<i32>, std_impls::number_ui_readonly::<i32>);
    add::<i64>(type_registry, std_impls::number_ui::<i64>, std_impls::number_ui_readonly::<i64>);
    add::<isize>(type_registry, std_impls::number_ui::<isize>, std_impls::number_ui_readonly::<isize>);
    add::<u8>(type_registry, std_impls::number_ui::<u8>, std_impls::number_ui_readonly::<u8>);
    add::<u16>(type_registry, std_impls::number_ui::<u16>, std_impls::number_ui_readonly::<u16>);
    add::<u32>(type_registry, std_impls::number_ui::<u32>, std_impls::number_ui_readonly::<u32>);
    add::<u64>(type_registry, std_impls::number_ui::<u64>, std_impls::number_ui_readonly::<u64>);
    add::<usize>(type_registry, std_impls::number_ui::<usize>, std_impls::number_ui_readonly::<usize>);
    add::<bool>(type_registry, std_impls::bool_ui, std_impls::bool_ui_readonly);
    add::<String>(type_registry, std_impls::string_ui, std_impls::string_ui_readonly);
    add::<Cow<str>>(type_registry, std_impls::cow_str_ui, std_impls::cow_str_ui_readonly);
    add::<std::time::Duration>(type_registry, std_impls::duration_ui, std_impls::duration_ui_readonly);
    add::<Instant>(type_registry, std_impls::instant_ui, std_impls::instant_ui_readonly);
}

/// Register [`InspectorEguiImpl`]s for [`bevy_math`](bevy_math)/`glam` types
#[rustfmt::skip]
pub fn register_glam_impls(type_registry: &mut TypeRegistry) {
    add::<bevy_math::Vec2>(type_registry, glam_impls::vec2_ui, glam_impls::vec2_ui_readonly);
    add::<bevy_math::Vec3>(type_registry, glam_impls::vec3_ui, glam_impls::vec3_ui_readonly);
    add::<bevy_math::Vec3A>(type_registry, glam_impls::vec3a_ui, glam_impls::vec3a_ui_readonly);
    add::<bevy_math::Vec4>(type_registry, glam_impls::vec4_ui, glam_impls::vec4_ui_readonly);
    add::<bevy_math::UVec2>(type_registry, glam_impls::uvec2_ui, glam_impls::uvec2_ui_readonly);
    add::<bevy_math::UVec3>(type_registry, glam_impls::uvec3_ui, glam_impls::uvec3_ui_readonly);
    add::<bevy_math::UVec4>(type_registry, glam_impls::uvec4_ui, glam_impls::uvec4_ui_readonly);
    add::<bevy_math::IVec2>(type_registry, glam_impls::ivec2_ui, glam_impls::ivec2_ui_readonly);
    add::<bevy_math::IVec3>(type_registry, glam_impls::ivec3_ui, glam_impls::ivec3_ui_readonly);
    add::<bevy_math::IVec4>(type_registry, glam_impls::ivec4_ui, glam_impls::ivec4_ui_readonly);
    add::<bevy_math::DVec2>(type_registry, glam_impls::dvec2_ui, glam_impls::dvec2_ui_readonly);
    add::<bevy_math::DVec3>(type_registry, glam_impls::dvec3_ui, glam_impls::dvec3_ui_readonly);
    add::<bevy_math::DVec4>(type_registry, glam_impls::dvec4_ui, glam_impls::dvec4_ui_readonly);
    add::<bevy_math::BVec2>(type_registry, glam_impls::bvec2_ui, glam_impls::bvec2_ui_readonly);
    add::<bevy_math::BVec3>(type_registry, glam_impls::bvec3_ui, glam_impls::bvec3_ui_readonly);
    add::<bevy_math::BVec4>(type_registry, glam_impls::bvec4_ui, glam_impls::bvec4_ui_readonly);
    add::<bevy_math::Mat2>(type_registry, glam_impls::mat2_ui, glam_impls::mat2_ui_readonly);
    add::<bevy_math::Mat3>(type_registry, glam_impls::mat3_ui, glam_impls::mat3_ui_readonly);
    add::<bevy_math::Mat3A>(type_registry, glam_impls::mat3a_ui, glam_impls::mat3a_ui_readonly);
    add::<bevy_math::Mat4>(type_registry, glam_impls::mat4_ui, glam_impls::mat4_ui_readonly);
    add::<bevy_math::DMat2>(type_registry, glam_impls::dmat2_ui, glam_impls::dmat2_ui_readonly);
    add::<bevy_math::DMat3>(type_registry, glam_impls::dmat3_ui, glam_impls::dmat3_ui_readonly);
    add::<bevy_math::DMat4>(type_registry, glam_impls::dmat4_ui, glam_impls::dmat4_ui_readonly);

    add::<bevy_math::Quat>(type_registry, glam_impls::quat::quat_ui, glam_impls::quat::quat_ui_readonly);
}

/// Register [`InspectorEguiImpl`]s for `bevy` types
#[rustfmt::skip]
pub fn register_bevy_impls(type_registry: &mut TypeRegistry) {
    add::<bevy_asset::Handle<bevy_render::texture::Image>>(type_registry, image::image_handle_ui, image::image_handle_ui_readonly);
    add::<bevy_render::color::Color>(type_registry, color::color_ui, color::color_ui_readonly);
}
