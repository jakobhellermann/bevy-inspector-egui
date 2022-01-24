use bevy::{math::EulerRot, prelude::*};
use bevy_egui::egui;

use crate::{Context, Inspectable};

#[derive(Clone)]
pub struct QuatAttributes {
    pub display: QuatDisplay,
}

#[derive(Copy, Clone)]
pub enum QuatDisplay {
    Raw,
    Euler,
    YawPitchRoll,
    AxisAngle,
}

impl Default for QuatAttributes {
    fn default() -> Self {
        QuatAttributes {
            display: QuatDisplay::Euler,
        }
    }
}

#[derive(Clone, Copy)]
struct Euler(Vec3);
#[derive(Clone, Copy)]
struct YawPitchRoll((f32, f32, f32));
#[derive(Clone, Copy)]
struct AxisAngle((Vec3, f32));

trait RotationEdit {
    fn from_quat(quat: Quat) -> Self;
    fn to_quat(self) -> Quat;

    fn ui(&mut self, ui: &mut egui::Ui, context: &mut Context) -> bool;
}

impl RotationEdit for Euler {
    fn from_quat(quat: Quat) -> Self {
        Euler(quat.to_euler(EulerRot::XYZ).into())
    }

    fn to_quat(self) -> Quat {
        Quat::from_euler(EulerRot::XYZ, self.0.x, self.0.y, self.0.z)
    }

    fn ui(&mut self, ui: &mut egui::Ui, context: &mut Context) -> bool {
        self.0.ui(ui, Default::default(), context)
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

    fn ui(&mut self, ui: &mut egui::Ui, _: &mut Context) -> bool {
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

    fn ui(&mut self, ui: &mut egui::Ui, context: &mut Context) -> bool {
        let (axis, angle) = &mut self.0;

        let mut changed = false;
        ui.vertical(|ui| {
            egui::Grid::new("axis-angle quat").show(ui, |ui| {
                ui.label("Axis");
                changed |= axis.ui(ui, Default::default(), context);
                ui.end_row();
                ui.label("Angle");
                changed |= ui.drag_angle(angle).changed();
                ui.end_row();
            });
        });
        changed
    }
}

fn quat_ui<T: Send + Sync + 'static + Copy + RotationEdit>(
    val: &mut Quat,
    ui: &mut egui::Ui,
    context: &mut Context,
) -> bool {
    let mut intermediate = *ui
        .memory()
        .data
        .get_temp_mut_or_insert_with(context.id(), || T::from_quat(*val));

    let externally_changed = !intermediate.to_quat().abs_diff_eq(*val, std::f32::EPSILON);
    if externally_changed {
        intermediate = T::from_quat(*val);
    }

    let changed = intermediate.ui(ui, context);

    if changed || externally_changed {
        *val = intermediate.to_quat();
        ui.memory().data.insert_temp(context.id(), intermediate);
    }

    changed
}

impl Inspectable for Quat {
    type Attributes = QuatAttributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &mut Context) -> bool {
        match options.display {
            QuatDisplay::Raw => {
                let mut vec4 = Vec4::from(*self);
                let changed = ui
                    .vertical(|ui| vec4.ui(ui, Default::default(), context))
                    .inner;
                if changed {
                    *self = Quat::from_vec4(vec4).normalize();
                }
                changed
            }
            QuatDisplay::Euler => quat_ui::<Euler>(self, ui, context),
            QuatDisplay::YawPitchRoll => quat_ui::<YawPitchRoll>(self, ui, context),
            QuatDisplay::AxisAngle => quat_ui::<AxisAngle>(self, ui, context),
        }
    }
}
