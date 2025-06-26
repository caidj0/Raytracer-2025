use crate::{
    aabb::AABB,
    hit::{HitRecord, Hittable},
    material::Material,
    utils::{
        interval::Interval,
        ray::Ray,
        vec3::{Point3, UnitVec3, Vec3},
    },
};

pub struct Sphere {
    center: Ray,
    radius: f64,
    mat: Box<dyn Material>,
    bbox: AABB,
}

impl Sphere {
    pub fn new(static_center: Point3, radius: f64, mat: Box<dyn Material>) -> Sphere {
        let rvec = Vec3::new(radius, radius, radius);
        Sphere {
            center: Ray::new(static_center, Vec3::ZERO),
            radius: f64::max(0.0, radius),
            mat,
            bbox: AABB::from_points(static_center - rvec, static_center + rvec),
        }
    }

    pub fn new_with_motion(
        center1: Point3,
        center2: Point3,
        radius: f64,
        mat: Box<dyn Material>,
    ) -> Sphere {
        let rvec = Vec3::new(radius, radius, radius);
        let center = Ray::new(center1, center2 - center1);
        let box1 = AABB::from_points(center.at(0.0) - rvec, center.at(0.0) + rvec);
        let box2 = AABB::from_points(center.at(1.0) - rvec, center.at(1.0) + rvec);
        Sphere {
            center,
            radius: f64::max(0.0, radius),
            mat,
            bbox: AABB::union(&box1, &box2),
        }
    }
}

impl Hittable for Sphere {
    fn hit(
        &self,
        r: &crate::utils::ray::Ray,
        interval: &Interval,
    ) -> Option<crate::hit::HitRecord> {
        let current_center = self.center.at(*r.time());
        let oc = current_center - r.origin();
        let a = r.direction().length_squared();
        let h = r.direction().dot(&oc);
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = h * h - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        let mut root = (h - sqrtd) / a;
        if !interval.contains(root) {
            root = (h + sqrtd) / a;
            if !interval.contains(root) {
                return None;
            }
        }

        let p = r.at(root);
        let outward_normal = UnitVec3::from_vec3_raw((p - current_center) / self.radius);
        let hr = HitRecord::new(p, outward_normal, self.mat.as_ref(), root, r);
        Some(hr)
    }

    fn bounding_box(&self) -> &AABB {
        &self.bbox
    }
}
