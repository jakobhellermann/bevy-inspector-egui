use crate::inspectable;
use crate::{flags, grid, simple_enum};

use bevy_inspector_egui::{
    egui::{self, ComboBox},
    options::NumberAttributes,
    Context, Inspectable, InspectableRegistry, WorldInspectorParams,
};

inspectable!(flags collider_changes ColliderChanges as ColliderChangesComponent: MODIFIED|PARENT|POSITION|GROUPS|SHAPE|TYPE|PARENT_EFFECTIVE_DOMINANCE);
inspectable!(grid collider_material ColliderMaterial as ColliderMaterialComponent: friction with NumberAttributes::positive().with_speed(0.01), restitution with NumberAttributes::normalized().with_speed(0.01), friction_combine_rule by coefficient_combine_rule, restitution_combine_rule by coefficient_combine_rule);
inspectable!(grid collider_parent ColliderParent as ColliderParentComponent: pos_wrt_parent); // handle?
inspectable!(enum collider_type ColliderType as ColliderTypeComponent: Solid|Sensor);
inspectable!(defer collider_position ColliderPosition as ColliderPositionComponent: 0);

inspectable!(grid rigidbody_activation RigidBodyActivation as RigidBodyActivationComponent: linear_threshold with NumberAttributes::min(-1.0), angular_threshold with NumberAttributes::min(-1.0), time_since_can_sleep, sleeping);
inspectable!(grid rigidbody_ccd RigidBodyCcd as RigidBodyCcdComponent: ccd_thickness with NumberAttributes::positive(), ccd_max_dist with NumberAttributes::positive(), ccd_active, ccd_enabled);
inspectable!(flags rigidbody_changes RigidBodyChanges as RigidBodyChangesComponent: MODIFIED|POSITION|SLEEP|COLLIDERS|TYPE|DOMINANCE);
inspectable!(grid rigidbody_damping RigidBodyDamping as RigidBodyDampingComponent: linear_damping with NumberAttributes::positive(), angular_damping with NumberAttributes::positive());
inspectable!(defer rigidbody_dominance RigidBodyDominance as RigidBodyDominanceComponent: 0);
inspectable!(grid rigidbody_position RigidBodyPosition as RigidBodyPositionComponent: position, next_position);
inspectable!(grid rigidbody_velocity RigidBodyVelocity as RigidBodyVelocityComponent: linvel, angvel);
inspectable!(enum rigidbody_type RigidBodyType as RigidBodyTypeComponent: Dynamic|Static|KinematicPositionBased|KinematicVelocityBased);

fn coefficient_combine_rule(
    coefficient_combine_rule: &mut CoefficientCombineRule,
    ui: &mut egui::Ui,
    context: &mut Context<'_>,
) -> bool {
    simple_enum!(ui context coefficient_combine_rule; CoefficientCombineRule: Average|Min|Multiply|Max)
}

fn collider_debug_render(
    val: &mut ColliderDebugRender,
    ui: &mut egui::Ui,
    context: &mut Context<'_>,
) -> bool {
    val.color.ui(ui, Default::default(), context)
}

fn collider_mass_props(
    val: &mut ColliderMassPropsComponent,
    ui: &mut egui::Ui,
    context: &mut Context<'_>,
) -> bool {
    let val = &mut val.0;

    let selected = match val {
        ColliderMassProps::Density(_) => "Density",
        ColliderMassProps::MassProperties(_) => "MassProperties",
    };

    ComboBox::from_id_source(context.id())
        .selected_text(selected)
        .show_ui(ui, |ui| {
            if ui
                .selectable_label(matches!(val, ColliderMassProps::Density(_)), "Density")
                .clicked()
            {
                *val = ColliderMassProps::Density(1.0);
            }
            if ui
                .selectable_label(
                    matches!(val, ColliderMassProps::MassProperties(_)),
                    "MassProperties",
                )
                .clicked()
            {
                *val = ColliderMassProps::MassProperties(Box::new(MassProperties::from_ball(
                    1.0, 1.0,
                )));
            }
        });

    match val {
        ColliderMassProps::Density(density) => {
            density.ui(ui, NumberAttributes::positive(), context)
        }
        ColliderMassProps::MassProperties(props) => {
            let props = &mut **props;
            mass_properties(props, ui, context)
        }
    }
}

fn inv(val: f32) -> f32 {
    if val == 0.0 {
        val
    } else {
        1.0 / val
    }
}

fn mass_properties(
    props: &mut MassProperties,
    ui: &mut egui::Ui,
    context: &mut Context<'_>,
) -> bool {
    let mut mass_changed = false;
    let mut changed = false;

    egui::Grid::new("collision groups").show(ui, |ui| {
        ui.label("local_com");
        changed |= props.local_com.ui(ui, Default::default(), context);
        ui.end_row();

        let mut mass = inv(props.inv_mass);
        ui.label("mass");
        mass_changed = mass.ui(ui, NumberAttributes::positive(), context);
        changed |= mass_changed;
        ui.end_row();

        if mass_changed {
            props.set_mass(mass, true);
        }
    });

    changed
}

fn collider_shape(
    val: &mut ColliderShapeComponent,
    ui: &mut egui::Ui,
    _context: &mut Context<'_>,
) -> bool {
    let shape: &mut SharedShape = &mut val.0;

    #[rustfmt::skip]
    let shape_name = if shape.0.is::<Ball>() { "Ball" }
    else if shape.0.is::<Capsule>() { "Capsule" }
    else if shape.0.is::<Cuboid>() { "Cuboid" }
    else if shape.0.is::<HeightField>() { "HeightField" }
    else if shape.0.is::<Segment>() { "Segment" }
    else if shape.0.is::<Triangle>() { "Triangle" }
    else if shape.0.is::<Compound>() { "Compound" }
    else if shape.0.is::<Polyline>() { "Polyline" }
    else if shape.0.is::<TriMesh>() { "TriMesh" }
    // else if shape.0.is::<ConvexPolyhedron>() { "Compound" }
    // else if shape.0.is::<Cylinder>() { "Cylinder" }
    // else if shape.0.is::<Cone>() { "Cone" }
    // else if shape.0.is::<ConvexPolyhedron>() { "ConvexPolyhedron" }
    else { "Unknown" };

    ui.label(shape_name);
    false
}

fn collider_flags(
    val: &mut ColliderFlagsComponent,
    ui: &mut egui::Ui,
    context: &mut Context<'_>,
) -> bool {
    let mut changed = false;
    ui.label("Active Collision Types:");
    changed |= flags!(ui context &mut val.active_collision_types; ActiveCollisionTypes: DYNAMIC_DYNAMIC|DYNAMIC_KINEMATIC|DYNAMIC_STATIC|KINEMATIC_KINEMATIC|KINEMATIC_STATIC|STATIC_STATIC);

    ui.label("Collision groups");
    ui.separator();
    changed |= grid!(ui context &mut val.collision_groups; memberships, filter);

    ui.label("Solver groups");
    ui.separator();
    changed |= grid!(ui context &mut val.solver_groups; memberships, filter);

    ui.label("Active hooks");
    changed |= flags!(ui context &mut val.active_hooks; ActiveHooks: FILTER_CONTACT_PAIRS|FILTER_INTERSECTION_PAIR|MODIFY_SOLVER_CONTACTS);

    ui.label("Active events");
    changed |=
        flags!(ui context &mut val.active_events; ActiveEvents: INTERSECTION_EVENTS|CONTACT_EVENTS);

    changed
}

fn rigidbody_colliders(
    val: &mut RigidBodyCollidersComponent,
    ui: &mut egui::Ui,
    _: &mut Context<'_>,
) -> bool {
    ui.label("Colliders");
    let colliders = &val.0 .0;
    for collider in colliders {
        ui.label(format!("- {:?}", collider));
    }
    false
}

fn rigidbody_massproperties(
    val: &mut RigidBodyMassPropsComponent,
    ui: &mut egui::Ui,
    context: &mut Context<'_>,
) -> bool {
    let val = &mut val.0;
    let mut changed = false;
    ui.label("flags");
    changed |= flags!(ui context &mut val.flags; RigidBodyMassPropsFlags: TRANSLATION_LOCKED_X|TRANSLATION_LOCKED_Y|TRANSLATION_LOCKED_Z|ROTATION_LOCKED_X|ROTATION_LOCKED_Y|ROTATION_LOCKED_Z);

    ui.label("local_mprops");
    egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
        changed |= mass_properties(&mut val.local_mprops, ui, context);
    });

    ui.label("world_com");
    changed |= val.world_com.ui(ui, Default::default(), context);

    changed
}

fn rigidbody_forces(
    val: &mut RigidBodyForcesComponent,
    ui: &mut egui::Ui,
    context: &mut Context<'_>,
) -> bool {
    let forces = &mut val.0;

    let mut changed = false;
    egui::Grid::new("forces").show(ui, |ui| {
        ui.label("force");

        let force_id = context.id().with("force");
        let mut force = *ui.memory().data.get_temp_mut_or(force_id, forces.force);
        let mut force_open = *ui.memory().data.get_temp_mut_or(force_id, true);
        let force_changed = force.ui(ui, Default::default(), context);
        if force_changed {
            ui.memory().data.insert_temp(force_id, force);
        }
        if ui.checkbox(&mut force_open, "Apply").clicked() {
            ui.memory().data.insert_temp(force_id, force_open);
        }
        if force_open {
            changed |= force_changed;
            forces.force = force;
        }
        ui.end_row();

        ui.label("torque");

        let torque_id = context.id().with("torque");
        let mut torque = *ui.memory().data.get_temp_mut_or(torque_id, forces.torque);
        let mut torque_open = *ui.memory().data.get_temp_mut_or(torque_id, true);
        let torque_changed = torque.ui(ui, Default::default(), context);
        if torque_changed {
            ui.memory().data.insert_temp(torque_id, torque);
        }
        if ui.checkbox(&mut torque_open, "Apply").clicked() {
            ui.memory().data.insert_temp(torque_id, torque_open);
        }
        if torque_open {
            changed |= torque_changed;
            forces.torque = torque;
        }
        ui.end_row();

        ui.label("gravity_scale");
        changed |= forces.gravity_scale.ui(ui, Default::default(), context);

        changed
    });

    changed
}

#[rustfmt::skip]
pub fn register(inspectable_registry: &mut InspectableRegistry) {
    inspectable_registry.register_raw::<ColliderBroadPhaseDataComponent, _>(noop_inspectable);
    inspectable_registry.register_raw::<ColliderChangesComponent, _>(collider_changes);
    inspectable_registry.register_raw::<ColliderFlagsComponent, _>(collider_flags);
    inspectable_registry.register_raw::<ColliderHandleComponent, _>(noop_inspectable);
    inspectable_registry.register_raw::<ColliderMassPropsComponent, _>(collider_mass_props);
    inspectable_registry.register_raw::<ColliderMaterialComponent, _>(collider_material);
    inspectable_registry.register_raw::<ColliderParentComponent, _>(collider_parent);
    inspectable_registry.register_raw::<ColliderPositionComponent, _>(collider_position);
    inspectable_registry.register_raw::<ColliderShapeComponent, _>(collider_shape);
    inspectable_registry.register_raw::<ColliderTypeComponent, _>(collider_type);
    inspectable_registry.register_raw::<RigidBodyActivationComponent, _>(rigidbody_activation);
    inspectable_registry.register_raw::<RigidBodyCcdComponent, _>(rigidbody_ccd);
    inspectable_registry.register_raw::<RigidBodyChangesComponent, _>(rigidbody_changes);
    inspectable_registry.register_raw::<RigidBodyCollidersComponent, _>(rigidbody_colliders);
    inspectable_registry.register_raw::<RigidBodyDampingComponent, _>(rigidbody_damping);
    inspectable_registry.register_raw::<RigidBodyDominanceComponent, _>(rigidbody_dominance);
    inspectable_registry.register_raw::<RigidBodyForcesComponent, _>(rigidbody_forces);
    inspectable_registry.register_raw::<RigidBodyHandleComponent, _>(noop_inspectable);
    inspectable_registry.register_raw::<RigidBodyIdsComponent, _>(noop_inspectable);
    inspectable_registry.register_raw::<RigidBodyMassPropsComponent, _>(rigidbody_massproperties);
    inspectable_registry.register_raw::<RigidBodyPositionComponent, _>(rigidbody_position);
    inspectable_registry.register_raw::<RigidBodyTypeComponent, _>(rigidbody_type);
    inspectable_registry.register_raw::<RigidBodyVelocityComponent, _>(rigidbody_velocity);

    inspectable_registry.register_raw::<ColliderDebugRender, _>(collider_debug_render);
}

pub fn register_params(params: &mut WorldInspectorParams) {
    params.ignore_component::<ColliderBroadPhaseDataComponent>();
    params.ignore_component::<RigidBodyIdsComponent>();
}

fn noop_inspectable<T>(_val: &mut T, _ui: &mut egui::Ui, _context: &mut Context<'_>) -> bool {
    false
}
