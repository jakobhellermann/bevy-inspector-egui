use bevy::prelude::*;
use nalgebra::{Isometry3, Quaternion, Translation3, Unit, UnitQuaternion, Vector3};

pub trait NalgebraQuatExt {
    fn to_glam_quat(&self) -> Quat;
}

impl NalgebraQuatExt for UnitQuaternion<f32> {
    fn to_glam_quat(&self) -> Quat {
        let quat = self.into_inner().coords;
        Quat::from_xyzw(quat.x, quat.y, quat.z, quat.w)
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
                Translation3::new(self.translation.x, self.translation.y, self.translation.z),
                self.rotation.to_na_unit_quat(),
            ),
            self.scale.into(),
        )
    }
}

impl TransformExt for GlobalTransform {
    fn to_na_isometry(&self) -> (Isometry3<f32>, Vector3<f32>) {
        (
            Isometry3::from_parts(
                Translation3::new(self.translation.x, self.translation.y, self.translation.z),
                self.rotation.to_na_unit_quat(),
            ),
            self.scale.into(),
        )
    }
}
