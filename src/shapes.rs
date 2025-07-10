use crate::{
    aabb::AABB,
    hit::Hittable,
    utils::{
        quaternion::Quaternion,
        ray::Ray,
        vec3::{Point3, UnitVec3, Vec3},
    },
};

pub mod environment;
pub mod obj;
pub mod quad;
pub mod sphere;
pub mod triangle;

pub trait Planar {
    fn cal_bounding_box(anchor: &Point3, u: &Vec3, v: &Vec3) -> AABB;

    fn is_interior(a: f64, b: f64) -> Option<(f64, f64)>;
}

pub struct Transform {
    object: Box<dyn Hittable>,
    offset: Vec3,
    quaternion: Quaternion,
    scale: Vec3,
    bbox: AABB,
}

impl Transform {
    pub fn new(
        object: Box<dyn Hittable>,
        offset: Option<Vec3>,
        quaternion: Option<Quaternion>,
        scale: Option<Vec3>,
    ) -> Transform {
        let mut t = Transform {
            bbox: AABB::EMPTY,
            object,
            offset: offset.unwrap_or(Vec3::ZERO),
            quaternion: quaternion.unwrap_or(Quaternion::identity()),
            scale: scale.unwrap_or(Vec3::new(1.0, 1.0, 1.0)),
        };
        t.calculate_bbox();
        t
    }

    fn calculate_bbox(&mut self) {
        let points = self.object.bounding_box().all_points();

        let (min, max) = points.iter().map(|p| self.transform(*p)).fold(
            (
                Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
                Vec3::new(-f64::INFINITY, -f64::INFINITY, -f64::INFINITY),
            ),
            |(min, max), p| {
                let min_x = min.x().min(p.x());
                let min_y = min.y().min(p.y());
                let min_z = min.z().min(p.z());
                let max_x = max.x().max(p.x());
                let max_y = max.y().max(p.y());
                let max_z = max.z().max(p.z());
                (
                    Vec3::new(min_x, min_y, min_z),
                    Vec3::new(max_x, max_y, max_z),
                )
            },
        );

        self.bbox = AABB::from_points(min, max);
    }

    fn transform(&self, v: Vec3) -> Vec3 {
        let scaled = v * self.scale;
        let rotated = self.quaternion.rotate_vector(scaled);
        rotated + self.offset
    }

    fn detransform(&self, v: Vec3) -> Vec3 {
        let offseted = v - self.offset;
        let rotated = self.quaternion.conjugate().rotate_vector(offseted);
        rotated / self.scale
    }
}

impl Hittable for Transform {
    fn hit(
        &self,
        r: &crate::utils::ray::Ray,
        interval: &crate::utils::interval::Interval,
    ) -> Option<crate::hit::HitRecord> {
        let origin = r.origin();
        let to = r.at(1.0);

        let local_origin = self.detransform(*origin);
        let local_to = self.detransform(to);

        let local_ray = Ray::new_with_time(local_origin, local_to - local_origin, *r.time());

        let mut rec = self.object.hit(&local_ray, interval)?;

        rec.p = self.transform(rec.p);
        rec.normal = UnitVec3::from_vec3(
            self.quaternion
                .rotate_vector(rec.normal.into_inner() / self.scale),
        )
        .expect("The transformed normal can't be normalized!");

        Some(rec)
    }

    fn bounding_box(&self) -> &AABB {
        &self.bbox
    }
    
    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {
        let local_origin = self.detransform(*origin);
        let local_direction = self.quaternion.conjugate().rotate_vector(*direction);

        self.object.pdf_value(&local_origin, &local_direction)
    }

    fn random(&self, origin: &Point3) -> UnitVec3 {
        let local_origin = self.detransform(*origin);
        let local_dir = self.object.random(&local_origin);
        let world_dir = self.quaternion.rotate_vector(local_dir.into_inner());
        UnitVec3::from_vec3(world_dir).expect("Random direction can't be normalized!")
    }
}
