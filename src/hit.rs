use crate::{
    aabb::AABB,
    material::Material,
    utils::{
        interval::Interval,
        ray::Ray,
        vec3::{Point3, UnitVec3},
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
        r: &Ray,
    ) -> HitRecord<'a> {
        let front_face = r.direction().dot(&normal) < 0.0;
        HitRecord {
            p,
            normal: if front_face { normal } else { -normal },
            mat,
            t,
            u: 0.0,
            v: 0.0, // TODO
            front_face,
        }
    }
}

pub trait Hittable {
    fn hit(&self, r: &Ray, interval: &Interval) -> Option<HitRecord>;

    fn bounding_box(&self) -> &AABB;
}
