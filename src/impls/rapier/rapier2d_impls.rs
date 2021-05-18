use crate::egui::Grid;
use crate::impls::NumberAttributes;
use crate::{utils, Context, Inspectable};
use bevy::prelude::*;
use bevy_rapier2d::{
    na::{Complex, Isometry2, Translation2, Unit},
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
        let mut com: Vec2 = self.local_com.into();
        changed |= com.ui(ui, Default::default(), context);
        self.local_com = com.into();
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
                let mut translation: Vec2 = position.translation.vector.into();
                changed |= translation.ui(ui, Default::default(), context);
                ui.end_row();

                ui.label("Rotation");
                let mut angle = position.rotation.angle();
                changed |= angle.ui(ui, Default::default(), context);
                let rotation = Unit::<Complex<_>>::new(angle);

                ui.end_row();

                if changed {
                    self.set_position(
                        Isometry2::from_parts(
                            Translation2::new(translation.x, translation.y),
                            rotation,
                        ),
                        false,
                    );
                }

                ui.label("Linear velocity");
                let mut linvel: Vec2 = (*self.linvel()).into();
                trunc_epsilon_vec2(&mut linvel);
                changed |= linvel.ui(ui, Default::default(), context);
                self.set_linvel(linvel.into(), false);
                ui.end_row();

                ui.label("Angular velocity");
                let mut angvel = self.angvel();
                changed |= angvel.ui(ui, Default::default(), context);
                self.set_angvel(angvel, false);
                ui.end_row();

                self.wake_up(false);
            });
        });
        changed
    }
}

fn trunc_epsilon_vec2(val: &mut bevy::math::Vec2) {
    super::trunc_epsilon_f32(&mut val.x);
    super::trunc_epsilon_f32(&mut val.y);
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
