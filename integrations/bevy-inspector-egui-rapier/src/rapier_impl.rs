use crate::macros::inspectable;

use bevy_inspector_egui::{
    egui::{self, ComboBox},
    options::NumberAttributes,
    Context, Inspectable, InspectableRegistry,
};

use bevy_rapier::geometry::Collider;
use bevy_rapier::prelude::*;

inspectable!(defer ccd Ccd: enabled);
inspectable!(defer gravity_scale GravityScale: 0);
inspectable!(marker sensor Sensor);
inspectable!(enum active_collision_types ActiveCollisionTypes: DYNAMIC_DYNAMIC|DYNAMIC_KINEMATIC|DYNAMIC_STATIC|KINEMATIC_KINEMATIC|KINEMATIC_STATIC|STATIC_STATIC);
inspectable!(enum active_hooks ActiveHooks: FILTER_CONTACT_PAIRS|FILTER_INTERSECTION_PAIR|MODIFY_SOLVER_CONTACTS);
inspectable!(enum coefficient_combine_rule CoefficientCombineRule: Average|Min|Multiply|Max);
inspectable!(enum rigid_body RigidBody: Dynamic|Fixed|KinematicPositionBased|KinematicVelocityBased);
inspectable!(enum_one collider_scale ColliderScale: Relative {VectAttributes::min(Vect::splat(0.01))} | Absolute {VectAttributes::min(Vect::splat(0.01))});
inspectable!(flags locked_axes LockedAxes: TRANSLATION_LOCKED_X|TRANSLATION_LOCKED_Y|TRANSLATION_LOCKED_Z|ROTATION_LOCKED_X|ROTATION_LOCKED_Y|ROTATION_LOCKED_Z);
inspectable!(grid collision_groups CollisionGroups: memberships, filters);
inspectable!(grid damping Damping: linear_damping, angular_damping);
inspectable!(grid dominance Dominance: groups);
inspectable!(grid external_force ExternalForce: force, torque);
inspectable!(grid external_impulse ExternalImpulse: impulse, torque_impulse);
inspectable!(grid friction Friction: coefficient with NumberAttributes::positive().with_speed(0.01), combine_rule by coefficient_combine_rule);
inspectable!(grid restitution Restitution: coefficient with NumberAttributes::positive().with_speed(0.01), combine_rule by coefficient_combine_rule);
inspectable!(grid sleeping Sleeping: linear_threshold, angular_threshold, sleeping);
inspectable!(grid solver_groups SolverGroups: memberships, filters);
inspectable!(grid transform_interpolation TransformInterpolation: start, end);
inspectable!(grid velocity Velocity: linvel, angvel);

fn additional_mass_properties(
    val: &mut AdditionalMassProperties,
    ui: &mut egui::Ui,
    context: &mut Context<'_>,
) -> bool {
    let selected = match val {
        AdditionalMassProperties::Mass(_) => "Mass",
        AdditionalMassProperties::MassProperties(_) => "MassProperties",
    };

    ComboBox::from_id_source(context.id())
        .selected_text(selected)
        .show_ui(ui, |ui| {
            if ui
                .selectable_label(matches!(val, AdditionalMassProperties::Density(_)), "Mass")
                .clicked()
            {
                *val = AdditionalMassProperties::Mass(1.0);
            }
            if ui
                .selectable_label(
                    matches!(val, AdditionalMassProperties::MassProperties(_)),
                    "MassProperties",
                )
                .clicked()
            {
                *val = AdditionalMassProperties::MassProperties(MassProperties::from_rapier(
                    bevy_rapier::rapier::dynamics::MassProperties::from_ball(1.0, 1.0),
                    1.0,
                ));
            }
        });

    match val {
        AdditionalMassProperties::Mass(mass) => {
            mass.ui(ui, NumberAttributes::positive(), context)
        }
        AdditionalMassProperties::MassProperties(props) => mass_properties(props, ui, context),
    }
}

fn collider_mass_properties(
    val: &mut ColliderMassProperties,
    ui: &mut egui::Ui,
    context: &mut Context<'_>,
) -> bool {
    let selected = match val {
        ColliderMassProperties::Density(_) => "Density",
        ColliderMassProperties::Mass(_) => "Mass",
        ColliderMassProperties::MassProperties(_) => "MassProperties",
    };

    ComboBox::from_id_source(context.id())
        .selected_text(selected)
        .show_ui(ui, |ui| {
            if ui
                .selectable_label(matches!(val, ColliderMassProperties::Density(_)), "Density")
                .clicked()
            {
                *val = ColliderMassProperties::Density(1.0);
            }
            if ui
                .selectable_label(matches!(val, ColliderMassProperties::Density(_)), "Mass")
                .clicked()
            {
                *val = ColliderMassProperties::Mass(1.0);
            }
            if ui
                .selectable_label(
                    matches!(val, ColliderMassProperties::MassProperties(_)),
                    "MassProperties",
                )
                .clicked()
            {
                *val = ColliderMassProperties::MassProperties(MassProperties::from_rapier(
                    bevy_rapier::rapier::dynamics::MassProperties::from_ball(1.0, 1.0),
                    1.0,
                ));
            }
        });

    match val {
        ColliderMassProperties::Density(density) => {
            density.ui(ui, NumberAttributes::positive(), context)
        }
        ColliderMassProperties::Mass(mass) => {
            mass.ui(ui, NumberAttributes::positive(), context)
        }
        ColliderMassProperties::MassProperties(props) => mass_properties(props, ui, context),
    }
}

fn mass_properties(
    props_: &mut MassProperties,
    ui: &mut egui::Ui,
    context: &mut Context<'_>,
) -> bool {
    fn inv(val: f32) -> f32 {
        if val == 0.0 {
            val
        } else {
            1.0 / val
        }
    }

    let mut props = props_.into_rapier(1.0);

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

    *props_ = MassProperties::from_rapier(props, 1.0);

    changed
}

fn collider(val: &mut Collider, ui: &mut egui::Ui, context: &mut Context<'_>) -> bool {
    let mut changed = false;

    egui::Grid::new("collider").show(ui, |ui| {
        ui.label("shape");
        changed |= collider_shape(val, ui, context);
        ui.end_row();
    });

    changed
}

fn collider_shape(val: &mut Collider, ui: &mut egui::Ui, _context: &mut Context<'_>) -> bool {
    let shape_name = match val.as_unscaled_typed_shape() {
        ColliderView::Ball(_) => "Ball",
        ColliderView::Cuboid(_) => "Cuboid",
        ColliderView::Capsule(_) => "Capsule",
        ColliderView::Segment(_) => "Segment",
        ColliderView::Triangle(_) => "Triangle",
        ColliderView::TriMesh(_) => "TriMesh",
        ColliderView::Polyline(_) => "Polyline",
        ColliderView::HalfSpace(_) => "HalfSpace",
        ColliderView::HeightField(_) => "HeightField",
        ColliderView::Compound(_) => "Compound",
        ColliderView::RoundCuboid(_) => "RoundCuboid",
        ColliderView::RoundTriangle(_) => "RoundTriangle",
        _ => "Unknown",
        // 2d
        // ColliderView::ConvexPolygon(_) => "ConvexPolygon",
        // ColliderView::RoundConvexPolygon(_) => "RoundConvexPolygon",
        // 3d
        // ColliderView::ConvexPolyhedron(_) => "ConvexPolyhedron",
        // ColliderView::Cylinder(_) => "Cylinder",
        // ColliderView::Cone(_) => "Cone",
        // ColliderView::RoundCylinder(_) => "RoundCylinder",
        // ColliderView::RoundCone(_) => "RoundCone",
        // ColliderView::RoundConvexPolyhedron(_) => "RoundConvexPolyhedron",
    };

    ui.label(shape_name);
    false
}

pub fn register(inspectable_registry: &mut InspectableRegistry) {
    inspectable_registry.register_raw::<ActiveCollisionTypes, _>(active_collision_types);
    inspectable_registry.register_raw::<ActiveHooks, _>(active_hooks);
    inspectable_registry.register_raw::<AdditionalMassProperties, _>(additional_mass_properties);
    inspectable_registry.register_raw::<Ccd, _>(ccd);
    inspectable_registry.register_raw::<Collider, _>(collider);
    inspectable_registry.register_raw::<ColliderMassProperties, _>(collider_mass_properties);
    inspectable_registry.register_raw::<ColliderScale, _>(collider_scale);
    inspectable_registry.register_raw::<CollisionGroups, _>(collision_groups);
    inspectable_registry.register_raw::<Damping, _>(damping);
    inspectable_registry.register_raw::<Dominance, _>(dominance);
    inspectable_registry.register_raw::<ExternalForce, _>(external_force);
    inspectable_registry.register_raw::<ExternalImpulse, _>(external_impulse);
    inspectable_registry.register_raw::<Friction, _>(friction);
    inspectable_registry.register_raw::<GravityScale, _>(gravity_scale);
    inspectable_registry.register_raw::<LockedAxes, _>(locked_axes);
    inspectable_registry.register_raw::<MassProperties, _>(mass_properties);
    inspectable_registry.register_raw::<RapierColliderHandle, _>(noop_inspectable);
    inspectable_registry.register_raw::<RapierImpulseJointHandle, _>(noop_inspectable);
    inspectable_registry.register_raw::<RapierMultibodyJointHandle, _>(noop_inspectable);
    inspectable_registry.register_raw::<Restitution, _>(restitution);
    inspectable_registry.register_raw::<RigidBody, _>(rigid_body);
    inspectable_registry.register_raw::<Sensor, _>(sensor);
    inspectable_registry.register_raw::<Sleeping, _>(sleeping);
    inspectable_registry.register_raw::<SolverGroups, _>(solver_groups);
    inspectable_registry.register_raw::<TransformInterpolation, _>(transform_interpolation);
    inspectable_registry.register_raw::<Velocity, _>(velocity);
}

fn noop_inspectable<T>(_val: &mut T, _ui: &mut egui::Ui, _context: &mut Context<'_>) -> bool {
    false
}

/*
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
*/
