use bevy::{prelude::*, utils::HashMap};
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

enum QuatEditState {
    Euler(Vec3),
    YawPitchRoll((f32, f32, f32)),
    AxisAngle((Vec3, f32)),
}
#[derive(Default)]
struct QuatEditStates(HashMap<egui::Id, QuatEditState>);

impl Inspectable for Quat {
    type Attributes = QuatAttributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Self::Attributes, context: &Context) {
        match options.display {
            QuatDisplay::Raw => {
                let mut vec4 = Vec4::from(*self);
                ui.vertical(|ui| vec4.ui(ui, Default::default(), context));
                *self = Quat::from(vec4).normalize();
            }
            QuatDisplay::Euler => {
                let world = expect_world!(ui, context, "Quat");
                let mut states = world.get_resource_or_insert_with(QuatEditStates::default);
                let state = states
                    .0
                    .entry(context.id())
                    .or_insert(QuatEditState::Euler(to_euler_angles(*self)));

                let euler_angles = match state {
                    QuatEditState::Euler(euler) => euler,
                    _ => unreachable!("invalid quat edit state"),
                };

                euler_angles.ui(ui, Default::default(), context);

                *self = from_euler_angles(*euler_angles);
            }
            QuatDisplay::YawPitchRoll => {
                let world = expect_world!(ui, context, "Quat");
                let mut states = world.get_resource_or_insert_with(QuatEditStates::default);
                let state = states
                    .0
                    .entry(context.id())
                    .or_insert(QuatEditState::YawPitchRoll(yaw_pitch_roll(*self)));

                let (yaw, pitch, roll) = match state {
                    QuatEditState::YawPitchRoll((y, p, r)) => (y, p, r),
                    _ => unreachable!("invalid quat edit state"),
                };

                ui.vertical(|ui| {
                    egui::Grid::new("ypr grid").show(ui, |ui| {
                        ui.label("Yaw");
                        ui.drag_angle(yaw);
                        ui.end_row();
                        ui.label("Pitch");
                        ui.drag_angle(pitch);
                        ui.end_row();
                        ui.label("Roll");
                        ui.drag_angle(roll);
                        ui.end_row();
                    });
                });

                *self = Quat::from_rotation_ypr(*yaw, *pitch, *roll);
            }
            QuatDisplay::AxisAngle => {
                let world = expect_world!(ui, context, "Quat");
                let mut states = world.get_resource_or_insert_with(QuatEditStates::default);
                let state = states
                    .0
                    .entry(context.id())
                    .or_insert(QuatEditState::AxisAngle(self.to_axis_angle()));

                let (axis, angle) = match state {
                    QuatEditState::AxisAngle((axis, angle)) => (axis, angle),
                    _ => unreachable!("invalid quat edit state"),
                };

                ui.vertical(|ui| {
                    egui::Grid::new("axis-angle quat").show(ui, |ui| {
                        ui.label("Axis");
                        axis.ui(ui, Default::default(), context);
                        ui.end_row();
                        ui.label("Angle");
                        ui.drag_angle(angle);
                        ui.end_row();
                    });
                });
                *self = Quat::from_axis_angle(axis.normalize(), *angle);
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
