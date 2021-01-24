use crate::{Context, Inspectable, InspectableWithContext};
use bevy_rapier3d::{
    physics::RigidBodyHandleComponent,
    rapier::dynamics::{RigidBody, RigidBodySet},
};

impl Inspectable for RigidBody {
    type Attributes = ();

    fn ui(&mut self, ui: &mut bevy_egui::egui::Ui, _options: Self::Attributes) {
        ui.label("Mass");
        self.mass().ui(ui, Default::default());
    }
}

impl InspectableWithContext for RigidBodyHandleComponent {
    type Attributes = <RigidBody as Inspectable>::Attributes;

    fn ui_with_context(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        options: Self::Attributes,
        context: &Context,
    ) {
        let mut bodies = match context.resources.get_mut::<RigidBodySet>() {
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

        body.ui_with_context(ui, options, context);
    }
}
