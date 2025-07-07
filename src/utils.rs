use std::ops::{Add, Mul};

pub mod color;
pub mod fresnel;
pub mod image;
pub mod interval;
pub mod onb;
pub mod perlin;
pub mod quaternion;
pub mod random;
pub mod ray;
pub mod vec3;

pub fn lerp<T>(a: T, b: T, t: f64) -> T
where
    T: Mul<f64, Output = T> + Add<Output = T> + Copy,
{
    a * (1.0 - t) + b * t
}