//! Custom UI implementations for specific types. Check [`InspectorPrimitive`] for an example.

use crate::{
    reflect_inspector::{InspectorUi, ProjectorReflect, errors::no_multiedit},
    utils::pretty_type_name,
};
use bevy_platform::time::Instant;
use bevy_reflect::{FromType, PartialReflect, Reflect, TypePath, TypeRegistry};
use std::{
    any::{Any, TypeId},
    borrow::Cow,
    path::PathBuf,
};

mod bevy_impls;
mod glam_impls;
#[cfg(feature = "bevy_image")]
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
    &mut [&mut dyn PartialReflect],
    &dyn ProjectorReflect,
) -> bool;

/// Custom UI implementation for a concrete type.
///
/// # Example Usage
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_inspector_egui::inspector_egui_impls::{InspectorEguiImpl, InspectorPrimitive};
/// use bevy_inspector_egui::quick::ResourceInspectorPlugin;
/// use bevy_inspector_egui::reflect_inspector::InspectorUi;
///
/// #[derive(Reflect, Default)]
/// struct ToggleOption(bool);
///
/// impl InspectorPrimitive for ToggleOption {
///     fn ui(&mut self, ui: &mut egui::Ui, _: &dyn std::any::Any, _: egui::Id, _: InspectorUi<'_, '_>) -> bool {
///         let mut changed = ui.radio_value(&mut self.0, false, "Disabled").changed();
///         changed |= ui.radio_value(&mut self.0, true, "Enabled").changed();
///         changed
///     }
///
///     fn ui_readonly(&self, ui: &mut egui::Ui, _: &dyn std::any::Any, _: egui::Id, _: InspectorUi<'_, '_>) {
///         let mut copy = self.0;
///         ui.add_enabled_ui(false, |ui| {
///             ui.radio_value(&mut copy, false, "Disabled").changed();
///             ui.radio_value(&mut copy, true, "Enabled").changed();
///         });
///     }
/// }
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         // ...
///         .register_type_data::<ToggleOption, InspectorEguiImpl>()
///         .run();
/// }
/// ```
pub trait InspectorPrimitive: Reflect {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) -> bool;
    fn ui_readonly(
        &self,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    );
}

fn ui_many_vtable<T: Reflect + PartialEq + Clone + Default + InspectorPrimitive>(
    ui: &mut egui::Ui,
    options: &dyn Any,
    id: egui::Id,
    env: InspectorUi<'_, '_>,
    values: &mut [&mut dyn bevy_reflect::PartialReflect],
    projector: &dyn ProjectorReflect,
) -> bool {
    let same = crate::inspector_egui_impls::iter_all_eq(values.iter_mut().map(|value| {
        projector(*value)
            .try_downcast_mut::<T>()
            .expect("non-fully-reflected value passed to ui_many_vtable")
    }));

    let mut temp = same.cloned().unwrap_or_default();
    if T::ui(&mut temp, ui, options, id, env) {
        for value in values.iter_mut() {
            let value = projector(*value)
                .try_downcast_mut::<T>()
                .expect("non-fully-reflected value passed to ui_many_vtable");
            *value = temp.clone();
        }

        return true;
    }
    false
}

fn ui_vtable<T: InspectorPrimitive>(
    val: &mut dyn Any,
    ui: &mut egui::Ui,
    options: &dyn Any,
    id: egui::Id,
    env: InspectorUi<'_, '_>,
) -> bool {
    let val = val.downcast_mut::<T>().unwrap();
    T::ui(val, ui, options, id, env)
}
fn ui_readonly_vtable<T: InspectorPrimitive>(
    val: &dyn Any,
    ui: &mut egui::Ui,
    options: &dyn Any,
    id: egui::Id,
    env: InspectorUi<'_, '_>,
) {
    let val = val.downcast_ref::<T>().unwrap();
    T::ui_readonly(val, ui, options, id, env)
}

/// Function pointers for displaying a concrete type, to be registered in the [`TypeRegistry`].
///
/// This can used for leaf types like `u8` or `String`, as well as people who want to completely customize the way
/// to display a certain type. You can use [`InspectorPrimitive`] to avoid manually writing the function pointers with correct downcasting.
#[derive(Clone)]
pub struct InspectorEguiImpl {
    fn_mut: InspectorEguiImplFn,
    fn_readonly: InspectorEguiImplFnReadonly,
    fn_many: InspectorEguiImplFnMany,
}

impl<T: InspectorPrimitive> FromType<T> for InspectorEguiImpl {
    fn from_type() -> Self {
        InspectorEguiImpl::of_with_many::<T>(many_unimplemented::<T>)
    }
}

impl InspectorEguiImpl {
    pub fn of<T: InspectorPrimitive + PartialEq + Clone + Default>() -> Self {
        InspectorEguiImpl {
            fn_mut: ui_vtable::<T>,
            fn_readonly: ui_readonly_vtable::<T>,
            fn_many: ui_many_vtable::<T>,
        }
    }
    pub fn of_with_many<T: InspectorPrimitive>(fn_many: InspectorEguiImplFnMany) -> Self {
        InspectorEguiImpl {
            fn_mut: ui_vtable::<T>,
            fn_readonly: ui_readonly_vtable::<T>,
            fn_many,
        }
    }

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
        values: &mut [&mut dyn PartialReflect],
        projector: &dyn ProjectorReflect,
    ) -> bool {
        (self.fn_many)(ui, options, id, env, values, projector)
    }
}

fn many_unimplemented<T: Any>(
    ui: &mut egui::Ui,
    _options: &dyn Any,
    _id: egui::Id,
    _env: InspectorUi<'_, '_>,
    _values: &mut [&mut dyn PartialReflect],
    _projector: &dyn ProjectorReflect,
) -> bool {
    no_multiedit(ui, &pretty_type_name::<T>());
    false
}

fn add<T: InspectorPrimitive + TypePath>(type_registry: &mut TypeRegistry) {
    type_registry.register_type_data::<T, InspectorEguiImpl>();
}
fn add_of_with_many<T: InspectorPrimitive>(
    type_registry: &mut TypeRegistry,
    fn_many: InspectorEguiImplFnMany,
) {
    type_registry
        .get_mut(TypeId::of::<T>())
        .unwrap_or_else(|| panic!("{} not registered", std::any::type_name::<T>()))
        .insert(InspectorEguiImpl::of_with_many::<T>(fn_many));
}

fn add_raw<T: 'static>(
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
    add_of_with_many::<f32>(type_registry, std_impls::number_ui_many::<f32>);
    add_of_with_many::<f64>(type_registry, std_impls::number_ui_many::<f64>);
    add_of_with_many::<i8>(type_registry, std_impls::number_ui_many::<i8>);
    add_of_with_many::<i16>(type_registry, std_impls::number_ui_many::<i16>);
    add_of_with_many::<i32>(type_registry, std_impls::number_ui_many::<i32>);
    add_of_with_many::<i64>(type_registry, std_impls::number_ui_many::<i64>);
    add_of_with_many::<isize>(type_registry, std_impls::number_ui_many::<isize>);
    add_of_with_many::<u8>(type_registry, std_impls::number_ui_many::<u8>);
    add_of_with_many::<u16>(type_registry, std_impls::number_ui_many::<u16>);
    add_of_with_many::<u32>(type_registry, std_impls::number_ui_many::<u32>);
    add_of_with_many::<u64>(type_registry, std_impls::number_ui_many::<u64>);
    add_of_with_many::<usize>(type_registry, std_impls::number_ui_many::<usize>);
    add::<bool>(type_registry);
    add::<String>(type_registry);
    type_registry.register::<Cow<str>>();
    add::<Cow<str>>(type_registry);
    type_registry.register::<PathBuf>();
    add::<PathBuf>(type_registry);

    type_registry.register::<std::ops::Range<f64>>();
    type_registry.register::<std::ops::RangeInclusive<f32>>();
    type_registry.register::<std::ops::RangeInclusive<f64>>();
    add::<std::ops::Range<f32>>(type_registry);
    add::<std::ops::Range<f64>>(type_registry);
    add::<std::ops::RangeInclusive<f32>>(type_registry);
    add::<std::ops::RangeInclusive<f64>>(type_registry);
    add::<TypeId>(type_registry);

    add::<std::time::Duration>(type_registry);
    add_of_with_many::<Instant>(type_registry, many_unimplemented::<Instant>);
}

/// Register [`InspectorEguiImpl`]s for [`bevy_math`]/`glam` types
#[rustfmt::skip]
pub fn register_glam_impls(type_registry: &mut TypeRegistry) {
    add_raw::<bevy_math::Vec2>(type_registry, glam_impls::vec2_ui, glam_impls::vec2_ui_readonly, glam_impls::vec2_ui_many);
    add_raw::<bevy_math::Vec3>(type_registry, glam_impls::vec3_ui, glam_impls::vec3_ui_readonly, glam_impls::vec3_ui_many);
    add_raw::<bevy_math::Vec3A>(type_registry, glam_impls::vec3a_ui, glam_impls::vec3a_ui_readonly, glam_impls::vec3a_ui_many);
    add_raw::<bevy_math::Vec4>(type_registry, glam_impls::vec4_ui, glam_impls::vec4_ui_readonly, glam_impls::vec4_ui_many);
    add_raw::<bevy_math::UVec2>(type_registry, glam_impls::uvec2_ui, glam_impls::uvec2_ui_readonly, glam_impls::uvec2_ui_many);
    add_raw::<bevy_math::UVec3>(type_registry, glam_impls::uvec3_ui, glam_impls::uvec3_ui_readonly, glam_impls::uvec3_ui_many);
    add_raw::<bevy_math::UVec4>(type_registry, glam_impls::uvec4_ui, glam_impls::uvec4_ui_readonly, glam_impls::uvec4_ui_many);
    add_raw::<bevy_math::IVec2>(type_registry, glam_impls::ivec2_ui, glam_impls::ivec2_ui_readonly, glam_impls::ivec2_ui_many);
    add_raw::<bevy_math::IVec3>(type_registry, glam_impls::ivec3_ui, glam_impls::ivec3_ui_readonly, glam_impls::ivec3_ui_many);
    add_raw::<bevy_math::IVec4>(type_registry, glam_impls::ivec4_ui, glam_impls::ivec4_ui_readonly, glam_impls::ivec4_ui_many);
    add_raw::<bevy_math::DVec2>(type_registry, glam_impls::dvec2_ui, glam_impls::dvec2_ui_readonly, glam_impls::dvec2_ui_many);
    add_raw::<bevy_math::DVec3>(type_registry, glam_impls::dvec3_ui, glam_impls::dvec3_ui_readonly, glam_impls::dvec3_ui_many);
    add_raw::<bevy_math::DVec4>(type_registry, glam_impls::dvec4_ui, glam_impls::dvec4_ui_readonly, glam_impls::dvec4_ui_many);
    add_raw::<bevy_math::BVec2>(type_registry, glam_impls::bvec2_ui, glam_impls::bvec2_ui_readonly, many_unimplemented::<bevy_math::BVec2>);
    add_raw::<bevy_math::BVec3>(type_registry, glam_impls::bvec3_ui, glam_impls::bvec3_ui_readonly, many_unimplemented::<bevy_math::BVec3>);
    add_raw::<bevy_math::BVec4>(type_registry, glam_impls::bvec4_ui, glam_impls::bvec4_ui_readonly, many_unimplemented::<bevy_math::BVec4>);
    add_raw::<bevy_math::Mat2>(type_registry, glam_impls::mat2_ui, glam_impls::mat2_ui_readonly, many_unimplemented::<bevy_math::Mat2>);
    add_raw::<bevy_math::Mat3>(type_registry, glam_impls::mat3_ui, glam_impls::mat3_ui_readonly, many_unimplemented::<bevy_math::Mat3>);
    add_raw::<bevy_math::Mat3A>(type_registry, glam_impls::mat3a_ui, glam_impls::mat3a_ui_readonly, many_unimplemented::<bevy_math::Mat3A>);
    add_raw::<bevy_math::Mat4>(type_registry, glam_impls::mat4_ui, glam_impls::mat4_ui_readonly, many_unimplemented::<bevy_math::Mat4>);
    add_raw::<bevy_math::DMat2>(type_registry, glam_impls::dmat2_ui, glam_impls::dmat2_ui_readonly, many_unimplemented::<bevy_math::DMat2>);
    add_raw::<bevy_math::DMat3>(type_registry, glam_impls::dmat3_ui, glam_impls::dmat3_ui_readonly, many_unimplemented::<bevy_math::DMat3>);
    add_raw::<bevy_math::DMat4>(type_registry, glam_impls::dmat4_ui, glam_impls::dmat4_ui_readonly, many_unimplemented::<bevy_math::DMat4>);

    add_raw::<bevy_math::Quat>(type_registry, glam_impls::quat::quat_ui, glam_impls::quat::quat_ui_readonly, glam_impls::quat::quat_ui_many);
}

/// Register [`InspectorEguiImpl`]s for `bevy` types
#[rustfmt::skip]
pub fn register_bevy_impls(type_registry: &mut TypeRegistry) {
    type_registry.register::<bevy_ecs::entity::Entity>();
    add_of_with_many::<bevy_ecs::entity::Entity>(type_registry, many_unimplemented::<bevy_ecs::entity::Entity>);
    add::<bevy_color::Color>(type_registry);

    #[cfg(feature = "bevy_render")]
    {
      type_registry.register::<bevy_camera::visibility::RenderLayers>();
      add_of_with_many::<bevy_asset::Handle<bevy_mesh::Mesh>>(type_registry, many_unimplemented::<bevy_asset::Handle<bevy_mesh::Mesh>>);
      add::<bevy_camera::visibility::RenderLayers>(type_registry);
    }
    #[cfg(feature = "bevy_image")]
    {
      add_of_with_many::<bevy_asset::Handle<bevy_image::Image>>(type_registry, many_unimplemented::<bevy_asset::Handle<bevy_image::Image>>);
    }
    #[cfg(feature = "bevy_gizmos")]
    add::<bevy_gizmos::config::GizmoConfigStore>(type_registry);

    add::<uuid::Uuid>(type_registry);
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
            let old_change = ui.memory_mut(|memory| *memory.data.get_temp_mut_or_default::<T>(id));
            let mut change = old_change;

            let widget = egui::DragValue::new(&mut change)
                .speed(speed)
                .custom_formatter(|_, _| "-".to_string());

            let changed = ui.add(widget).changed();
            if changed {
                f(change - old_change, false);
            }

            ui.memory_mut(|memory| *memory.data.get_temp_mut_or_default(id) = change);
            changed
        }
    }
}

pub(crate) fn iter_all_eq<T: PartialEq>(mut iter: impl Iterator<Item = T>) -> Option<T> {
    let first = iter.next()?;
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
            values: &mut [&mut dyn bevy_reflect::PartialReflect],
            projector: &dyn $crate::reflect_inspector::ProjectorReflect,
        ) -> bool {
            let same = $crate::inspector_egui_impls::iter_all_eq(
                values
                    .iter_mut()
                    .map(|value| projector(*value).try_downcast_ref::<$ty>().unwrap()),
            );

            let mut temp = same.cloned().unwrap_or_default();
            if $inner(&mut temp, ui, options, id, env) {
                for value in values.iter_mut() {
                    let value = projector(*value).try_downcast_mut::<$ty>().unwrap();
                    *value = temp.clone();
                }

                return true;
            }
            false
        }
    };
}
