use crate::egui_reflect_inspector::InspectorUi;
use bevy_reflect::TypeRegistry;
use std::{
    any::{Any, TypeId},
    time::Instant,
};

mod glam_impls;
mod image;
mod std_impls;

type InspectorEguiImplFn = fn(&mut dyn Any, &mut egui::Ui, &dyn Any, InspectorUi<'_, '_>) -> bool;

#[derive(Clone)]
pub struct InspectorEguiImpl {
    f: InspectorEguiImplFn,
}

impl InspectorEguiImpl {
    pub fn new(f: InspectorEguiImplFn) -> Self {
        InspectorEguiImpl { f }
    }

    pub(crate) fn execute<'a, 'c: 'a>(
        &'a self,
        value: &mut dyn Any,
        ui: &mut egui::Ui,
        options: &dyn Any,
        env: InspectorUi<'_, '_>,
    ) -> bool {
        (self.f)(value, ui, options, env)
    }
}

pub fn register_default_impls(type_registry: &mut TypeRegistry) {
    fn add<T: 'static>(type_registry: &mut TypeRegistry, f: InspectorEguiImplFn) {
        type_registry
            .get_mut(TypeId::of::<T>())
            .unwrap()
            .insert(InspectorEguiImpl::new(f));
    }
    add::<f32>(type_registry, std_impls::number_ui_subint::<f32>);
    add::<f64>(type_registry, std_impls::number_ui_subint::<f64>);
    add::<i8>(type_registry, std_impls::number_ui::<i8>);
    add::<i16>(type_registry, std_impls::number_ui::<i16>);
    add::<i32>(type_registry, std_impls::number_ui::<i32>);
    add::<i64>(type_registry, std_impls::number_ui::<i64>);
    add::<isize>(type_registry, std_impls::number_ui::<isize>);
    add::<u8>(type_registry, std_impls::number_ui::<u8>);
    add::<u16>(type_registry, std_impls::number_ui::<u16>);
    add::<u32>(type_registry, std_impls::number_ui::<u32>);
    add::<u64>(type_registry, std_impls::number_ui::<u64>);
    add::<usize>(type_registry, std_impls::number_ui::<usize>);
    add::<bool>(type_registry, std_impls::bool_ui);
    add::<String>(type_registry, std_impls::string_ui);
    add::<std::time::Duration>(type_registry, std_impls::duration_ui);
    add::<Instant>(type_registry, std_impls::instant_ui);

    add::<bevy_asset::Handle<bevy_render::texture::Image>>(type_registry, image::image_handle_ui);

    add::<bevy_math::Vec2>(type_registry, glam_impls::vec2_ui);
    add::<bevy_math::Vec3>(type_registry, glam_impls::vec3_ui);
    add::<bevy_math::Vec3A>(type_registry, glam_impls::vec3a_ui);
    add::<bevy_math::Vec4>(type_registry, glam_impls::vec4_ui);
    add::<bevy_math::UVec2>(type_registry, glam_impls::uvec2_ui);
    add::<bevy_math::UVec3>(type_registry, glam_impls::uvec3_ui);
    add::<bevy_math::UVec4>(type_registry, glam_impls::uvec4_ui);
    add::<bevy_math::IVec2>(type_registry, glam_impls::ivec2_ui);
    add::<bevy_math::IVec3>(type_registry, glam_impls::ivec3_ui);
    add::<bevy_math::IVec4>(type_registry, glam_impls::ivec4_ui);
    add::<bevy_math::DVec2>(type_registry, glam_impls::dvec2_ui);
    add::<bevy_math::DVec3>(type_registry, glam_impls::dvec3_ui);
    add::<bevy_math::DVec4>(type_registry, glam_impls::dvec4_ui);
    add::<bevy_math::BVec2>(type_registry, glam_impls::bvec2_ui);
    add::<bevy_math::BVec3>(type_registry, glam_impls::bvec3_ui);
    add::<bevy_math::BVec4>(type_registry, glam_impls::bvec4_ui);
    add::<bevy_math::Mat2>(type_registry, glam_impls::mat2_ui);
    add::<bevy_math::Mat3>(type_registry, glam_impls::mat3_ui);
    add::<bevy_math::Mat3A>(type_registry, glam_impls::mat3a_ui);
    add::<bevy_math::Mat4>(type_registry, glam_impls::mat4_ui);
    add::<bevy_math::DMat2>(type_registry, glam_impls::dmat2_ui);
    add::<bevy_math::DMat3>(type_registry, glam_impls::dmat3_ui);
    add::<bevy_math::DMat4>(type_registry, glam_impls::dmat4_ui);

    add::<bevy_math::Quat>(type_registry, glam_impls::quat::quat_ui);
}
