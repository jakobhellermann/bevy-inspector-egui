use bevy::prelude::*;
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
                    *self = Quat::from(vec4).normalize();
                }
                changed
            }
            QuatDisplay::Euler => {
                let mut euler_angles = ui
                    .memory()
                    .id_data_temp
                    .get_mut_or_insert_with(context.id(), || Euler(to_euler_angles(*self)))
                    .0;

                let changed = euler_angles.ui(ui, Default::default(), context);
                if changed {
                    *self = from_euler_angles(euler_angles);
                    *ui.memory().id_data_temp.get_mut(&context.id()).unwrap() = Euler(euler_angles);
                }
                changed
            }
            QuatDisplay::YawPitchRoll => {
                let (mut yaw, mut pitch, mut roll) = ui
                    .memory()
                    .id_data_temp
                    .get_mut_or_insert_with(context.id(), || YawPitchRoll(yaw_pitch_roll(*self)))
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
                    *self = Quat::from_rotation_ypr(yaw, pitch, roll);
                    *ui.memory().id_data_temp.get_mut(&context.id()).unwrap() =
                        YawPitchRoll((yaw, pitch, roll));
                }

                changed
            }
            QuatDisplay::AxisAngle => {
                let (mut axis, mut angle) = ui
                    .memory()
                    .id_data_temp
                    .get_mut_or_insert_with(context.id(), || AxisAngle(self.to_axis_angle()))
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
                    *ui.memory().id_data_temp.get_mut(&context.id()).unwrap() =
                        AxisAngle((axis, angle));
                }
                changed
            }
        }
    }
}

// yaw - Z, pitch - Y, roll - X
fn to_euler_angles(val: Quat) -> Vec3 {
    let (yaw, pitch, roll) = yaw_pitch_roll(val);
    Vec3::new(roll, pitch, yaw)
}
fn from_euler_angles(val: Vec3) -> Quat {
    let yaw = val.z;
    let pitch = val.y;
    let roll = val.x;
    Quat::from_rotation_ypr(yaw, pitch, roll)
}

#[allow(clippy::many_single_char_names)]
fn yaw_pitch_roll(q: Quat) -> (f32, f32, f32) {
    let [x, y, z, w] = *q.as_ref();

    fn atan2(a: f32, b: f32) -> f32 {
        a.atan2(b)
    }
    fn asin(a: f32) -> f32 {
        a.asin()
    }

    let yaw = atan2(2.0 * (y * z + w * x), w * w - x * x - y * y + z * z);
    let pitch = asin(-2.0 * (x * z - w * y));
    let roll = atan2(2.0 * (x * y + w * z), w * w + x * x - y * y - z * z);

    (yaw, pitch, roll)
}
