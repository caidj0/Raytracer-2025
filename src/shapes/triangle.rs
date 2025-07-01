use crate::{
    aabb::AABB,
    hit::{HitRecord, Hittable},
    material::Material,
    shapes::Planar,
    utils::{
        interval::Interval,
        random::Random,
        ray::Ray,
        vec3::{Point3, UnitVec3, Vec3},
    },
};

pub struct Triangle<'a> {
    anchor: Point3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    mat: &'a dyn Material,
    bbox: AABB,
    normal: UnitVec3,
    parm_d: f64,
    area: f64,
}

impl<'a> Triangle<'a> {
    pub fn new(anchor: Point3, u: Vec3, v: Vec3, mat: &'a dyn Material) -> Triangle<'a> {
        let n = Vec3::cross(&u, &v);
        let normal = UnitVec3::from_vec3(n).expect("The length of normal should be normalizable!");
        let parm_d = normal.dot(&anchor);
        let w = n / n.length_squared();
        let area = n.length() / 2.0;
        Triangle {
            anchor,
            u,
            v,
            w,
            mat,
            bbox: Triangle::cal_bounding_box(&anchor, &u, &v),
            normal,
            parm_d,
            area,
        }
    }
}

impl<'a> Planar for Triangle<'a> {
    fn cal_bounding_box(anchor: &Point3, u: &Vec3, v: &Vec3) -> AABB {
        let bbox1 = AABB::from_points(*anchor, anchor + u);
        let bbox2 = AABB::from_points(*anchor, anchor + v);

        AABB::union(bbox1, bbox2)
    }

    fn is_interior(a: f64, b: f64) -> Option<(f64, f64)> {
        const UNIT_INTERVAL: Interval = Interval::new(0.0, 1.0);

        if UNIT_INTERVAL.contains(a) && UNIT_INTERVAL.contains(b) && UNIT_INTERVAL.contains(a + b) {
            Some((a, b))
        } else {
            None
        }
    }
}

impl<'a> Hittable for Triangle<'a> {
    fn hit(&self, r: &Ray, interval: &Interval) -> Option<HitRecord> {
        // 从 Quad 的 Hit 复制而来

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

        let (u, v) = Triangle::is_interior(alpha, beta)?;

        Some(HitRecord::new(
            intersection,
            self.normal,
            self.mat,
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

    fn random(&self, origin: &Point3) -> Vec3 {
        let mut u_l = Random::f64();
        let mut v_l = Random::f64();

        if u_l + v_l > 1.0 {
            (u_l, v_l) = (1.0 - v_l, 1.0 - u_l);
        }

        let p = self.anchor + (u_l * self.u) + (v_l * self.v);
        p - origin
    }
}
