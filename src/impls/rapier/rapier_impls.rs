use super::nalgebra_conversions::*;
use crate::egui::Grid;
use crate::impls::NumberAttributes;
use crate::{utils, Context, Inspectable};
use bevy_rapier3d::{
    na::Isometry3,
    physics::RigidBodyHandleComponent,
    rapier::dynamics::{BodyStatus, MassProperties, RigidBody, RigidBodySet},
};

impl_for_simple_enum!(BodyStatus: Dynamic, Static, Kinematic);

impl Inspectable for MassProperties {
    type Attributes = ();

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        _options: Self::Attributes,
        context: &Context,
    ) -> bool {
        let mut changed = false;

        ui.label("Mass");
        let mut mass = 1. / self.inv_mass;
        changed |= mass.ui(ui, NumberAttributes::min(0.001), context);
        self.inv_mass = 1. / mass;
        ui.end_row();

        ui.label("Center of mass");
        let mut com = self.local_com.to_glam_vec3();
        changed |= com.ui(ui, Default::default(), context);
        self.local_com = com.to_na_point3();
        ui.end_row();

        changed
    }
}

impl Inspectable for RigidBody {
    type Attributes = ();

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        _options: Self::Attributes,
        context: &Context,
    ) -> bool {
        // PERF: some updates here can be avoided
        let mut changed = false;
        ui.vertical_centered(|ui| {
            Grid::new(context.id()).show(ui, |ui| {
                ui.label("Body Status");
                let mut body_status = self.body_status();
                changed |= body_status.ui(ui, Default::default(), context);
                self.set_body_status(body_status);
                ui.end_row();

                let mut mass_properties = *self.mass_properties();
                changed |= mass_properties.ui(ui, Default::default(), context);
                self.set_mass_properties(mass_properties, false);

                let position = self.position();

                ui.label("Translation");
                let mut translation = position.translation.vector.to_glam_vec3();
                changed |= translation.ui(ui, Default::default(), context);
                ui.end_row();

                ui.label("Rotation");
                let mut rotation = position.rotation.to_glam_quat();
                changed |= rotation.ui(ui, Default::default(), context);
                ui.end_row();

                if changed {
                    self.set_position(
                        Isometry3::from_parts(
                            translation.to_na_translation(),
                            rotation.to_na_unit_quat(),
                        ),
                        false,
                    );
                }

                ui.label("Linear velocity");
                let mut linvel = self.linvel().to_glam_vec3();
                trunc_epsilon_vec3(&mut linvel);
                changed |= linvel.ui(ui, Default::default(), context);
                self.set_linvel(linvel.to_na_vector3(), false);
                ui.end_row();

                ui.label("Angular velocity");
                let mut angvel = self.angvel().to_glam_vec3();
                trunc_epsilon_vec3(&mut angvel);
                changed |= angvel.ui(ui, Default::default(), context);
                self.set_angvel(angvel.to_na_vector3(), false);
                ui.end_row();

                self.wake_up(false);
            });
        });
        changed
    }
}

fn trunc_epsilon_f32(val: &mut f32) {
    if val.abs() < f32::EPSILON {
        *val = 0.0;
    }
}
fn trunc_epsilon_vec3(val: &mut bevy::math::Vec3) {
    trunc_epsilon_f32(&mut val.x);
    trunc_epsilon_f32(&mut val.y);
    trunc_epsilon_f32(&mut val.z);
}

impl Inspectable for RigidBodyHandleComponent {
    type Attributes = <RigidBody as Inspectable>::Attributes;

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        options: Self::Attributes,
        context: &Context,
    ) -> bool {
        let world = expect_world!(ui, context, "RigidBodyHandleComponent");
        let mut bodies = world.get_resource_mut::<RigidBodySet>().unwrap();

        let body = match bodies.get_mut(self.handle()) {
            Some(body) => body,
            None => {
                utils::error_label(ui, "This handle does not exist on RigidBodySet");
                return false;
            }
        };

        body.ui(ui, options, context)
    }
}
