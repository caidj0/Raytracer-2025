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

pub fn schlick_r0_from_relative_ior(eta: f64) -> f64 {
    (eta - 1.0).powi(2) / (eta + 1.0).powi(2)
}

pub fn dielectric(cos_theta_in: f64, n_in: f64, n_out: f64) -> f64 {
    let mut cos_theta_in = cos_theta_in.clamp(-1.0, 1.0);

    let (mut n_in, mut n_out) = (n_in, n_out);

    if cos_theta_in < 0.0 {
        (n_in, n_out) = (n_out, n_in);
        cos_theta_in = -cos_theta_in;
    }

    let sin_theta_in = (1.0 - cos_theta_in * cos_theta_in).max(0.0).sqrt();
    let sin_theta_out = n_in / n_out * sin_theta_in;

    if sin_theta_out >= 1.0 {
        return 1.0;
    }

    let cos_theta_out = (1.0 - sin_theta_out * sin_theta_out).max(0.0).sqrt();

    let r_parallel = (n_out * cos_theta_in - n_in * cos_theta_out)
        / (n_out * cos_theta_in + n_in * cos_theta_out);
    let r_perp = (n_in * cos_theta_in - n_out * cos_theta_out)
        / (n_in * cos_theta_in + n_out * cos_theta_out);

    (r_parallel * r_parallel + r_perp * r_perp) / 2.0
}
