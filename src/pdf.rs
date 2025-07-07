use std::f64::consts::PI;

use crate::{
    hit::Hittable,
    utils::{
        color::Color, onb::OrthonormalBasis, random::Random, vec3::{Point3, UnitVec3, Vec3}
    },
};

pub trait PDF {
    fn value(&self, direction: &Vec3) -> (Color, f64);
    fn generate(&self) -> Option<UnitVec3>;
}

pub struct SpherePDF {
    pub attenuation: Color,
}

impl PDF for SpherePDF {
    fn value(&self, _direction: &Vec3) -> (Vec3, f64) {
        (self.attenuation, 1.0 / (4.0 * PI))
    }

    fn generate(&self) -> Option<UnitVec3> {
        Some(UnitVec3::random_unit_vector())
    }
}

pub struct CosinePDF {
    attentuation: Color,
    uvw: OrthonormalBasis,
}

impl CosinePDF {
    pub fn new(attentuation: Color, w: &UnitVec3) -> CosinePDF {
        CosinePDF {
            attentuation,
            uvw: OrthonormalBasis::new(w),
        }
    }
}

impl PDF for CosinePDF {
    fn value(&self, direction: &Vec3) -> (Vec3, f64) {
        let cosine_theta = Vec3::dot(&UnitVec3::from_vec3(*direction).unwrap(), self.uvw.v());
        (self.attentuation, f64::max(0.0, cosine_theta / PI))
    }

    fn generate(&self) -> Option<UnitVec3> {
        Some(UnitVec3::from_vec3_raw(
            self.uvw
                .onb_to_world(UnitVec3::random_cosine_direction().into_inner()),
        ))
    }
}

pub struct HittablePDF<'a> {
    objects: &'a dyn Hittable,
    origin: Point3,
}

impl<'a> HittablePDF<'a> {
    pub fn new(objects: &'a dyn Hittable, origin: Point3) -> HittablePDF<'a> {
        HittablePDF { objects, origin }
    }
}

impl<'a> PDF for HittablePDF<'a> {
    fn value(&self, direction: &Vec3) -> (Color, f64) {
        (Color::BLACK, self.objects.pdf_value(&self.origin, direction))
    }

    fn generate(&self) -> Option<UnitVec3> {
        Some(self.objects.random(&self.origin))
    }
}

// 混合两者的 PDF，但只使用前者的 attentuation
pub struct MixturePDF<'a> {
    p: [&'a dyn PDF; 2],
}

impl<'a> MixturePDF<'a> {
    pub fn new(p0: &'a dyn PDF, p1: &'a dyn PDF) -> MixturePDF<'a> {
        MixturePDF { p: [p0, p1] }
    }
}

impl<'a> PDF for MixturePDF<'a> {
    fn value(&self, direction: &Vec3) -> (Color, f64) {
        let (attentuation, value0) = self.p[0].value(direction);
        let (_, value1) = self.p[1].value(direction);
        (attentuation, value0 * 0.5 + value1 * 0.5)
    }

    fn generate(&self) -> Option<UnitVec3> {
        if Random::f64() < 0.5 {
            self.p[0].generate()
        } else {
            self.p[1].generate()
        }
    }
}