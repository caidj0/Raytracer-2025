use std::{f64::consts::PI, rc::Rc};

use crate::{
    aabb::AABB,
    hit::{HitRecord, Hittable},
    material::Material,
    utils::{
        interval::Interval,
        onb::OrthonormalBasis,
        random::Random,
        ray::Ray,
        vec3::{Point3, UnitVec3, Vec3},
    },
};

#[derive(Clone)]
pub struct Sphere {
    center: Ray,
    radius: f64,
    mat: Rc<dyn Material>,
    bbox: AABB,
}

impl Sphere {
    pub fn new(static_center: Point3, radius: f64, mat: Rc<dyn Material>) -> Sphere {
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
        mat: Rc<dyn Material>,
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

    fn get_sphere_uv(p: UnitVec3) -> (f64, f64) {
        let theta = f64::acos(-p.y());
        let phi = f64::atan2(-p.z(), p.x()) + PI;

        let u = phi / (2.0 * PI);
        let v = theta / PI;

        (u, v)
    }

    fn random_to_sphere(radius: f64, distance_squared: f64) -> Vec3 {
        let r1 = Random::f64();
        let r2 = Random::f64();
        let z = 1.0 + r2 * ((1.0 - radius * radius / distance_squared).sqrt() - 1.0);

        let phi = 2.0 * PI * r1;
        let x = phi.cos() * (1.0 - z * z).sqrt();
        let y = phi.sin() * (1.0 - z * z).sqrt();

        Vec3::new(x, y, z)
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
        let (u, v) = Sphere::get_sphere_uv(outward_normal);
        let hr = HitRecord::new(p, outward_normal, self.mat.as_ref(), root, u, v, r);
        Some(hr)
    }

    fn bounding_box(&self) -> &AABB {
        &self.bbox
    }

    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {
        // 只适用于静态球

        let Some(_) = self.hit(
            &Ray::new(*origin, *direction),
            &Interval::new(0.001, f64::INFINITY),
        ) else {
            return 0.0;
        };

        let dist_squared = (self.center.at(0.0) - origin).length_squared();
        let cos_theta_max = (1.0 - self.radius * self.radius / dist_squared).sqrt();
        let solid_angle = 2.0 * PI * (1.0 - cos_theta_max);

        1.0 / solid_angle
    }

    fn random(&self, origin: &Point3) -> Vec3 {
        let direction = self.center.at(0.0) - origin;
        let distance_squared = direction.length_squared();
        let uvw = OrthonormalBasis::new(
            &UnitVec3::from_vec3(direction).expect("The direction should be normalizable!"),
        );

        uvw.transform(Self::random_to_sphere(self.radius, distance_squared))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_uv() {
        let (u, v) = Sphere::get_sphere_uv(UnitVec3::from_vec3_raw(Vec3::new(1.0, 0.0, 0.0)));
        assert_eq!((u, v), (0.5, 0.5));

        let (u, v) = Sphere::get_sphere_uv(UnitVec3::from_vec3_raw(Vec3::new(-1.0, 0.0, 0.0)));
        assert_eq!((u, v), (0.0, 0.5));

        let (u, v) = Sphere::get_sphere_uv(UnitVec3::from_vec3_raw(Vec3::new(0.0, 1.0, 0.0)));
        assert_eq!((u, v), (0.5, 1.0));

        let (u, v) = Sphere::get_sphere_uv(UnitVec3::from_vec3_raw(Vec3::new(0.0, -1.0, 0.0)));
        assert_eq!((u, v), (0.5, 0.0));

        let (u, v) = Sphere::get_sphere_uv(UnitVec3::from_vec3_raw(Vec3::new(0.0, 0.0, 1.0)));
        assert_eq!((u, v), (0.25, 0.5));

        let (u, v) = Sphere::get_sphere_uv(UnitVec3::from_vec3_raw(Vec3::new(0.0, 0.0, -1.0)));
        assert_eq!((u, v), (0.75, 0.5));
    }
}
