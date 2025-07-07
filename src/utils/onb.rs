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

        let u = UnitVec3::from_vec3(Vec3::cross(normal, &a)).unwrap();
        let w = UnitVec3::from_vec3_raw(Vec3::cross(&u, normal));

        OrthonormalBasis {
            axis: [u, *normal, w],
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

    pub fn onb_to_world(&self, v: Vec3) -> Vec3 {
        v[0] * self.axis[0].as_inner()
            + v[1] * self.axis[1].as_inner()
            + v[2] * self.axis[2].as_inner()
    }

    pub fn world_to_onb(&self, v: Vec3) -> Vec3 {
        Vec3::new(
            v.dot(self.u().as_inner()),
            v.dot(self.v().as_inner()),
            v.dot(self.w().as_inner()),
        )
    }
}
