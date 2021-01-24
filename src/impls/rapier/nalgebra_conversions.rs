use bevy::prelude::*;
use bevy_rapier3d::na::{
    Isometry3, Point3, Quaternion, Translation3, Unit, UnitQuaternion, Vector3,
};

pub trait NalgebraVecExt {
    fn to_glam_vec3(&self) -> Vec3;
}

impl NalgebraVecExt for Vector3<f32> {
    fn to_glam_vec3(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl NalgebraVecExt for Point3<f32> {
    fn to_glam_vec3(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

pub trait NalgebraQuatExt {
    fn to_glam_quat(&self) -> Quat;
}

impl NalgebraQuatExt for UnitQuaternion<f32> {
    fn to_glam_quat(&self) -> Quat {
        let quat = self.into_inner().coords;
        Quat::from_xyzw(quat.x, quat.y, quat.z, quat.w)
    }
}

pub trait GlamVecExt {
    fn to_na_vector3(&self) -> Vector3<f32>;
    fn to_na_point3(&self) -> Point3<f32>;
    fn to_na_translation(&self) -> Translation3<f32>;
}

impl GlamVecExt for Vec3 {
    fn to_na_vector3(&self) -> Vector3<f32> {
        Vector3::new(self.x, self.y, self.z)
    }

    fn to_na_point3(&self) -> Point3<f32> {
        Point3::new(self.x, self.y, self.z)
    }

    fn to_na_translation(&self) -> Translation3<f32> {
        Translation3::new(self.x, self.y, self.z)
    }
}

pub trait GlamQuatExt {
    fn to_na_quat(&self) -> Quaternion<f32>;
    fn to_na_unit_quat(&self) -> UnitQuaternion<f32>;
}

impl GlamQuatExt for Quat {
    fn to_na_quat(&self) -> Quaternion<f32> {
        Quaternion::new(self.w, self.x, self.y, self.z)
    }

    fn to_na_unit_quat(&self) -> UnitQuaternion<f32> {
        Unit::new_normalize(self.to_na_quat())
    }
}

pub trait TransformExt {
    fn to_na_isometry(&self) -> (Isometry3<f32>, Vector3<f32>);
}

impl TransformExt for Transform {
    fn to_na_isometry(&self) -> (Isometry3<f32>, Vector3<f32>) {
        (
            Isometry3::from_parts(
                self.translation.to_na_translation(),
                self.rotation.to_na_unit_quat(),
            ),
            self.scale.to_na_vector3(),
        )
    }
}

impl TransformExt for GlobalTransform {
    fn to_na_isometry(&self) -> (Isometry3<f32>, Vector3<f32>) {
        (
            Isometry3::from_parts(
                self.translation.to_na_translation(),
                self.rotation.to_na_unit_quat(),
            ),
            self.scale.to_na_vector3(),
        )
    }
}
