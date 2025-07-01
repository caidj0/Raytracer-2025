use std::{f64::consts::PI, ops::Mul};

use crate::utils::vec3::{UnitVec3, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Quaternion {
    w: f64,
    x: f64,
    y: f64,
    z: f64,
}

impl Quaternion {
    pub fn identity() -> Quaternion {
        Quaternion {
            w: 1.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn from_euler(yaw: f64, pitch: f64, roll: f64) -> Quaternion {
        let cy = (0.5 * yaw).cos();
        let sy = (0.5 * yaw).sin();
        let cp = (0.5 * pitch).cos();
        let sp = (0.5 * pitch).sin();
        let cr = (0.5 * roll).cos();
        let sr = (0.5 * roll).sin();

        Quaternion {
            w: cr * cp * cy + sr * sp * sy,
            x: sr * cp * cy - cr * sp * sy,
            y: cr * sp * cy + sr * cp * sy,
            z: cr * cp * sy - sr * sp * cy,
        }
    }

    pub fn from_axis_angle(axis: Vec3, angle_in_degrees: f64) -> Quaternion {
        let half = angle_in_degrees.to_radians() * 0.5;
        let s = half.sin();
        let c = half.cos();
        let a = UnitVec3::from_vec3(axis).unwrap();

        Quaternion {
            w: c,
            x: a.x() * s,
            y: a.y() * s,
            z: a.z() * s,
        }
    }

    pub fn to_eular(self) -> (f64, f64, f64) {
        let sinr_cosp = 2.0 * (self.w * self.x + self.y * self.z);
        let cosr_cosp = 1.0 - 2.0 * (self.x * self.x + self.y * self.y);
        let roll = sinr_cosp.atan2(cosr_cosp);

        let sinp = 2.0 * (self.w * self.y - self.z * self.x);
        let pitch = if sinp.abs() >= 1.0 {
            sinp.signum() * PI / 2.0
        } else {
            sinp.asin()
        };

        let siny_cosp = 2.0 * (self.w * self.z + self.x * self.y);
        let cosy_cosp = 1.0 - 2.0 * (self.y * self.y + self.z * self.z);
        let yaw = siny_cosp.atan2(cosy_cosp);

        (yaw, pitch, roll)
    }

    pub fn rotate_vector(self, v: Vec3) -> Vec3 {
        let qv = Quaternion {
            w: 0.0,
            x: v.x(),
            y: v.y(),
            z: v.z(),
        };

        let result = self * qv * self.conjugate();
        Vec3::new(result.x, result.y, result.z)
    }

    pub fn conjugate(self) -> Quaternion {
        Quaternion {
            w: self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Mul for Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: Self) -> Self::Output {
        Quaternion {
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            z: self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn test_identity_quaternion() {
        let q = Quaternion::identity();
        assert_eq!(q.w, 1.0);
        assert_eq!(q.x, 0.0);
        assert_eq!(q.y, 0.0);
        assert_eq!(q.z, 0.0);
    }

    #[test]
    fn test_from_euler_and_to_euler() {
        let yaw = 1.0;
        let pitch = 0.5;
        let roll = -0.3;
        let q = Quaternion::from_euler(yaw, pitch, roll);
        let (y, p, r) = q.to_eular();
        assert!(approx_eq(y, yaw, 1e-10));
        assert!(approx_eq(p, pitch, 1e-10));
        assert!(approx_eq(r, roll, 1e-10));
    }

    #[test]
    fn test_from_axis_angle_90deg_x() {
        let axis = Vec3::new(1.0, 0.0, 0.0);
        let angle = 90.0;
        let q = Quaternion::from_axis_angle(axis, angle);
        // Should rotate (0,1,0) to (0,0,1)
        let v = Vec3::new(0.0, 1.0, 0.0);
        let rotated = q.rotate_vector(v);
        assert!(approx_eq(rotated.x(), 0.0, 1e-10));
        assert!(approx_eq(rotated.y(), 0.0, 1e-10));
        assert!(approx_eq(rotated.z(), 1.0, 1e-10));
    }

    #[test]
    fn test_quaternion_multiplication() {
        let q1 = Quaternion::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), 90.0);
        let q2 = Quaternion::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), 90.0);
        let q = q1 * q2;
        let v = Vec3::new(0.0, 0.0, 1.0);
        let rotated = q.rotate_vector(v);
        assert!(approx_eq(rotated.x(), 1.0, 1e-10));
        assert!(approx_eq(rotated.y(), 0.0, 1e-10));
        assert!(approx_eq(rotated.z(), 0.0, 1e-10));
    }

    #[test]
    fn test_conjugate() {
        let q = Quaternion {
            w: 1.0,
            x: 2.0,
            y: 3.0,
            z: 4.0,
        };
        let qc = q.conjugate();
        assert_eq!(qc.w, 1.0);
        assert_eq!(qc.x, -2.0);
        assert_eq!(qc.y, -3.0);
        assert_eq!(qc.z, -4.0);
    }

    #[test]
    fn test_rotate_vector_identity() {
        let q = Quaternion::identity();
        let v = Vec3::new(1.0, 2.0, 3.0);
        let rotated = q.rotate_vector(v);
        assert!(approx_eq(rotated.x(), v.x(), 1e-10));
        assert!(approx_eq(rotated.y(), v.y(), 1e-10));
        assert!(approx_eq(rotated.z(), v.z(), 1e-10));
    }
}
