use std::ops::Range;

use crate::{
    material::Material,
    utils::{
        ray::Ray,
        vec3::{Point3, UnitVec3},
    },
};

pub struct HitRecord<'a> {
    pub p: Point3,        // 击中位置
    pub normal: UnitVec3, // 法线，必须为单位矢量
    pub mat: &'a dyn Material,
    pub t: f64, // 射线长度
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
            front_face,
        }
    }
}

pub trait Hittable {
    fn hit(&self, r: &Ray, interval: &Range<f64>) -> Option<HitRecord>;
}
