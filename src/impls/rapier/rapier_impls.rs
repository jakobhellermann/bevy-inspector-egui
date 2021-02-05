use super::nalgebra_conversions::*;
use crate::egui::Grid;
use crate::impls::NumberAttributes;
use crate::{utils, Context, Inspectable};
use bevy_rapier3d::{
    na::Isometry3,
    physics::RigidBodyHandleComponent,
    rapier::dynamics::{BodyStatus, MassProperties, RigidBody, RigidBodySet},
};

impl_for_simple_enum!(BodyStatus with Dynamic, Static, Kinematic);

impl Inspectable for MassProperties {
    type Attributes = ();

    fn ui(&mut self, ui: &mut bevy_egui::egui::Ui, _options: Self::Attributes, context: &Context) {
        ui.label("Mass");
        let mut mass = 1. / self.inv_mass;
        mass.ui(ui, NumberAttributes::min(0.001), context);
        self.inv_mass = 1. / mass;

        ui.label("Center of mass");
        let mut com = self.local_com.to_glam_vec3();
        com.ui(ui, Default::default(), context);
        self.local_com = com.to_na_point3();
    }
}

impl Inspectable for RigidBody {
    type Attributes = ();

    fn ui(&mut self, ui: &mut bevy_egui::egui::Ui, _options: Self::Attributes, context: &Context) {
        ui.vertical_centered(|ui| {
            Grid::new(std::any::TypeId::of::<RigidBody>()).show(ui, |ui| {
                ui.label("Body Status");
                self.body_status.ui(ui, Default::default(), context);
                ui.end_row();

                let mut mass_properties = *self.mass_properties();
                mass_properties.ui(ui, Default::default(), context);
                self.set_mass_properties(mass_properties, false);
                ui.end_row();

                let position = self.position();

                ui.label("Translation");
                let mut translation = position.translation.vector.to_glam_vec3();
                translation.ui(ui, Default::default(), context);
                ui.end_row();

                ui.label("Rotation");
                let mut rotation = position.rotation.to_glam_quat();
                rotation.ui(ui, Default::default(), context);
                ui.end_row();

                self.set_position(
                    Isometry3::from_parts(
                        translation.to_na_translation(),
                        rotation.to_na_unit_quat(),
                    ),
                    false,
                );

                ui.label("Linear velocity");
                let mut linvel = self.linvel().to_glam_vec3();
                linvel.ui(ui, Default::default(), context);
                self.set_linvel(linvel.to_na_vector3(), false);
                ui.end_row();

                ui.label("Angular velocity");
                let mut angvel = self.angvel().to_glam_vec3();
                angvel.ui(ui, Default::default(), context);
                self.set_angvel(angvel.to_na_vector3(), false);
                ui.end_row();

                self.wake_up(false);
            });
        });
    }
}

impl Inspectable for RigidBodyHandleComponent {
    type Attributes = <RigidBody as Inspectable>::Attributes;

    fn ui(&mut self, ui: &mut bevy_egui::egui::Ui, options: Self::Attributes, context: &Context) {
        let resources = expect_context!(ui, context.resources, "RigidBodyHandleComponent");
        let mut bodies = expect_resource!(ui, resources, get_mut RigidBodySet);

        let body = match bodies.get_mut(self.handle()) {
            Some(body) => body,
            None => {
                utils::error_label(ui, "This handle does not exist on RigidBodySet");
                return;
            }
        };

        body.ui(ui, options, context);
    }
}
