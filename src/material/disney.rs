use std::{f64::consts::PI, sync::Arc};

use crate::{
    material::Material,
    texture::Texture,
    utils::{
        color::Color,
        fresnel::{schlick_f64, schlick_weight},
        lerp,
        vec3::{UnitVec3, Vec3},
    },
};

pub struct Disney {
    base_color: Color,

    sheen: f64,
    sheen_tint: f64,
    clearcoat: f64,
    clearcoat_alpha: f64,
}

impl Material for Disney {
    fn scatter(
        &self,
        r_in: &crate::utils::ray::Ray,
        rec: &crate::hit::HitRecord,
    ) -> Option<super::ScatterRecord> {
        None
    }

    fn emitted(&self, r_in: &crate::utils::ray::Ray, rec: &crate::hit::HitRecord) -> Color {
        Color::BLACK
    }

    fn scattering_pdf(
        &self,
        r_in: &crate::utils::ray::Ray,
        rec: &crate::hit::HitRecord,
        scattered: &crate::utils::ray::Ray,
    ) -> f64 {
        // unimplemented!("If using pdf, this function should be overloaded!")
        0.0
    }
}

impl Disney {
    fn evaluate_sheen(&self, v_out: &UnitVec3, v_half: &UnitVec3, v_in: &UnitVec3) -> Color {
        if self.sheen <= 0.0 {
            return Color::BLACK;
        }

        let dot_hl = v_half.dot(v_in);
        let tint = calculate_tint(self.base_color);
        self.sheen * lerp(Vec3::new(1.0, 1.0, 1.0), tint, self.sheen_tint) * schlick_weight(dot_hl)
    }

    fn evaluate_clearcoat(
        &self,
        v_out: &UnitVec3,
        v_half: &UnitVec3,
        v_in: &UnitVec3,
    ) -> (f64, f64, f64) {
        if self.clearcoat <= 0.0 {
            return (0.0, 0.0, 0.0);
        }

        // N, H, L, V 分别代表 法线，半向量，入射向量，出射向量
        let dot_nh = v_half.y();
        let dot_nl = v_in.y();
        let dot_nv = v_out.y();
        let dot_hl = v_half.dot(&v_in);

        let d = gtr1(dot_nh, lerp(0.1, 0.001, self.clearcoat_alpha));
        let f = schlick_f64(0.04, dot_hl);
        let gl = separable_smith_ggxg1(v_in, 0.25);
        let gv = separable_smith_ggxg1(v_out, 0.25);

        let value = 0.25 * self.clearcoat * d * f * gl * gv;
        let forward_pdf = d / (4.0 * dot_nl.abs());
        let reverse_pdf = d / (4.0 * dot_nv.abs());

        (value, forward_pdf, reverse_pdf)
    }
}

fn calculate_tint(base_color: Color) -> Color {
    let luminance = Color::new(0.3, 0.6, 1.0).dot(&base_color);

    if luminance > 0.0 {
        base_color * (1.0 / luminance)
    } else {
        Color::WHITE
    }
}

fn gtr1(dot_hl: f64, a: f64) -> f64 {
    if a >= 1.0 {
        return 1.0 / PI;
    }

    let a2 = a * a;

    (a2 - 1.0) / (PI * a2.ln() * (1.0 + (a2 - 1.0) * dot_hl * dot_hl))
}

fn separable_smith_ggxg1(w: &UnitVec3, a: f64) -> f64 {
    let a2 = a * a;
    let dot_nv = w.y();

    2.0 / (1.0 + (a2 + (1.0 - a2) * dot_nv * dot_nv).sqrt())
}
