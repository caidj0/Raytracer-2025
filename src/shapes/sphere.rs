use std::ops::Range;

use crate::{
    hit::{HitRecord, Hittable}, material::Material, utils::vec3::Point3
};

pub struct Sphere {
    center: Point3,
    radius: f64,
    mat: Box<dyn Material>,
}

impl Sphere {
    pub fn new(center: Point3, radius: f64, mat: Box<dyn Material>) -> Sphere {
        Sphere {
            center: center,
            radius: if radius > 0.0 { radius } else { 0.0 },
            mat: mat
        }
    }
}

impl Hittable for Sphere {
    fn hit(
        &self,
        r: &crate::utils::ray::Ray,
        interval: &Range<f64>,
    ) -> Option<crate::hit::HitRecord> {
        let oc = self.center - r.origin();
        let a = r.direction().length_squared();
        let h = r.direction().dot(&oc);
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = h * h - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        let mut root = (h - sqrtd) / a;
        if !interval.contains(&root) {
            root = (h + sqrtd) / a;
            if !interval.contains(&root) {
                return None;
            }
        }

        let p = r.at(root);
        let normal = (p - self.center) / self.radius;
        let outward_normal = (p - self.center) / self.radius;
        let mut hr = HitRecord {
            p: p,
            normal: normal,
            mat: &self.mat,
            t: root,
            front_face: false,
        };
        hr.set_face_normal(r, &outward_normal);
        Some(hr)
    }
}
