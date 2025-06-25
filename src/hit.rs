use std::ops::Range;

use crate::{
    material::Material,
    utils::{
        ray::Ray,
        vec3::{Point3, Vec3},
    },
};

pub struct HitRecord<'a> {
    pub p: Point3, // 击中位置
    pub normal: Vec3, // 法线，必须为单位矢量
    pub mat: &'a dyn Material,
    pub t: f64, // 射线长度
    pub front_face: bool,
}

impl<'a> HitRecord<'a> {
    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: &Vec3) {
        self.front_face = r.direction().dot(outward_normal) < 0.0;
        self.normal = if self.front_face {
            *outward_normal
        } else {
            -outward_normal
        };
    }
}

pub trait Hittable {
    fn hit(&self, r: &Ray, interval: &Range<f64>) -> Option<HitRecord>;
}
