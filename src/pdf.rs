use std::f64::consts::PI;

use crate::{
    hit::Hittable,
    material::disney::Disney,
    utils::{
        onb::OrthonormalBasis,
        random::Random,
        vec3::{Point3, UnitVec3, Vec3},
    },
};

pub trait PDF {
    fn value(&self, direction: &Vec3) -> f64;
    fn generate(&self) -> UnitVec3; // 需不需要是单位向量？
}

pub struct SpherePDF;

impl PDF for SpherePDF {
    fn value(&self, _direction: &Vec3) -> f64 {
        1.0 / (4.0 * PI)
    }

    fn generate(&self) -> UnitVec3 {
        UnitVec3::random_unit_vector()
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
        let cosine_theta = Vec3::dot(&UnitVec3::from_vec3(*direction).unwrap(), self.uvw.v());
        f64::max(0.0, cosine_theta / PI)
    }

    fn generate(&self) -> UnitVec3 {
        UnitVec3::from_vec3_raw(
            self.uvw
                .onb_to_world(UnitVec3::random_cosine_direction().into_inner()),
        )
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

    fn generate(&self) -> UnitVec3 {
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

    fn generate(&self) -> UnitVec3 {
        if Random::f64() < 0.5 {
            self.p[0].generate()
        } else {
            self.p[1].generate()
        }
    }
}

pub struct DisneyPDF<'a> {
    material: &'a Disney,
    uvw: OrthonormalBasis,
    v_in: UnitVec3,
}

impl<'a> DisneyPDF<'a> {
    pub fn new(material: &'a Disney, normal: &UnitVec3, v_in: &UnitVec3) -> Self {
        let uvw = OrthonormalBasis::new(normal);
        let v_in_local = UnitVec3::from_vec3_raw(uvw.world_to_onb(v_in.into_inner()));

        Self {
            material,
            uvw,
            v_in: v_in_local,
        }
    }
}

impl<'a> PDF for DisneyPDF<'a> {
    fn value(&self, direction: &Vec3) -> f64 {
        let v_out = UnitVec3::from_vec3_raw(
            self.uvw
                .world_to_onb(UnitVec3::from_vec3(*direction).unwrap().into_inner()),
        );
        self.material.calculate_total_pdf(&v_out, &self.v_in)
    }

    fn generate(&self) -> UnitVec3 {
        let (v_out_local, _) = self.material.sample_disney_bsdf(&self.v_in);
        UnitVec3::from_vec3_raw(self.uvw.onb_to_world(v_out_local.into_inner()))
    }
}
