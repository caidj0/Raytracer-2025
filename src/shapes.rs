use crate::{
    aabb::AABB,
    utils::vec3::{Point3, Vec3},
};

pub mod quad;
pub mod sphere;
pub mod triangle;

pub trait Planar {
    fn cal_bounding_box(anchor: &Point3, u: &Vec3, v: &Vec3) -> AABB;

    fn is_interior(a: f64, b: f64) -> Option<(f64, f64)>;
}
