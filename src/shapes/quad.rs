use std::rc::Rc;

use crate::{
    aabb::AABB,
    hit::{HitRecord, Hittable},
    material::Material,
    shapes::Planar,
    utils::{
        interval::Interval,
        vec3::{Point3, UnitVec3, Vec3},
    },
};

pub struct Quad {
    anchor: Point3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    mat: Rc<dyn Material>,
    bbox: AABB,
    normal: UnitVec3,
    parm_d: f64,
}

impl Quad {
    pub fn new(anchor: Point3, u: Vec3, v: Vec3, mat: Rc<dyn Material>) -> Quad {
        let n = Vec3::cross(&u, &v);
        let normal = UnitVec3::from_vec3(n).expect("The length of normal should be normalizable!");
        let parm_d = normal.dot(&anchor);
        let w = n / n.length_squared();
        Quad {
            anchor,
            u,
            v,
            w,
            mat,
            bbox: Quad::cal_bounding_box(&anchor, &u, &v),
            normal,
            parm_d,
        }
    }
}

impl Planar for Quad {
    fn cal_bounding_box(anchor: &Point3, u: &Vec3, v: &Vec3) -> AABB {
        let bbox_diagonal1 = AABB::from_points(*anchor, anchor + u + v);
        let bbox_diagonal2 = AABB::from_points(anchor + u, anchor + v);

        AABB::union(&bbox_diagonal1, &bbox_diagonal2)
    }

    fn is_interior(a: f64, b: f64) -> Option<(f64, f64)> {
        let unit_interval = Interval::new(0.0, 1.0);

        if !unit_interval.contains(a) || !unit_interval.contains(b) {
            None
        } else {
            Some((a, b))
        }
    }
}

impl Hittable for Quad {
    fn hit(
        &self,
        r: &crate::utils::ray::Ray,
        interval: &crate::utils::interval::Interval,
    ) -> Option<crate::hit::HitRecord> {
        let denom = self.normal.dot(r.direction());
        if denom.abs() < 1e-8 {
            return None;
        }

        let t = (self.parm_d - self.normal.dot(r.origin())) / denom;
        if !interval.contains(t) {
            return None;
        }

        let intersection = r.at(t);
        let hit_vector_from_anchor = intersection - self.anchor;
        let alpha = Vec3::dot(&self.w, &Vec3::cross(&hit_vector_from_anchor, &self.v));
        let beta = Vec3::dot(&self.w, &Vec3::cross(&self.u, &hit_vector_from_anchor));

        let (u, v) = Quad::is_interior(alpha, beta)?;

        Some(HitRecord::new(
            intersection,
            self.normal,
            self.mat.as_ref(),
            t,
            u,
            v,
            r,
        ))
    }

    fn bounding_box(&self) -> &AABB {
        &self.bbox
    }
}
