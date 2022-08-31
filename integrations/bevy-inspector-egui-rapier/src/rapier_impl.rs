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
    use bevy_rapier::rapier::dynamics;
    use AdditionalMassProperties::{Mass, MassProperties as Properties};

    let selected = match val {
        Mass(_) => "Mass",
        Properties(_) => "MassProperties",
    };

    ComboBox::from_id_source(context.id())
        .selected_text(selected)
        .show_ui(ui, |ui| {
            macro_rules! clicked {
                ($compare: pat, $name: expr) => {
                    ui.selectable_label(matches!(val, $compare), $name)
                        .clicked()
                };
            }
            match () {
                () if clicked!(Mass(_), "Mass") => *val = Mass(1.0),
                () if clicked!(Properties(_), "MassProperties") => {
                    *val = Properties(MassProperties::from_rapier(
                        dynamics::MassProperties::from_ball(1.0, 1.0),
                        1.0,
                    ))
                }
                () => {}
            };
        });

    match val {
        Mass(mass) => mass.ui(ui, NumberAttributes::positive(), context),
        Properties(props) => mass_properties(props, ui, context),
    }
}

fn collider_mass_properties(
    val: &mut ColliderMassProperties,
    ui: &mut egui::Ui,
    context: &mut Context<'_>,
) -> bool {
    use bevy_rapier::rapier::dynamics;
    use ColliderMassProperties::{Density, Mass, MassProperties as Properties};

    let selected = match val {
        Density(_) => "Density",
        Mass(_) => "Mass",
        Properties(_) => "MassProperties",
    };

    ComboBox::from_id_source(context.id())
        .selected_text(selected)
        .show_ui(ui, |ui| {
            macro_rules! clicked {
                ($compare: pat, $name: expr) => {
                    ui.selectable_label(matches!(val, $compare), $name)
                        .clicked()
                };
            }
            match () {
                () if clicked!(Mass(_), "Mass") => *val = Mass(1.0),
                () if clicked!(Density(_), "Density") => *val = Density(1.0),
                () if clicked!(Properties(_), "MassProperties") => {
                    *val = Properties(MassProperties::from_rapier(
                        dynamics::MassProperties::from_ball(1.0, 1.0),
                        1.0,
                    ))
                }
                () => {}
            }
        });

    match val {
        Properties(props) => mass_properties(props, ui, context),
        Density(density) => density.ui(ui, NumberAttributes::positive(), context),
        Mass(mass) => mass.ui(ui, NumberAttributes::positive(), context),
    }
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

fn collider_shape(old_val: &mut Collider, ui: &mut egui::Ui, context: &mut Context<'_>) -> bool {
    #[cfg(feature = "rapier2d")]
    use bevy::math::Vec2 as Vect;
    #[cfg(feature = "rapier3d")]
    use bevy::math::Vec3 as Vect;
    use bevy_rapier::parry::shape::ShapeType as Cv;

    let mut val = old_val.raw.0.clone_box();
    let shape_name = match val.shape_type() {
        Cv::Ball => "Ball",
        Cv::Cuboid => "Cuboid",
        Cv::Capsule => "Capsule",
        Cv::Segment => "Segment",
        Cv::Triangle => "Triangle",
        Cv::TriMesh => "TriMesh",
        Cv::Polyline => "Polyline",
        Cv::HalfSpace => "HalfSpace",
        Cv::HeightField => "HeightField",
        Cv::Compound => "Compound",
        Cv::RoundCuboid => "RoundCuboid",
        Cv::RoundTriangle => "RoundTriangle",
        // 2d
        #[cfg(feature = "rapier2d")]
        Cv::ConvexPolygon => "ConvexPolygon",
        #[cfg(feature = "rapier2d")]
        Cv::RoundConvexPolygon => "RoundConvexPolygon",
        // 3d
        #[cfg(feature = "rapier3d")]
        Cv::ConvexPolyhedron => "ConvexPolyhedron",
        #[cfg(feature = "rapier3d")]
        Cv::Cylinder => "Cylinder",
        #[cfg(feature = "rapier3d")]
        Cv::Cone => "Cone",
        #[cfg(feature = "rapier3d")]
        Cv::RoundCylinder => "RoundCylinder",
        #[cfg(feature = "rapier3d")]
        Cv::RoundCone => "RoundCone",
        #[cfg(feature = "rapier3d")]
        Cv::RoundConvexPolyhedron => "RoundConvexPolyhedron",
        Cv::Custom => "Custom shape",
    };

    let mut changed = false;
    ComboBox::from_id_source(context.id())
        .selected_text(shape_name)
        .show_ui(ui, |ui| {
            use bevy_rapier::parry::shape::{Ball, Capsule, Cuboid, RoundShape, Triangle};
            #[cfg(feature = "rapier3d")]
            use bevy_rapier::parry::shape::{Cone, Cylinder};

            macro_rules! clicked {
                ($compare: pat, $name: expr) => {
                    ui.selectable_label(matches!(val.shape_type(), $compare), $name)
                        .clicked()
                };
            }
            changed = true;
            match () {
                () if clicked!(Cv::Ball, "Ball") => val = Box::new(Ball::new(1.0)),

                () if clicked!(Cv::Cuboid, "Cuboid") => {
                    val = Box::new(Cuboid::new(Vect::ONE.into()))
                }

                () if clicked!(Cv::Capsule, "Capsule") => val = Box::new(Capsule::new_y(1.0, 1.0)),

                () if clicked!(Cv::Triangle, "Triangle") => {
                    val = Box::new(Triangle::new(
                        Vect::ZERO.into(),
                        Vect::X.into(),
                        Vect::Y.into(),
                    ))
                }
                () if clicked!(Cv::RoundCuboid, "RoundCuboid") => {
                    val = Box::new(RoundShape {
                        inner_shape: Cuboid::new(Vect::ONE.into()),
                        border_radius: 0.2,
                    })
                }
                () if clicked!(Cv::RoundTriangle, "RoundTriangle") => {
                    val = Box::new(RoundShape {
                        inner_shape: Triangle::new(
                            Vect::ZERO.into(),
                            Vect::X.into(),
                            Vect::Y.into(),
                        ),
                        border_radius: 0.2,
                    })
                }

                #[cfg(feature = "rapier3d")]
                () if clicked!(Cv::Cylinder, "Cylinder") => val = Box::new(Cylinder::new(1.0, 0.5)),
                #[cfg(feature = "rapier3d")]
                () if clicked!(Cv::Cone, "Cone") => val = Box::new(Cone::new(1.0, 0.5)),
                #[cfg(feature = "rapier3d")]
                () if clicked!(Cv::RoundCylinder, "RoundCylinder") => {
                    val = Box::new(RoundShape {
                        inner_shape: Cylinder::new(1.0, 0.5),
                        border_radius: 0.1,
                    })
                }
                #[cfg(feature = "rapier3d")]
                () if clicked!(Cv::RoundCone, "RoundCone") => {
                    val = Box::new(RoundShape {
                        inner_shape: Cone::new(1.0, 0.5),
                        border_radius: 0.1,
                    })
                }
                // Uneditable shape, do nothing
                () => changed = false,
            }
        });

    ui.end_row();
    let pos = NumberAttributes::positive;
    if let Some(ball) = val.as_ball_mut() {
        ui.label("radius");
        changed |= ball.radius.ui(ui, pos(), context);
    } else if let Some(cuboid) = val.as_cuboid_mut() {
        ui.label("half extents");
        changed |= cuboid.half_extents.ui(ui, (), context);
    } else if let Some(capsule) = val.as_capsule_mut() {
        ui.label("base point");
        changed |= capsule.segment.a.ui(ui, (), context);
        ui.end_row();

        ui.label("top point");
        changed |= capsule.segment.b.ui(ui, (), context);
        ui.end_row();

        ui.label("radius");
        changed |= capsule.radius.ui(ui, pos(), context);
    } else if let Some(triangle) = val.as_triangle_mut() {
        ui.label("a");
        changed |= triangle.a.ui(ui, (), context);
        ui.end_row();

        ui.label("b");
        changed |= triangle.b.ui(ui, (), context);
        ui.end_row();

        ui.label("c");
        changed |= triangle.c.ui(ui, (), context);
    } else {
        ui.label("Shape doesn't support editing (yet)");
    };
    #[cfg(feature = "rapier3d")]
    if let Some(cylinder) = val.as_cylinder_mut() {
        ui.label("half height");
        changed |= cylinder.half_height.ui(ui, pos(), context);
        ui.end_row();

        ui.label("radius");
        changed |= cylinder.radius.ui(ui, pos(), context);
    } else if let Some(cone) = val.as_cone_mut() {
        ui.label("half height");
        changed |= cone.half_height.ui(ui, pos(), context);
        ui.end_row();

        ui.label("radius");
        changed |= cone.radius.ui(ui, pos(), context);
    };

    if changed {
        *old_val = bevy_rapier::rapier::geometry::SharedShape(val.into()).into();
    }
    changed
}

fn mass_properties(
    props_: &mut MassProperties,
    ui: &mut egui::Ui,
    context: &mut Context<'_>,
) -> bool {
    let inv = |val: f32| if val == 0.0 { val } else { 1.0 / val };

    let mut props = props_.into_rapier(1.0);

    let mut mass_changed = false;
    let mut changed = false;

    egui::Grid::new("collision groups").show(ui, |ui| {
        ui.label("local_com");
        changed |= props.local_com.ui(ui, (), context);
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
