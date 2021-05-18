mod nalgebra_conversions;
#[cfg(feature = "rapier2d")]
mod rapier2d_impls;
#[cfg(feature = "rapier")]
mod rapier_impls;

fn trunc_epsilon_f32(val: &mut f32) {
    if val.abs() < f32::EPSILON {
        *val = 0.0;
    }
}
