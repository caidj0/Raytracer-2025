use std::{
    f64::consts::PI,
    fmt::Display,
    iter::Sum,
    ops::{AddAssign, Deref, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Range},
};

use crate::utils::random::Random;

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Vec3 {
    e: [f64; 3],
}

pub type Point3 = Vec3;

impl Vec3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { e: [x, y, z] }
    }

    pub const fn from(e: [f64; 3]) -> Vec3 {
        Vec3 { e }
    }

    pub fn random() -> Vec3 {
        Vec3 {
            e: [Random::f64(), Random::f64(), Random::f64()],
        }
    }

    pub fn random_range(range: Range<f64>) -> Vec3 {
        Vec3 {
            e: [
                Random::random_range(range.clone()),
                Random::random_range(range.clone()),
                Random::random_range(range),
            ],
        }
    }

    pub fn random_in_unit_disk() -> Vec3 {
        let theta = Random::random_range(0.0..(2.0 * PI));
        let r = Random::f64().sqrt();
        Vec3 {
            e: [r * theta.cos(), r * theta.sin(), 0.0],
        }
    }

    pub fn reflect(&self, normal: &UnitVec3) -> Vec3 {
        *self - 2.0 * Vec3::dot(self, normal) * *normal.as_inner()
    }

    pub fn x(&self) -> f64 {
        self.e[0]
    }
    pub fn y(&self) -> f64 {
        self.e[1]
    }
    pub fn z(&self) -> f64 {
        self.e[2]
    }

    pub fn e(&self) -> [f64; 3] {
        self.e
    }

    pub fn length_squared(&self) -> f64 {
        self[0] * self[0] + self[1] * self[1] + self[2] * self[2]
    }

    pub fn near_zero(&self) -> bool {
        let s = 1e-8;
        self[0].abs() < s && self[1].abs() < s && self[2].abs() < s
    }

    pub fn length(&self) -> f64 {
        f64::sqrt(self.length_squared())
    }

    pub fn dot(&self, rhs: &Vec3) -> f64 {
        self[0] * rhs[0] + self[1] * rhs[1] + self[2] * rhs[2]
    }

    pub fn cross(&self, rhs: &Vec3) -> Vec3 {
        Vec3::new(
            self[1] * rhs[2] - self[2] * rhs[1],
            self[2] * rhs[0] - self[0] * rhs[2],
            self[0] * rhs[1] - self[1] * rhs[0],
        )
    }

    pub const ZERO: Vec3 = Vec3::new(0.0, 0.0, 0.0);
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3::new(self * rhs[0], self * rhs[1], self * rhs[2])
    }
}

impl Mul<&Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, rhs: &Vec3) -> Self::Output {
        Vec3::new(self * rhs[0], self * rhs[1], self * rhs[2])
    }
}

impl Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec3::new(self[0] * rhs, self[1] * rhs, self[2] * rhs)
    }
}

impl Mul<f64> for &Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec3::new(self[0] * rhs, self[1] * rhs, self[2] * rhs)
    }
}

impl Index<usize> for Vec3 {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.e[index]
    }
}

impl IndexMut<usize> for Vec3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.e[index]
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self[0] += rhs[0];
        self[1] += rhs[1];
        self[2] += rhs[2];
    }
}

impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        Vec3::new(-self[0], -self[1], -self[2])
    }
}

impl Neg for &Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        Vec3::new(-self[0], -self[1], -self[2])
    }
}

impl MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, rhs: f64) {
        self[0] *= rhs;
        self[1] *= rhs;
        self[2] *= rhs;
    }
}

impl DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, rhs: f64) {
        *self *= 1.0 / rhs
    }
}

impl Div<f64> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f64) -> Self::Output {
        1.0 / rhs * self
    }
}

impl Div<f64> for &Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f64) -> Self::Output {
        1.0 / rhs * self
    }
}

impl Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self[0], self[1], self[2])
    }
}

macro_rules! impl_op {
    ($trait:ident, $func:ident, $op:tt) => {
        impl std::ops::$trait<Vec3> for Vec3 {
            type Output = Vec3;

            fn $func(self, rhs: Vec3) -> Vec3 {
                Vec3::new(self[0] $op rhs[0], self[1] $op rhs[1], self[2] $op rhs[2])
            }
        }

        impl std::ops::$trait<Vec3> for &Vec3 {
            type Output = Vec3;

            fn $func(self, rhs: Vec3) -> Vec3 {
                Vec3::new(self[0] $op rhs[0], self[1] $op rhs[1], self[2] $op rhs[2])
            }
        }

        impl std::ops::$trait<&Vec3> for Vec3 {
            type Output = Vec3;

            fn $func(self, rhs: &Vec3) -> Vec3 {
                Vec3::new(self[0] $op rhs[0], self[1] $op rhs[1], self[2] $op rhs[2])
            }
        }

        impl std::ops::$trait<&Vec3> for &Vec3 {
            type Output = Vec3;

            fn $func(self, rhs: &Vec3) -> Vec3 {
                Vec3::new(self[0] $op rhs[0], self[1] $op rhs[1], self[2] $op rhs[2])
            }
        }
    };
}

impl_op!(Add, add, +);
impl_op!(Sub, sub, -);
impl_op!(Mul, mul, *);
impl_op!(Div, div, /);

impl Sum for Vec3 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Vec3::new(0.0, 0.0, 0.0), |a, b| a + b)
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct UnitVec3(Vec3);

impl UnitVec3 {
    pub const fn from_vec3_raw(vec: Vec3) -> UnitVec3 {
        UnitVec3(vec)
    }

    pub fn from_vec3(vec: Vec3) -> Option<UnitVec3> {
        let v = vec / vec.length();
        if v[0].is_finite() && v[1].is_finite() && v[2].is_finite() {
            Some(UnitVec3(v))
        } else {
            None
        }
    }

    pub fn new(x: f64, y: f64, z: f64) -> Option<UnitVec3> {
        let v = Vec3 { e: [x, y, z] };
        UnitVec3::from_vec3(v)
    }

    pub fn random_unit_vector() -> UnitVec3 {
        let r1 = Random::f64();
        let r2 = Random::f64();

        let x = f64::cos(2.0 * PI * r1) * 2.0 * f64::sqrt(r2 * (1.0 - r2));
        let y = f64::sin(2.0 * PI * r1) * 2.0 * f64::sqrt(r2 * (1.0 - r2));
        let z = 1.0 - 2.0 * r2;

        UnitVec3::from_vec3_raw(Vec3::new(x, y, z))
    }

    pub fn random_on_hemisphere(normal: &UnitVec3) -> UnitVec3 {
        let on_unit_sphere = UnitVec3::random_unit_vector();
        if on_unit_sphere.dot(normal) > 0.0 {
            on_unit_sphere
        } else {
            -on_unit_sphere
        }
    }

    pub fn random_cosine_direction() -> UnitVec3 {
        let r1 = Random::f64();
        let r2 = Random::f64();

        let phi = 2.0 * PI * r1;
        let x = phi.cos() * r2.sqrt();
        let y = phi.sin() * r2.sqrt();
        let z = (1.0 - r2).sqrt();

        UnitVec3::from_vec3_raw(Vec3::new(x, y, z))
    }

    pub fn refract(&self, normal: &UnitVec3, relative_eta: f64) -> Option<UnitVec3> {
        let cos_theta = (-self).dot(normal).min(1.0);
        let out_perp = relative_eta * (self.as_inner() + cos_theta * normal.as_inner());
        let out_parallel_length = (1.0 - out_perp.length_squared()).sqrt();
        if out_parallel_length.is_nan() {
            return None;
        }
        let out_parallel = -out_parallel_length * normal.as_inner();
        Some(UnitVec3::from_vec3_raw(out_perp + out_parallel))
    }

    pub fn into_inner(self) -> Vec3 {
        self.0
    }

    pub fn as_inner(&self) -> &Vec3 {
        &self.0
    }
}

impl Neg for UnitVec3 {
    type Output = UnitVec3;

    fn neg(self) -> Self::Output {
        UnitVec3(-self.0)
    }
}

impl Neg for &UnitVec3 {
    type Output = UnitVec3;

    fn neg(self) -> Self::Output {
        UnitVec3(-self.0)
    }
}

impl Deref for UnitVec3 {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_new_and_getters() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(v.x(), 1.0);
        assert_eq!(v.y(), 2.0);
        assert_eq!(v.z(), 3.0);
    }

    #[test]
    fn test_indexing() {
        let mut v = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(v[0], 1.0);
        assert_eq!(v[1], 2.0);
        assert_eq!(v[2], 3.0);

        v[0] = 4.0;
        v[1] = 5.0;
        v[2] = 6.0;
        assert_eq!(v[0], 4.0);
        assert_eq!(v[1], 5.0);
        assert_eq!(v[2], 6.0);
    }

    #[test]
    fn test_negation() {
        let v = Vec3::new(1.0, -2.0, 3.0);
        assert_eq!(-v, Vec3::new(-1.0, 2.0, -3.0));
    }

    #[test]
    fn test_add() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        assert_eq!(v1 + v2, Vec3::new(5.0, 7.0, 9.0));
    }

    #[test]
    fn test_add_assign() {
        let mut v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        v1 += v2;
        assert_eq!(v1, Vec3::new(5.0, 7.0, 9.0));
    }

    #[test]
    fn test_sub() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        assert_eq!(v1 - v2, Vec3::new(-3.0, -3.0, -3.0));
    }

    #[test]
    fn test_mul_scalar() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(v * 2.0, Vec3::new(2.0, 4.0, 6.0));
        assert_eq!(2.0 * v, Vec3::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn test_mul_assign_scalar() {
        let mut v = Vec3::new(1.0, 2.0, 3.0);
        v *= 2.0;
        assert_eq!(v, Vec3::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn test_mul_vec() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        assert_eq!(v1 * v2, Vec3::new(4.0, 10.0, 18.0));
    }

    #[test]
    fn test_div_scalar() {
        let v = Vec3::new(2.0, 4.0, 6.0);
        assert_eq!(v / 2.0, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_div_assign_scalar() {
        let mut v = Vec3::new(2.0, 4.0, 6.0);
        v /= 2.0;
        assert_eq!(v, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_length() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        assert_eq!(v.length_squared(), 25.0);
        assert_eq!(v.length(), 5.0);
    }

    #[test]
    fn test_dot_product() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        assert_eq!(v1.dot(&v2), 32.0);
    }

    #[test]
    fn test_cross_product() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        assert_eq!(v1.cross(&v2), Vec3::new(-3.0, 6.0, -3.0));
    }

    #[test]
    fn test_unit_vector() {
        let v = Vec3::new(0.0, 5.0, 0.0);
        let unit_v = UnitVec3::from_vec3(v).unwrap();
        assert_eq!(unit_v, UnitVec3::new(0.0, 1.0, 0.0).unwrap());
        assert!((unit_v.length() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_display() {
        let v = Vec3::new(1.1, 2.2, 3.3);
        assert_eq!(format!("{}", v), "1.1 2.2 3.3");
    }
}
