//! UI implementations for leaf types

use crate::egui_reflect_inspector::{errors::error_message_no_multiedit, InspectorUi};
use bevy_reflect::{Reflect, TypeRegistry};
use bevy_utils::Instant;
use std::{
    any::{Any, TypeId},
    borrow::Cow,
};

mod bevy_impls;
mod glam_impls;
mod image;
mod std_impls;

type InspectorEguiImplFn =
    fn(&mut dyn Any, &mut egui::Ui, &dyn Any, egui::Id, InspectorUi<'_, '_>) -> bool;
type InspectorEguiImplFnReadonly =
    fn(&dyn Any, &mut egui::Ui, &dyn Any, egui::Id, InspectorUi<'_, '_>);
type InspectorEguiImplFnMany = for<'a> fn(
    &mut egui::Ui,
    &dyn Any,
    egui::Id,
    InspectorUi<'_, '_>,
    &mut [&mut dyn Reflect],
    &dyn Fn(&mut dyn Reflect) -> &mut dyn Reflect,
) -> bool;

/// Function pointers for displaying a concrete type, to be registered in the [`TypeRegistry`].
///
/// This can used for leaf types like `u8` or `String`, as well as people who want to completely customize the way
/// to display a certain type.
#[derive(Clone)]
pub struct InspectorEguiImpl {
    fn_mut: InspectorEguiImplFn,
    fn_readonly: InspectorEguiImplFnReadonly,
    fn_many: InspectorEguiImplFnMany,
}

impl InspectorEguiImpl {
    /// Create a new [`InspectorEguiImpl`] from functions displaying a type
    pub fn new(
        fn_mut: InspectorEguiImplFn,
        fn_readonly: InspectorEguiImplFnReadonly,
        fn_many: InspectorEguiImplFnMany,
    ) -> Self {
        InspectorEguiImpl {
            fn_mut,
            fn_readonly,
            fn_many,
        }
    }

    pub fn execute<'a, 'c: 'a>(
        &'a self,
        value: &mut dyn Any,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) -> bool {
        (self.fn_mut)(value, ui, options, id, env)
    }
    pub fn execute_readonly<'a, 'c: 'a>(
        &'a self,
        value: &dyn Any,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) {
        (self.fn_readonly)(value, ui, options, id, env)
    }
    pub fn execute_many<'a, 'c: 'a, 'e>(
        &'a self,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
        values: &mut [&mut dyn Reflect],
        projector: &dyn Fn(&mut dyn Reflect) -> &mut dyn Reflect,
    ) -> bool {
        (self.fn_many)(ui, options, id, env, values, projector)
    }
}

fn many_unimplemented<T: Any>(
    ui: &mut egui::Ui,
    _options: &dyn Any,
    _id: egui::Id,
    _env: InspectorUi<'_, '_>,
    _values: &mut [&mut dyn Reflect],
    _projector: &dyn Fn(&mut dyn Reflect) -> &mut dyn Reflect,
) -> bool {
    error_message_no_multiedit(ui, &pretty_type_name::pretty_type_name::<T>());
    false
}

fn add_no_many<T: 'static>(
    type_registry: &mut TypeRegistry,
    fn_mut: InspectorEguiImplFn,
    fn_readonly: InspectorEguiImplFnReadonly,
) {
    type_registry
        .get_mut(TypeId::of::<T>())
        .unwrap_or_else(|| panic!("{} not registered", std::any::type_name::<T>()))
        .insert(InspectorEguiImpl::new(
            fn_mut,
            fn_readonly,
            many_unimplemented::<T>,
        ));
}
fn add<T: 'static>(
    type_registry: &mut TypeRegistry,
    fn_mut: InspectorEguiImplFn,
    fn_readonly: InspectorEguiImplFnReadonly,
    fn_many: InspectorEguiImplFnMany,
) {
    type_registry
        .get_mut(TypeId::of::<T>())
        .unwrap_or_else(|| panic!("{} not registered", std::any::type_name::<T>()))
        .insert(InspectorEguiImpl::new(fn_mut, fn_readonly, fn_many));
}

/// Register [`InspectorEguiImpl`]s for primitive rust types as well as standard library types
#[rustfmt::skip]
pub fn register_std_impls(type_registry: &mut TypeRegistry) {
    add::<f32>(type_registry, std_impls::number_ui_subint::<f32>, std_impls::number_ui_readonly::<f32>, std_impls::number_ui_many::<f32>);
    add::<f64>(type_registry, std_impls::number_ui_subint::<f64>, std_impls::number_ui_readonly::<f64>, std_impls::number_ui_many::<f64>);
    add::<i8>(type_registry, std_impls::number_ui::<i8>, std_impls::number_ui_readonly::<i8>, std_impls::number_ui_many::<i8>);
    add::<i16>(type_registry, std_impls::number_ui::<i16>, std_impls::number_ui_readonly::<i16>, std_impls::number_ui_many::<i16>);
    add::<i32>(type_registry, std_impls::number_ui::<i32>, std_impls::number_ui_readonly::<i32>, std_impls::number_ui_many::<i32>);
    add::<i64>(type_registry, std_impls::number_ui::<i64>, std_impls::number_ui_readonly::<i64>, std_impls::number_ui_many::<i64>);
    add::<isize>(type_registry, std_impls::number_ui::<isize>, std_impls::number_ui_readonly::<isize>, std_impls::number_ui_many::<isize>);
    add::<u8>(type_registry, std_impls::number_ui::<u8>, std_impls::number_ui_readonly::<u8>, std_impls::number_ui_many::<u8>);
    add::<u16>(type_registry, std_impls::number_ui::<u16>, std_impls::number_ui_readonly::<u16>, std_impls::number_ui_many::<u16>);
    add::<u32>(type_registry, std_impls::number_ui::<u32>, std_impls::number_ui_readonly::<u32>, std_impls::number_ui_many::<u32>);
    add::<u64>(type_registry, std_impls::number_ui::<u64>, std_impls::number_ui_readonly::<u64>, std_impls::number_ui_many::<u64>);
    add::<usize>(type_registry, std_impls::number_ui::<usize>, std_impls::number_ui_readonly::<usize>, std_impls::number_ui_many::<usize>);
    add::<bool>(type_registry, std_impls::bool_ui, std_impls::bool_ui_readonly, std_impls::bool_ui_many);
    add::<String>(type_registry, std_impls::string_ui, std_impls::string_ui_readonly, std_impls::string_ui_many);
    add::<Cow<str>>(type_registry, std_impls::cow_str_ui, std_impls::cow_str_ui_readonly, std_impls::cow_str_ui_many);
    add_no_many::<std::time::Duration>(type_registry, std_impls::duration_ui, std_impls::duration_ui_readonly);
    add_no_many::<Instant>(type_registry, std_impls::instant_ui, std_impls::instant_ui_readonly);
}

/// Register [`InspectorEguiImpl`]s for [`bevy_math`](bevy_math)/`glam` types
#[rustfmt::skip]
pub fn register_glam_impls(type_registry: &mut TypeRegistry) {
    add::<bevy_math::Vec2>(type_registry, glam_impls::vec2_ui, glam_impls::vec2_ui_readonly, glam_impls::vec2_ui_many);
    add::<bevy_math::Vec3>(type_registry, glam_impls::vec3_ui, glam_impls::vec3_ui_readonly, glam_impls::vec3_ui_many);
    add::<bevy_math::Vec3A>(type_registry, glam_impls::vec3a_ui, glam_impls::vec3a_ui_readonly, glam_impls::vec3a_ui_many);
    add::<bevy_math::Vec4>(type_registry, glam_impls::vec4_ui, glam_impls::vec4_ui_readonly, glam_impls::vec4_ui_many);
    add::<bevy_math::UVec2>(type_registry, glam_impls::uvec2_ui, glam_impls::uvec2_ui_readonly, glam_impls::uvec2_ui_many);
    add::<bevy_math::UVec3>(type_registry, glam_impls::uvec3_ui, glam_impls::uvec3_ui_readonly, glam_impls::uvec3_ui_many);
    add::<bevy_math::UVec4>(type_registry, glam_impls::uvec4_ui, glam_impls::uvec4_ui_readonly, glam_impls::uvec4_ui_many);
    add::<bevy_math::IVec2>(type_registry, glam_impls::ivec2_ui, glam_impls::ivec2_ui_readonly, glam_impls::ivec2_ui_many);
    add::<bevy_math::IVec3>(type_registry, glam_impls::ivec3_ui, glam_impls::ivec3_ui_readonly, glam_impls::ivec3_ui_many);
    add::<bevy_math::IVec4>(type_registry, glam_impls::ivec4_ui, glam_impls::ivec4_ui_readonly, glam_impls::ivec4_ui_many);
    add::<bevy_math::DVec2>(type_registry, glam_impls::dvec2_ui, glam_impls::dvec2_ui_readonly, glam_impls::dvec2_ui_many);
    add::<bevy_math::DVec3>(type_registry, glam_impls::dvec3_ui, glam_impls::dvec3_ui_readonly, glam_impls::dvec3_ui_many);
    add::<bevy_math::DVec4>(type_registry, glam_impls::dvec4_ui, glam_impls::dvec4_ui_readonly, glam_impls::dvec4_ui_many);
    add_no_many::<bevy_math::BVec2>(type_registry, glam_impls::bvec2_ui, glam_impls::bvec2_ui_readonly);
    add_no_many::<bevy_math::BVec3>(type_registry, glam_impls::bvec3_ui, glam_impls::bvec3_ui_readonly);
    add_no_many::<bevy_math::BVec4>(type_registry, glam_impls::bvec4_ui, glam_impls::bvec4_ui_readonly);
    add_no_many::<bevy_math::Mat2>(type_registry, glam_impls::mat2_ui, glam_impls::mat2_ui_readonly);
    add_no_many::<bevy_math::Mat3>(type_registry, glam_impls::mat3_ui, glam_impls::mat3_ui_readonly);
    add_no_many::<bevy_math::Mat3A>(type_registry, glam_impls::mat3a_ui, glam_impls::mat3a_ui_readonly);
    add_no_many::<bevy_math::Mat4>(type_registry, glam_impls::mat4_ui, glam_impls::mat4_ui_readonly);
    add_no_many::<bevy_math::DMat2>(type_registry, glam_impls::dmat2_ui, glam_impls::dmat2_ui_readonly);
    add_no_many::<bevy_math::DMat3>(type_registry, glam_impls::dmat3_ui, glam_impls::dmat3_ui_readonly);
    add_no_many::<bevy_math::DMat4>(type_registry, glam_impls::dmat4_ui, glam_impls::dmat4_ui_readonly);

    add::<bevy_math::Quat>(type_registry, glam_impls::quat::quat_ui, glam_impls::quat::quat_ui_readonly, glam_impls::quat::quat_ui_many);
}

/// Register [`InspectorEguiImpl`]s for `bevy` types
#[rustfmt::skip]
pub fn register_bevy_impls(type_registry: &mut TypeRegistry) {
    add_no_many::<bevy_asset::HandleId>(type_registry, bevy_impls::handle_id_ui, bevy_impls::handle_id_ui_readonly);
    add_no_many::<bevy_asset::Handle<bevy_render::texture::Image>>(type_registry, image::image_handle_ui, image::image_handle_ui_readonly);
    add_no_many::<bevy_asset::Handle<bevy_render::mesh::Mesh>>(type_registry, bevy_impls::mesh_ui, bevy_impls::mesh_ui_readonly);
    add::<bevy_render::color::Color>(type_registry, bevy_impls::color_ui, bevy_impls::color_ui_readonly, bevy_impls::color_ui_many);
}

pub(crate) fn change_slider<T>(
    ui: &mut egui::Ui,
    id: egui::Id,
    same: Option<T>,
    f: impl FnOnce(T, bool),
) -> bool
where
    T: egui::emath::Numeric + std::ops::Sub<Output = T> + Default + Send + Sync + 'static,
{
    let speed = if T::INTEGRAL { 1.0 } else { 0.1 };

    match same {
        Some(mut same) => {
            let widget = egui::DragValue::new(&mut same).speed(speed);

            let changed = ui.add(widget).changed();
            if changed {
                f(same, true);
            }

            changed
        }
        None => {
            let old_change = *ui.memory().data.get_temp_mut_or_default::<T>(id);
            let mut change = old_change;

            let widget = egui::DragValue::new(&mut change)
                .speed(speed)
                .custom_formatter(|_, _| "-".to_string());

            let changed = ui.add(widget).changed();
            if changed {
                f(change - old_change, false);
            }

            *ui.memory().data.get_temp_mut_or_default(id) = change;
            changed
        }
    }
}

pub(crate) fn iter_all_eq<T: Copy + PartialEq>(mut iter: impl Iterator<Item = T>) -> Option<T> {
    let Some(first) = iter.next() else { return None };

    iter.all(|elem| elem == first).then_some(first)
}

#[macro_export]
#[doc(hidden)]
macro_rules! many_ui {
    ($name:ident $inner:ident $ty:ty) => {
        pub fn $name(
            ui: &mut egui::Ui,
            options: &dyn Any,
            id: egui::Id,
            env: InspectorUi<'_, '_>,
            values: &mut [&mut dyn bevy_reflect::Reflect],
            projector: &dyn Fn(&mut dyn bevy_reflect::Reflect) -> &mut dyn bevy_reflect::Reflect,
        ) -> bool {
            let same = $crate::inspector_egui_impls::iter_all_eq(
                values
                    .iter_mut()
                    .map(|value| projector(*value).downcast_ref::<$ty>().unwrap()),
            );

            let mut temp = same.cloned().unwrap_or_default();
            if $inner(&mut temp, ui, options, id, env) {
                for value in values.iter_mut() {
                    let value = projector(*value).downcast_mut::<$ty>().unwrap();
                    *value = temp.clone();
                }

                return true;
            }
            false
        }
    };
}
