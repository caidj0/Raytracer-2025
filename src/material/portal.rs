use crate::{
    hit::HitRecord,
    material::{Material, ScatterRecord},
    utils::{color::Color, quaternion::Quaternion, ray::Ray, vec3::Vec3},
};

type RayPortalFn = Box<dyn Fn(&Ray, &HitRecord) -> Ray + Send + Sync>;

pub struct Portal {
    pub attenuation: Color,
    pub f: RayPortalFn,
}

impl Portal {
    pub fn new(attenuation: Color, position_offset: Vec3, rotation: Quaternion) -> Portal {
        Portal {
            attenuation,
            f: Box::new(move |ray, rec| {
                let origin = rec.p + position_offset;
                let direction = rotation.rotate_vector(*ray.direction());
                Ray::new_with_time(origin, direction, *ray.time())
            }),
        }
    }
}

impl Material for Portal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<super::ScatterRecord> {
        Some(ScatterRecord {
            attenuation: self.attenuation,
            scatter_type: super::ScatterType::Ray((self.f)(r_in, rec)),
        })
    }
}
