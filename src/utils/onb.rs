use crate::utils::vec3::{UnitVec3, Vec3};

pub struct OrthonormalBasis {
    axis: [UnitVec3; 3],
}

impl OrthonormalBasis {
    pub fn new(normal: &UnitVec3) -> OrthonormalBasis {
        let a = if normal.x().abs() > 0.9 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };

        let v = UnitVec3::from_vec3(Vec3::cross(normal, &a)).unwrap();
        let u = UnitVec3::from_vec3_raw(Vec3::cross(&v, normal));

        OrthonormalBasis {
            axis: [u, v, *normal],
        }
    }

    pub fn u(&self) -> &UnitVec3 {
        &self.axis[0]
    }
    pub fn v(&self) -> &UnitVec3 {
        &self.axis[1]
    }
    pub fn w(&self) -> &UnitVec3 {
        &self.axis[2]
    }

    pub fn transform(&self, v: Vec3) -> Vec3 {
        v[0] * self.axis[0].as_inner()
            + v[1] * self.axis[1].as_inner()
            + v[2] * self.axis[2].as_inner()
    }
}
