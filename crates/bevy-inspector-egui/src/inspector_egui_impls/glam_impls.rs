use std::any::Any;

use bevy_math::{prelude::*, DMat2, DMat3, DMat4, DVec2, DVec3, DVec4, Mat3A, Vec3A};

use crate::egui_reflect_inspector::InspectorUi;

macro_rules! vec_ui {
    ($name:ident $name_readonly:ident $ty:ty: $count:literal $($component:ident)*) => {
        pub fn $name(
            value: &mut dyn Any,
            ui: &mut egui::Ui,
            _: &dyn Any,
            mut env: InspectorUi<'_, '_>,
        ) -> bool {
            let value = value.downcast_mut::<$ty>().unwrap();

            let mut changed = false;
            ui.scope(|ui| {
                ui.style_mut().spacing.item_spacing = egui::Vec2::new(4.0, 0.);

                ui.columns($count, |ui| match ui {
                    [$($component),*] => {
                        $(changed |= env.ui_for_reflect(&mut value.$component, $component, egui::Id::null());)*
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
            mut env: InspectorUi<'_, '_>,
        ) {
            let value = value.downcast_ref::<$ty>().unwrap();

            ui.scope(|ui| {
                ui.style_mut().spacing.item_spacing = egui::Vec2::new(4.0, 0.);

                ui.columns($count, |ui| match ui {
                    [$($component),*] => {
                        $(env.ui_for_reflect_readonly(&value.$component, $component, egui::Id::null());)*
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
            mut env: InspectorUi<'_, '_>,
        ) -> bool {

            let value = value.downcast_mut::<$ty>().unwrap();

            let mut changed = false;
            ui.vertical(|ui| {
                $(changed |= env.ui_for_reflect(&mut value.$component, ui, egui::Id::null());)*
            });
            changed
        }

        pub fn $name_readonly(
            value: &dyn Any,
            ui: &mut egui::Ui,
            _: &dyn Any,
            mut env: InspectorUi<'_, '_>,
        ) {
            let value = value.downcast_ref::<$ty>().unwrap();

            ui.vertical(|ui| {
                $(env.ui_for_reflect_readonly(&value.$component, ui, egui::Id::null());)*
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

mat_ui!(mat2_ui mat2_ui_readonly Mat2: x_axis y_axis);
mat_ui!(mat3_ui mat3_ui_readonly Mat3: x_axis y_axis z_axis);
mat_ui!(mat3a_ui mat3a_ui_readonly Mat3A: x_axis y_axis z_axis);
mat_ui!(mat4_ui mat4_ui_readonly Mat4: x_axis y_axis z_axis w_axis);
mat_ui!(dmat2_ui dmat2_ui_readonly DMat2: x_axis y_axis);
mat_ui!(dmat3_ui dmat3_ui_readonly DMat3: x_axis y_axis z_axis);
mat_ui!(dmat4_ui dmat4_ui_readonly DMat4: x_axis y_axis z_axis w_axis);

pub mod quat {
    use std::any::Any;

    use bevy_egui::egui;
    use bevy_math::{prelude::*, EulerRot};

    use crate::{
        egui_reflect_inspector::InspectorUi,
        inspector_options::std_options::{QuatDisplay, QuatOptions},
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
            env.ui_for_reflect(&mut self.0, ui, egui::Id::null())
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
                    changed |= env.ui_for_reflect(axis, ui, egui::Id::null());
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
        let mut intermediate = *ui
            .memory()
            .data
            .get_temp_mut_or_insert_with(id, || T::from_quat(*val));

        let externally_changed = !intermediate.to_quat().abs_diff_eq(*val, std::f32::EPSILON);
        if externally_changed {
            intermediate = T::from_quat(*val);
        }

        let changed = intermediate.ui(ui, env);

        if changed || externally_changed {
            *val = intermediate.to_quat();
            ui.memory().data.insert_temp(id, intermediate);
        }

        changed
    }

    pub fn quat_ui(
        value: &mut dyn Any,
        ui: &mut egui::Ui,
        options: &dyn Any,
        mut env: InspectorUi<'_, '_>,
    ) -> bool {
        let value = value.downcast_mut::<Quat>().unwrap();

        let options = options
            .downcast_ref::<QuatOptions>()
            .cloned()
            .unwrap_or_default();

        ui.vertical(|ui| {
            let changed = match options.display {
                QuatDisplay::Raw => {
                    let mut vec4 = Vec4::from(*value);
                    let changed = env.ui_for_reflect(&mut vec4, ui, egui::Id::null());
                    if changed {
                        *value = Quat::from_vec4(vec4).normalize();
                    }
                    changed
                }
                QuatDisplay::Euler => quat_ui_kind::<Euler>(value, ui, env),
                QuatDisplay::YawPitchRoll => quat_ui_kind::<YawPitchRoll>(value, ui, env),
                QuatDisplay::AxisAngle => quat_ui_kind::<AxisAngle>(value, ui, env),
            };

            changed
        })
        .inner
    }

    pub fn quat_ui_readonly(
        value: &dyn Any,
        ui: &mut egui::Ui,
        options: &dyn Any,
        env: InspectorUi<'_, '_>,
    ) {
        let mut value = value.downcast_ref::<Quat>().unwrap().clone();
        ui.add_enabled_ui(false, |ui| quat_ui(&mut value, ui, options, env));
    }
}
