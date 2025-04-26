use std::any::Any;

use bevy_math::{DMat2, DMat3, DMat4, DVec2, DVec3, DVec4, Mat3A, Vec3A, prelude::*};
use bevy_reflect::PartialReflect;

use crate::inspector_options::std_options::NumberOptions;
use crate::reflect_inspector::InspectorUi;
use crate::reflect_inspector::ProjectorReflect;

macro_rules! vec_ui_many {
    ($name_many:ident $ty:ty>$elem_ty:ty: $count:literal $($component:ident)*) => {
        pub fn $name_many(
            ui: &mut egui::Ui,
            _: &dyn Any,
            id: egui::Id,
            _env: InspectorUi<'_, '_>,
            values: &mut [&mut dyn PartialReflect],
            projector: &dyn ProjectorReflect,
        ) -> bool {
            let mut changed = false;
            ui.scope(|ui| {
                ui.style_mut().spacing.item_spacing = egui::Vec2::new(4.0, 0.);

                ui.columns($count, |ui| match ui {
                    [$($component),*] => {
                        $(

                            let same = super::iter_all_eq(values.iter_mut().map(|value| {
                                // FIXME: scary to change a macro
                                projector(*value).try_downcast_ref::<$ty>().unwrap().$component
                            }));

                            let id = id.with(stringify!($component));
                            changed |= crate::inspector_egui_impls::change_slider($component, id, same, |change, overwrite| {
                                for value in values.iter_mut() {
                                    let value = projector(*value);
                                    let value = value.try_downcast_mut::<$ty>().unwrap();

                                    if false { value.$component = change };
                                    if overwrite {
                                        value.$component = change;
                                    } else {
                                        value.$component += change;
                                    }

                                }
                            });
                        )*
                    }
                    _ => unreachable!(),
                });
            });
            changed
        }
    };
}

macro_rules! vec_ui {
    ($name:ident $name_readonly:ident $ty:ty: $count:literal $($component:ident)*) => {
        pub fn $name(
            value: &mut dyn Any,
            ui: &mut egui::Ui,
            options: &dyn Any,
            id: egui::Id,
            mut env: InspectorUi<'_, '_>,
        ) -> bool {
            let value = value.downcast_mut::<$ty>().unwrap();

            let options = options
                .downcast_ref::<NumberOptions<$ty>>()
                .cloned()
                .unwrap_or_default();

            let mut changed = false;
            ui.scope(|ui| {
                ui.style_mut().spacing.item_spacing = egui::Vec2::new(4.0, 0.);

                ui.columns($count, |ui| match ui {
                    [$($component),*] => {
                        $(changed |= env.ui_for_reflect_with_options(&mut value.$component, $component, id.with(stringify!($component)), &options.map(|vec| vec.$component));)*
                    }
                    _ => unreachable!(),
                });
            });
            changed
        }

        pub fn $name_readonly(
            value: &dyn Any,
            ui: &mut egui::Ui,
            _: &dyn Any,
            _: egui::Id,
            mut env: InspectorUi<'_, '_>,
        ) {
            let value = value.downcast_ref::<$ty>().unwrap();

            ui.scope(|ui| {
                ui.style_mut().spacing.item_spacing = egui::Vec2::new(4.0, 0.);

                ui.columns($count, |ui| match ui {
                    [$($component),*] => {
                        $(env.ui_for_reflect_readonly(&value.$component, $component);)*
                    }
                    _ => unreachable!(),
                });
            });
        }
    };
}

macro_rules! mat_ui {
    ($name:ident $name_readonly:ident $ty:ty: $($component:ident)*) => {
        pub fn $name(
            value: &mut dyn Any,
            ui: &mut egui::Ui,
            _: &dyn Any,
            _: egui::Id,
            mut env: InspectorUi<'_, '_>,
        ) -> bool {
            let value = value.downcast_mut::<$ty>().unwrap();

            let mut changed = false;
            ui.vertical(|ui| {
                $(changed |= env.ui_for_reflect(&mut value.$component, ui);)*
            });
            changed
        }

        pub fn $name_readonly(
            value: &dyn Any,
            ui: &mut egui::Ui,
            _: &dyn Any,
            _: egui::Id,
            mut env: InspectorUi<'_, '_>,
        ) {
            let value = value.downcast_ref::<$ty>().unwrap();

            ui.vertical(|ui| {
                $(env.ui_for_reflect_readonly(&value.$component, ui);)*
            });
        }
    };
}

vec_ui!(vec2_ui vec2_ui_readonly Vec2: 2 x y);
vec_ui!(vec3_ui vec3_ui_readonly Vec3: 3 x y z);
vec_ui!(vec3a_ui vec3a_ui_readonly Vec3A: 3 x y z);
vec_ui!(vec4_ui vec4_ui_readonly Vec4: 4 x y z w);
vec_ui!(uvec2_ui uvec2_ui_readonly UVec2: 2 x y);
vec_ui!(uvec3_ui uvec3_ui_readonly UVec3: 3 x y z);
vec_ui!(uvec4_ui uvec4_ui_readonly UVec4: 4 x y z w);
vec_ui!(ivec2_ui ivec2_ui_readonly IVec2: 2 x y);
vec_ui!(ivec3_ui ivec3_ui_readonly IVec3: 3 x y z);
vec_ui!(ivec4_ui ivec4_ui_readonly IVec4: 4 x y z w);
vec_ui!(dvec2_ui dvec2_ui_readonly DVec2: 2 x y);
vec_ui!(dvec3_ui dvec3_ui_readonly DVec3: 3 x y z);
vec_ui!(dvec4_ui dvec4_ui_readonly DVec4: 4 x y z w);
vec_ui!(bvec2_ui bvec2_ui_readonly BVec2: 2 x y);
vec_ui!(bvec3_ui bvec3_ui_readonly BVec3: 3 x y z);
vec_ui!(bvec4_ui bvec4_ui_readonly BVec4: 4 x y z w);
vec_ui_many!(vec2_ui_many Vec2>f32: 2 x y);
vec_ui_many!(vec3_ui_many Vec3>f32: 3 x y z);
vec_ui_many!(vec3a_ui_many Vec3A>f32: 3 x y z);
vec_ui_many!(vec4_ui_many Vec4>f32: 4 x y z w);
vec_ui_many!(uvec2_ui_many UVec2>u32: 2 x y);
vec_ui_many!(uvec3_ui_many UVec3>u32: 3 x y z);
vec_ui_many!(uvec4_ui_many UVec4>u32: 4 x y z w);
vec_ui_many!(ivec2_ui_many IVec2>i32: 2 x y);
vec_ui_many!(ivec3_ui_many IVec3>i32: 3 x y z);
vec_ui_many!(ivec4_ui_many IVec4>i32: 4 x y z w);
vec_ui_many!(dvec2_ui_many DVec2>f64: 2 x y);
vec_ui_many!(dvec3_ui_many DVec3>f64: 3 x y z);
vec_ui_many!(dvec4_ui_many DVec4>f64: 4 x y z w);

mat_ui!(mat2_ui mat2_ui_readonly Mat2: x_axis y_axis);
mat_ui!(mat3_ui mat3_ui_readonly Mat3: x_axis y_axis z_axis);
mat_ui!(mat3a_ui mat3a_ui_readonly Mat3A: x_axis y_axis z_axis);
mat_ui!(mat4_ui mat4_ui_readonly Mat4: x_axis y_axis z_axis w_axis);
mat_ui!(dmat2_ui dmat2_ui_readonly DMat2: x_axis y_axis);
mat_ui!(dmat3_ui dmat3_ui_readonly DMat3: x_axis y_axis z_axis);
mat_ui!(dmat4_ui dmat4_ui_readonly DMat4: x_axis y_axis z_axis w_axis);

pub mod quat {
    use std::any::Any;

    use bevy_math::prelude::*;

    use crate::{
        inspector_options::std_options::{QuatDisplay, QuatOptions},
        many_ui,
        reflect_inspector::InspectorUi,
    };

    #[derive(Clone, Copy)]
    struct Euler(Vec3);
    #[derive(Clone, Copy)]
    struct YawPitchRoll((f32, f32, f32));
    #[derive(Clone, Copy)]
    struct AxisAngle((Vec3, f32));

    trait RotationEdit {
        fn from_quat(quat: Quat) -> Self;
        fn to_quat(self) -> Quat;

        fn ui(&mut self, ui: &mut egui::Ui, env: InspectorUi<'_, '_>) -> bool;
    }

    impl RotationEdit for Euler {
        fn from_quat(quat: Quat) -> Self {
            Euler(quat.to_euler(EulerRot::XYZ).into())
        }

        fn to_quat(self) -> Quat {
            Quat::from_euler(EulerRot::XYZ, self.0.x, self.0.y, self.0.z)
        }

        fn ui(&mut self, ui: &mut egui::Ui, mut env: InspectorUi<'_, '_>) -> bool {
            env.ui_for_reflect(&mut self.0, ui)
        }
    }

    impl RotationEdit for YawPitchRoll {
        fn from_quat(quat: Quat) -> Self {
            YawPitchRoll(quat.to_euler(EulerRot::YXZ))
        }

        fn to_quat(self) -> Quat {
            let (y, p, r) = self.0;
            Quat::from_euler(EulerRot::YXZ, y, p, r)
        }

        fn ui(&mut self, ui: &mut egui::Ui, _env: InspectorUi<'_, '_>) -> bool {
            let (yaw, pitch, roll) = &mut self.0;

            let mut changed = false;
            ui.vertical(|ui| {
                egui::Grid::new("ypr grid").show(ui, |ui| {
                    ui.label("Yaw");
                    changed |= ui.drag_angle(yaw).changed();
                    ui.end_row();
                    ui.label("Pitch").changed();
                    changed |= ui.drag_angle(pitch).changed();
                    ui.end_row();
                    ui.label("Roll");
                    changed |= ui.drag_angle(roll).changed();
                    ui.end_row();
                });
            });
            changed
        }
    }

    impl RotationEdit for AxisAngle {
        fn from_quat(quat: Quat) -> Self {
            AxisAngle(quat.to_axis_angle())
        }

        fn to_quat(self) -> Quat {
            let (axis, angle) = self.0;
            let axis = axis.normalize();
            if axis.is_nan() {
                Quat::IDENTITY
            } else {
                Quat::from_axis_angle(axis.normalize(), angle)
            }
        }

        fn ui(&mut self, ui: &mut egui::Ui, mut env: InspectorUi<'_, '_>) -> bool {
            let (axis, angle) = &mut self.0;

            let mut changed = false;
            ui.vertical(|ui| {
                egui::Grid::new("axis-angle quat").show(ui, |ui| {
                    ui.label("Axis");
                    changed |= env.ui_for_reflect(axis, ui);
                    ui.end_row();
                    ui.label("Angle");
                    changed |= ui.drag_angle(angle).changed();
                    ui.end_row();
                });
            });
            changed
        }
    }

    fn quat_ui_kind<T: Send + Sync + 'static + Copy + RotationEdit>(
        val: &mut Quat,
        ui: &mut egui::Ui,
        env: InspectorUi<'_, '_>,
    ) -> bool {
        let id = ui.id();
        let mut intermediate = ui.memory_mut(|memory| {
            *memory
                .data
                .get_temp_mut_or_insert_with(id, || T::from_quat(*val))
        });

        let externally_changed = !intermediate.to_quat().abs_diff_eq(*val, f32::EPSILON);
        if externally_changed {
            intermediate = T::from_quat(*val);
        }

        let changed = intermediate.ui(ui, env);

        if changed || externally_changed {
            *val = intermediate.to_quat();
            ui.memory_mut(|memory| memory.data.insert_temp(id, intermediate));
        }

        changed
    }

    pub fn quat_ui(
        value: &mut dyn Any,
        ui: &mut egui::Ui,
        options: &dyn Any,
        _: egui::Id,
        mut env: InspectorUi<'_, '_>,
    ) -> bool {
        let value = value.downcast_mut::<Quat>().unwrap();

        let options = options
            .downcast_ref::<QuatOptions>()
            .cloned()
            .unwrap_or_default();

        ui.vertical(|ui| match options.display {
            QuatDisplay::Raw => {
                let mut vec4 = Vec4::from(*value);
                let changed = env.ui_for_reflect(&mut vec4, ui);
                if changed {
                    *value = Quat::from_vec4(vec4).normalize();
                }
                changed
            }
            QuatDisplay::Euler => quat_ui_kind::<Euler>(value, ui, env),
            QuatDisplay::YawPitchRoll => quat_ui_kind::<YawPitchRoll>(value, ui, env),
            QuatDisplay::AxisAngle => quat_ui_kind::<AxisAngle>(value, ui, env),
        })
        .inner
    }

    pub fn quat_ui_readonly(
        value: &dyn Any,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) {
        let mut value = *value.downcast_ref::<Quat>().unwrap();
        ui.add_enabled_ui(false, |ui| quat_ui(&mut value, ui, options, id, env));
    }

    many_ui!(quat_ui_many quat_ui Quat);
}
