use crate::{
    hit::{HitRecord, Hittable},
    utils::vec3::Point3,
};

pub struct Sphere {
    center: Point3,
    radius: f64,
}

impl Sphere {
    pub fn new(center: Point3, radius: f64) -> Sphere {
        Sphere {
            center,
            radius: if radius > 0.0 { radius } else { 0.0 },
        }
    }
}

impl Hittable for Sphere {
    fn hit(
        &self,
        r: &crate::utils::ray::Ray,
        ray_tmin: f64,
        ray_tmax: f64,
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
        if root <= ray_tmin || ray_tmax <= root {
            root = (h + sqrtd) / a;
            if root <= ray_tmin || ray_tmax <= root {
                return None;
            }
        }

        let p = r.at(root);
        let normal = (p - self.center) / self.radius;
        let outward_normal = (p - self.center) / self.radius;
        let mut hr = HitRecord {
            p,
            normal,
            t: root,
            front_face: false,
        };
        hr.set_face_normal(r, &outward_normal);
        Some(hr)
    }
}
