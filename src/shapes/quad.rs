use std::sync::Arc;

use crate::{
    aabb::AABB,
    hit::{HitRecord, Hittable},
    hits::Hittables,
    material::Material,
    shapes::Planar,
    utils::{
        interval::Interval,
        random::Random,
        ray::Ray,
        vec3::{Point3, UnitVec3, Vec3},
    },
};

#[derive(Clone)]
pub struct Quad {
    anchor: Point3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    mat: Arc<dyn Material>,
    bbox: AABB,
    normal: UnitVec3,
    parm_d: f64,
    area: f64,
}

impl Quad {
    pub fn new(anchor: Point3, u: Vec3, v: Vec3, mat: Arc<dyn Material>) -> Quad {
        let n = Vec3::cross(&u, &v);
        let normal = UnitVec3::from_vec3(n).expect("The length of normal should be normalizable!");
        let parm_d = normal.dot(&anchor);
        let w = n / n.length_squared();
        let area = n.length();
        Quad {
            anchor,
            u,
            v,
            w,
            mat,
            bbox: Quad::cal_bounding_box(&anchor, &u, &v),
            normal,
            parm_d,
            area,
        }
    }
}

impl Planar for Quad {
    fn cal_bounding_box(anchor: &Point3, u: &Vec3, v: &Vec3) -> AABB {
        let bbox_diagonal1 = AABB::from_points(*anchor, anchor + u + v);
        let bbox_diagonal2 = AABB::from_points(anchor + u, anchor + v);

        AABB::union(bbox_diagonal1, bbox_diagonal2)
    }

    fn is_interior(a: f64, b: f64) -> Option<(f64, f64)> {
        const UNIT_INTERVAL: Interval = Interval::new(0.0, 1.0);

        if !UNIT_INTERVAL.contains(a) || !UNIT_INTERVAL.contains(b) {
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

    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {
        let Some(rec) = self.hit(
            &Ray::new(*origin, *direction),
            &Interval::new(0.001, f64::INFINITY),
        ) else {
            return 0.0;
        };

        let distance_squared = rec.t * rec.t * direction.length_squared();
        let cosine = (direction.dot(&rec.normal) / direction.length()).abs();

        distance_squared / (cosine * self.area)
    }

    fn random(&self, origin: &Point3) -> UnitVec3 {
        let p = self.anchor + (Random::f64() * self.u) + (Random::f64() * self.v);
        UnitVec3::from_vec3(p - origin).unwrap()
    }
}

pub fn build_box(a: Point3, b: Point3, mat: Arc<dyn Material>) -> Hittables {
    let mut sides = Hittables::default();

    let min = Point3::from(
        Iterator::zip(a.e().iter(), b.e().iter())
            .map(|(x, y)| f64::min(*x, *y))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
    );

    let max = Point3::from(
        Iterator::zip(a.e().iter(), b.e().iter())
            .map(|(x, y)| f64::max(*x, *y))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
    );

    let dx = Vec3::new(max.x() - min.x(), 0.0, 0.0);
    let dy = Vec3::new(0.0, max.y() - min.y(), 0.0);
    let dz = Vec3::new(0.0, 0.0, max.z() - min.z());

    sides.add(Box::new(Quad::new(
        Point3::new(min.x(), min.y(), max.z()),
        dx,
        dy,
        mat.clone(),
    )));
    sides.add(Box::new(Quad::new(
        Point3::new(max.x(), min.y(), max.z()),
        -dz,
        dy,
        mat.clone(),
    )));
    sides.add(Box::new(Quad::new(
        Point3::new(max.x(), min.y(), min.z()),
        -dx,
        dy,
        mat.clone(),
    )));
    sides.add(Box::new(Quad::new(
        Point3::new(min.x(), min.y(), min.z()),
        dz,
        dy,
        mat.clone(),
    )));
    sides.add(Box::new(Quad::new(
        Point3::new(min.x(), max.y(), max.z()),
        dx,
        -dz,
        mat.clone(),
    )));
    sides.add(Box::new(Quad::new(
        Point3::new(min.x(), min.y(), min.z()),
        dx,
        dz,
        mat,
    )));

    sides
}
