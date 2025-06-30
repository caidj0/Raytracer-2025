use std::f64::consts::PI;

use crate::{
    hit::Hittable,
    utils::{
        onb::OrthonormalBasis,
        random::Random,
        vec3::{Point3, UnitVec3, Vec3},
    },
};

pub trait PDF {
    fn value(&self, direction: &Vec3) -> f64;
    fn generate(&self) -> Vec3; // 需不需要是单位向量？
}

pub struct SpherePDF;

impl PDF for SpherePDF {
    fn value(&self, _direction: &Vec3) -> f64 {
        1.0 / (4.0 * PI)
    }

    fn generate(&self) -> Vec3 {
        UnitVec3::random_unit_vector().into_inner()
    }
}

pub struct CosinePDF {
    uvw: OrthonormalBasis,
}

impl CosinePDF {
    pub fn new(w: &UnitVec3) -> CosinePDF {
        CosinePDF {
            uvw: OrthonormalBasis::new(w),
        }
    }
}

impl PDF for CosinePDF {
    fn value(&self, direction: &Vec3) -> f64 {
        let cosine_theta = Vec3::dot(&UnitVec3::from_vec3(*direction).unwrap(), &self.uvw.w());
        f64::max(0.0, cosine_theta / PI)
    }

    fn generate(&self) -> Vec3 {
        self.uvw
            .transform(UnitVec3::random_cosine_direction().into_inner())
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
    fn value(&self, direction: &Vec3) -> f64 {
        self.objects.pdf_value(&self.origin, direction)
    }

    fn generate(&self) -> Vec3 {
        self.objects.random(&self.origin)
    }
}

pub struct MixturePDF<'a> {
    p: [&'a dyn PDF; 2],
}

impl<'a> MixturePDF<'a> {
    pub fn new(p0: &'a dyn PDF, p1: &'a dyn PDF) -> MixturePDF<'a> {
        MixturePDF { p: [p0, p1] }
    }
}

impl<'a> PDF for MixturePDF<'a> {
    fn value(&self, direction: &Vec3) -> f64 {
        0.5 * self.p[0].value(direction) + 0.5 * self.p[1].value(direction)
    }

    fn generate(&self) -> Vec3 {
        if Random::f64() < 0.5 {
            self.p[0].generate()
        } else {
            self.p[1].generate()
        }
    }
}
