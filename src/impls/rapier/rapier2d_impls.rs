use crate::impls::NumberAttributes;
use crate::{Context, Inspectable};
use bevy::prelude::*;
use bevy_egui::egui;
use bevy_rapier2d::prelude::*;
use bevy_rapier2d::rapier::dynamics::{MassProperties, RigidBodyType};

impl_for_simple_enum!(
    RigidBodyType: Dynamic,
    Static,
    KinematicPositionBased,
    KinematicVelocityBased
);

impl_for_struct_delegate_fields!(RigidBodyPosition: position, next_position);
impl_for_struct_delegate_fields!(RigidBodyVelocity: linvel, angvel);
impl_for_struct_delegate_fields!(
    RigidBodyMassProps: flags,
    local_mprops inline _inl,
    // world_com,
    // effective_inv_mass,
    // effective_world_inv_inertia_sqrt
);
impl_for_bitflags!(
    RigidBodyMassPropsFlags: TRANSLATION_LOCKED,
    ROTATION_LOCKED_X,
    ROTATION_LOCKED_Y,
    ROTATION_LOCKED_Z,
);

impl_for_struct_delegate_fields!(RigidBodyForces: gravity_scale, force, torque);
impl_for_struct_delegate_fields!(RigidBodyActivation: threshold, energy, sleeping);
impl_for_struct_delegate_fields!(RigidBodyDamping: linear_damping, angular_damping);
impl_defer_to!(RigidBodyDominance: 0);

impl_for_struct_delegate_fields!(
    RigidBodyCcd: ccd_enabled,
    ccd_active,
    ccd_thickness,
    ccd_max_dist,
);
impl_for_bitflags!(RigidBodyChanges: MODIFIED, POSITION, SLEEP, COLLIDERS, TYPE);

impl_for_simple_enum!(ColliderType: Solid, Sensor);
impl_defer_to!(ColliderPosition: 0);
impl_for_struct_delegate_fields!(
    ColliderMaterial: friction,
    restitution,
    friction_combine_rule,
    restitution_combine_rule
);
impl_for_simple_enum!(CoefficientCombineRule: Average, Min, Max, Multiply);
impl_for_struct_delegate_fields!(
    ColliderFlags: active_collision_types,
    collision_groups,
    solver_groups,
    active_hooks,
    active_events,
);
impl_for_bitflags!(
    ActiveCollisionTypes: DYNAMIC_DYNAMIC,
    DYNAMIC_KINEMATIC,
    DYNAMIC_STATIC,
    KINEMATIC_KINEMATIC,
    KINEMATIC_STATIC,
    STATIC_STATIC,
);
impl_for_struct_delegate_fields!(InteractionGroups: memberships, filter);
impl_for_bitflags!(
    ActiveHooks: FILTER_CONTACT_PAIRS,
    FILTER_INTERSECTION_PAIR,
    MODIFY_SOLVER_CONTACTS
);
impl_for_bitflags!(ActiveEvents: INTERSECTION_EVENTS, CONTACT_EVENTS,);
impl_for_simple_enum!(ColliderPositionSync: Discrete);

impl_for_bitflags!(
    ColliderChanges: MODIFIED,
    PARENT,
    POSITION,
    GROUPS,
    SHAPE,
    TYPE,
);
impl_for_struct_delegate_fields!(ColliderParent: handle, pos_wrt_parent);

impl Inspectable for MassProperties {
    type Attributes = ();

    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        _options: Self::Attributes,
        context: &mut Context,
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

impl Inspectable for RigidBodyHandle {
    type Attributes = ();

    fn ui(&mut self, ui: &mut bevy_egui::egui::Ui, _: Self::Attributes, _: &mut Context) -> bool {
        let entity = self.entity();
        ui.label(format!("{:?}", entity));
        false
    }
}

impl Inspectable for ColliderHandle {
    type Attributes = ();

    fn ui(&mut self, ui: &mut bevy_egui::egui::Ui, _: Self::Attributes, _: &mut Context) -> bool {
        let entity = self.entity();
        ui.label(format!("{:?}", entity));
        false
    }
}

impl Inspectable for SharedShape {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, _: &mut Context) -> bool {
        #[rustfmt::skip]
        let shape = if self.0.is::<Ball>() { "Ball" }
        else if self.0.is::<HalfSpace>() { "HalfSpace" }
        else if self.0.is::<Capsule>() { "Capsule" }
        else if self.0.is::<Segment>() { "Segment" }
        else if self.0.is::<Triangle>() { "Triangle" }
        else if self.0.is::<Polyline>() { "Polyline" }
        else if self.0.is::<TriMesh>() { "TriMesh" }
        else { "Unknown" };

        ui.label(shape);

        false
    }
}

impl Inspectable for RigidBodyColliders {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, _: &mut Context) -> bool {
        ui.horizontal(|ui| {
            for (i, handle) in self.0.iter().enumerate() {
                if i == self.0.len() - 1 {
                    ui.label(format!("{:?}", handle.entity()));
                } else {
                    ui.label(format!("{:?}, ", handle.entity()));
                }
            }
        });
        false
    }
}

impl Inspectable for ColliderMassProps {
    type Attributes = ();

    fn ui(&mut self, ui: &mut egui::Ui, _: Self::Attributes, context: &mut Context) -> bool {
        match self {
            ColliderMassProps::Density(density) => {
                ui.label("Density: ");
                density.ui(ui, NumberAttributes::positive(), context)
            }
            ColliderMassProps::MassProperties(properties) => {
                egui::Grid::new(context.id())
                    .show(ui, |ui| properties.ui(ui, Default::default(), context))
                    .inner
            }
        }
    }
}
