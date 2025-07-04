use std::{f64::consts::PI, sync::Arc};

use crate::{
    texture::Texture,
    utils::{color::Color, ray::Ray, vec3::UnitVec3},
};

#[derive(Debug)]
pub struct Environment {
    pub texture: Arc<dyn Texture>,
}

impl Environment {
    pub fn value(&self, ray: &Ray) -> Color {
        let p = UnitVec3::from_vec3(*ray.direction()).expect("The direction can't be normalized!");

        let theta = f64::acos(-p.y());
        let phi = f64::atan2(-p.z(), p.x()) + PI;

        let u = phi / (2.0 * PI);
        let v = theta / PI;

        self.texture.value(u, v, &p)
    }
}
