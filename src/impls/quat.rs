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

#[derive(Clone)]
struct Euler(Vec3);
#[derive(Clone)]
struct YawPitchRoll((f32, f32, f32));
#[derive(Clone)]
struct AxisAngle((Vec3, f32));

impl Inspectable for Quat {
    type Attributes = QuatAttributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context) -> bool {
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
            QuatDisplay::Euler => {
                let mut euler_angles = ui
                    .memory()
                    .data
                    .get_temp_mut_or_insert_with(context.id(), || {
                        Euler(Vec3::from(self.to_euler(EulerRot::XYZ)))
                    })
                    .0;

                let changed = euler_angles.ui(ui, Default::default(), context);
                if changed {
                    *self = Quat::from_euler(
                        EulerRot::XYZ,
                        euler_angles.x,
                        euler_angles.y,
                        euler_angles.z,
                    );
                    ui.memory()
                        .data
                        .insert_temp(context.id(), Euler(euler_angles));
                }
                changed
            }
            QuatDisplay::YawPitchRoll => {
                let (mut yaw, mut pitch, mut roll) = ui
                    .memory()
                    .data
                    .get_temp_mut_or_insert_with(context.id(), || {
                        YawPitchRoll(self.to_euler(EulerRot::YXZ))
                    })
                    .0;

                let mut changed = false;
                ui.vertical(|ui| {
                    egui::Grid::new("ypr grid").show(ui, |ui| {
                        ui.label("Yaw");
                        changed |= ui.drag_angle(&mut yaw).changed();
                        ui.end_row();
                        ui.label("Pitch").changed();
                        changed |= ui.drag_angle(&mut pitch).changed();
                        ui.end_row();
                        ui.label("Roll");
                        changed |= ui.drag_angle(&mut roll).changed();
                        ui.end_row();
                    });
                });

                if changed {
                    *self = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
                    ui.memory()
                        .data
                        .insert_temp(context.id(), YawPitchRoll((yaw, pitch, roll)));
                }

                changed
            }
            QuatDisplay::AxisAngle => {
                let (mut axis, mut angle) = ui
                    .memory()
                    .data
                    .get_temp_mut_or_insert_with(context.id(), || AxisAngle(self.to_axis_angle()))
                    .0;

                let mut changed = false;
                ui.vertical(|ui| {
                    egui::Grid::new("axis-angle quat").show(ui, |ui| {
                        ui.label("Axis");
                        changed |= axis.ui(ui, Default::default(), context);
                        ui.end_row();
                        ui.label("Angle");
                        changed |= ui.drag_angle(&mut angle).changed();
                        ui.end_row();
                    });
                });
                if changed {
                    *self = Quat::from_axis_angle(axis.normalize(), angle);
                    ui.memory()
                        .data
                        .insert_temp(context.id(), AxisAngle((axis, angle)));
                }
                changed
            }
        }
    }
}
