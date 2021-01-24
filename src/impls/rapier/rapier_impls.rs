use super::nalgebra_conversions::*;
use crate::impls::NumberAttributes;
use crate::{Context, Inspectable};
use bevy_rapier3d::{
    na::Isometry3,
    physics::RigidBodyHandleComponent,
    rapier::dynamics::{MassProperties, RigidBody, RigidBodySet},
};

impl Inspectable for MassProperties {
    type Attributes = ();

    fn ui(&mut self, ui: &mut bevy_egui::egui::Ui, _options: Self::Attributes, context: &Context) {
        ui.label("Mass");
        let mut mass = 1. / self.inv_mass;
        (&mut mass).ui(
            ui,
            NumberAttributes {
                min: Some(0.001),
                ..Default::default()
            },
            context
        );
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
        let mut mass_properties = self.mass_properties().clone();
        mass_properties.ui(ui, Default::default(), context);
        self.set_mass_properties(mass_properties, false);

        let position = self.position();

        ui.label("Translation");
        let mut translation = position.translation.vector.to_glam_vec3();
        translation.ui(ui, Default::default(), context);

        ui.label("Rotation");
        let mut rotation = position.rotation.to_glam_quat();
        rotation.ui(ui, Default::default(), context);

        self.set_position(
            Isometry3::from_parts(translation.to_na_translation(), rotation.to_na_unit_quat()),
            false,
        );

        ui.label("Linear velocity");
        let mut linvel = self.linvel().to_glam_vec3();
        linvel.ui(ui, Default::default(), context);
        self.set_linvel(linvel.to_na_vector3(), false);

        ui.label("Angular velocity");
        let mut angvel = self.angvel().to_glam_vec3();
        angvel.ui(ui, Default::default(), context);
        self.set_angvel(angvel.to_na_vector3(), false);

        self.wake_up(false);
    }
}

impl Inspectable for RigidBodyHandleComponent {
    type Attributes = <RigidBody as Inspectable>::Attributes;

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        options: Self::Attributes,
        context: &Context,
    ) {
        let resources = if let Some(resources) = context.resources.as_ref() {
          resources
        } else {
          ui.label("RigidBodyHandleComponent requires &Resources in Inspectable Context");
          return;
        };

        let mut bodies = match resources.get_mut::<RigidBodySet>() {
            Some(bodies) => bodies,
            None => {
                ui.label("RigidBodySet is a required resource but is missing");
                return;
            }
        };

        let body = match bodies.get_mut(self.handle()) {
            Some(body) => body,
            None => {
                ui.label("This handle does not exist on RigidBodySet");
                return;
            }
        };

        body.ui(ui, options, context);
    }
}
