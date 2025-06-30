use crate::{
    aabb::AABB,
    material::Material,
    utils::{
        interval::Interval,
        ray::Ray,
        vec3::{Point3, UnitVec3, Vec3},
    },
};

pub struct HitRecord<'a> {
    pub p: Point3, // 击中位置
    pub normal: UnitVec3,
    pub mat: &'a dyn Material,
    pub t: f64, // 射线长度

    pub u: f64,
    pub v: f64, // 撞击点表面坐标

    pub front_face: bool,
}

impl<'a> HitRecord<'a> {
    pub fn new(
        p: Point3,
        normal: UnitVec3,
        mat: &'a dyn Material,
        t: f64,
        u: f64,
        v: f64,
        r_in: &Ray,
    ) -> HitRecord<'a> {
        let front_face = r_in.direction().dot(&normal) < 0.0;
        HitRecord {
            p,
            normal: if front_face { normal } else { -normal },
            mat,
            t,
            u,
            v,
            front_face,
        }
    }
}

pub trait Hittable {
    fn hit(&self, r: &Ray, interval: &Interval) -> Option<HitRecord>;

    fn bounding_box(&self) -> &AABB;

    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {
        0.0
    }

    fn random(&self, origin: &Point3) -> Vec3 {
        Vec3::new(1.0, 0.0, 0.0)
    }
}

pub struct Translate {
    object: Box<dyn Hittable>,
    offset: Vec3,
    bbox: AABB,
}

impl Translate {
    pub fn new(object: Box<dyn Hittable>, offset: Vec3) -> Translate {
        Translate {
            bbox: *object.bounding_box() + offset,
            object,
            offset,
        }
    }
}

impl Hittable for Translate {
    fn hit(&self, r: &Ray, interval: &Interval) -> Option<HitRecord> {
        let offset_r = Ray::new_with_time(r.origin() - self.offset, *r.direction(), *r.time());

        let mut rec = self.object.hit(&offset_r, interval)?;

        rec.p += self.offset;

        Some(rec)
    }

    fn bounding_box(&self) -> &AABB {
        &self.bbox
    }
}

pub struct RotateY {
    object: Box<dyn Hittable>,
    sin_theta: f64,
    cos_theta: f64,
    bbox: AABB,
}

impl RotateY {
    pub fn new(object: Box<dyn Hittable>, angle_in_degrees: f64) -> RotateY {
        let radians = angle_in_degrees.to_radians();
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();
        let bbox = object.bounding_box();

        let xyz_iter = (0..2).flat_map(|i| {
            (0..2).flat_map(move |j| {
                (0..2).map(move |k| {
                    let x = if i == 0 {
                        *bbox.x().min()
                    } else {
                        *bbox.x().max()
                    };
                    let y = if j == 0 {
                        *bbox.y().min()
                    } else {
                        *bbox.y().max()
                    };
                    let z = if k == 0 {
                        *bbox.z().min()
                    } else {
                        *bbox.z().max()
                    };

                    let newx = cos_theta * x + sin_theta * z;
                    let newz = -sin_theta * x + cos_theta * z;

                    Vec3::new(newx, y, newz)
                })
            })
        });

        let (min, max): (Point3, Point3) = xyz_iter.fold(
            (
                Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
                Point3::new(-f64::INFINITY, -f64::INFINITY, -f64::INFINITY),
            ),
            |x, y| {
                (
                    Vec3::from(
                        (0..3)
                            .map(|c| f64::min(x.0[c], y[c]))
                            .collect::<Vec<_>>()
                            .try_into()
                            .unwrap(),
                    ),
                    Vec3::from(
                        (0..3)
                            .map(|c| f64::max(x.0[c], y[c]))
                            .collect::<Vec<_>>()
                            .try_into()
                            .unwrap(),
                    ),
                )
            },
        );

        let bbox = AABB::from_points(min, max);

        RotateY {
            object,
            sin_theta,
            cos_theta,
            bbox,
        }
    }
}

impl Hittable for RotateY {
    fn hit(&self, r: &Ray, interval: &Interval) -> Option<HitRecord> {
        let transform = |p: Point3, cos_theta: f64, sin_theta: f64| {
            Point3::new(
                cos_theta * p.x() - sin_theta * p.z(),
                p.y(),
                sin_theta * p.x() + cos_theta * p.z(),
            )
        };

        let origin = transform(*r.origin(), self.cos_theta, self.sin_theta);

        let direction = transform(*r.direction(), self.cos_theta, self.sin_theta);

        let rotated_ray = Ray::new_with_time(origin, direction, *r.time());

        let mut rec = self.object.hit(&rotated_ray, interval)?;

        rec.p = transform(rec.p, self.cos_theta, -self.sin_theta);
        rec.normal = UnitVec3::from_vec3_raw(transform(
            rec.normal.into_inner(),
            self.cos_theta,
            -self.sin_theta,
        ));

        Some(rec)
    }

    fn bounding_box(&self) -> &AABB {
        &self.bbox
    }
}
