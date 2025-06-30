use std::rc::Rc;

use palette::num::ClampAssign;

use crate::{
    hit::{HitRecord, Hittable},
    material::{Isotropic, Material},
    texture::Texture,
    utils::{
        color::Color,
        interval::Interval,
        random::Random,
        vec3::{UnitVec3, Vec3},
    },
};

pub struct ConstantMedium {
    boundary: Box<dyn Hittable>,
    neg_inv_density: f64,
    phase_function: Box<dyn Material>,
}

impl ConstantMedium {
    pub fn new_with_tex(
        boundary: Box<dyn Hittable>,
        density: f64,
        texture: Rc<dyn Texture>,
    ) -> ConstantMedium {
        ConstantMedium {
            boundary,
            neg_inv_density: -1.0 / density,
            phase_function: Box::new(Isotropic::new(texture)),
        }
    }

    pub fn new_with_color(
        boundary: Box<dyn Hittable>,
        density: f64,
        albedo: Color,
    ) -> ConstantMedium {
        ConstantMedium {
            boundary,
            neg_inv_density: -1.0 / density,
            phase_function: Box::new(Isotropic::from_color(albedo)),
        }
    }
}

impl Hittable for ConstantMedium {
    fn hit(
        &self,
        r: &crate::utils::ray::Ray,
        interval: &crate::utils::interval::Interval,
    ) -> Option<crate::hit::HitRecord> {
        // 如果要实现非凸的体积，需要对射线进行背面剔除，体积始终是封闭的

        let mut rec1 = self.boundary.hit(r, &Interval::UNIVERSE)?;
        let mut rec2 = self
            .boundary
            .hit(r, &Interval::new(rec1.t + 0.0001, f64::INFINITY))?;

        rec1.t.clamp_min_assign(*interval.min());
        rec2.t.clamp_max_assign(*interval.max());

        if rec1.t >= rec2.t {
            return None;
        }

        rec1.t.clamp_min_assign(0.0);

        let ray_length = r.direction().length();
        let distance_inside_boundary = (rec2.t - rec1.t) * ray_length;
        let hit_distance = self.neg_inv_density * Random::f64().ln();

        if hit_distance > distance_inside_boundary {
            return None;
        }

        let t = rec1.t + hit_distance / ray_length;
        let p = r.at(t);

        // 对于体积，法线方向是任意取值的
        let normal = UnitVec3::from_vec3_raw(Vec3::new(1.0, 0.0, 0.0));

        let mat = self.phase_function.as_ref();

        Some(HitRecord::new(p, normal, mat, t, 0.0, 0.0, r))
    }

    fn bounding_box(&self) -> &crate::aabb::AABB {
        self.boundary.bounding_box()
    }
}
