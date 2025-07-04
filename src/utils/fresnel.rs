use crate::utils::{lerp, vec3::Vec3};

pub fn schlick(r0: Vec3, radians: f64) -> Vec3 {
    let exp = (1.0 - radians).powi(5);
    r0 + (Vec3::new(1.0, 1.0, 1.0) - r0) * exp
}

pub fn schlick_f64(r0: f64, radians: f64) -> f64 {
    lerp(1.0, schlick_weight(radians), r0)
}

pub fn schlick_weight(u: f64) -> f64 {
    let m = (1.0 - u).clamp(0.0, 1.0);
    m.powi(5)
}

