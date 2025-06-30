use std::rc::Rc;

use crate::{
    hit::HitRecord,
    texture::{SolidColor, Texture},
    utils::{
        color::Color,
        random::Random,
        ray::Ray,
        vec3::{Point3, UnitVec3},
    },
};

pub trait Material {
    fn scatter(&self, _r_in: &Ray, _rec: &HitRecord) -> Option<(Color, Ray)> {
        None
    }

    fn emitted(&self, _u: f64, _v: f64, _p: &Point3) -> Color {
        Color::BLACK
    }
}

pub struct Lambertian {
    texture: Rc<dyn Texture>,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Lambertian {
        Lambertian {
            texture: Rc::new(SolidColor::new(albedo)),
        }
    }

    pub fn from_tex(texure: Rc<dyn Texture>) -> Lambertian {
        Lambertian { texture: texure }
    }
}

impl Material for Lambertian {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let raw_scatter_direction =
            rec.normal.as_inner() + UnitVec3::random_unit_vector().as_inner();
        let scatter_direction = if raw_scatter_direction.near_zero() {
            rec.normal.into_inner()
        } else {
            raw_scatter_direction
        };
        Some((
            self.texture.value(rec.u, rec.v, &rec.p),
            Ray::new_with_time(rec.p, scatter_direction, *r_in.time()),
        ))
    }
}

pub struct Metal {
    albedo: Color,
    fuzz: f64,
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Metal {
        Metal {
            albedo,
            fuzz: fuzz.clamp(0.0, 1.0),
        }
    }
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let raw_reflected = UnitVec3::from_vec3(*r_in.direction())?.reflect(&rec.normal);
        let reflected = UnitVec3::from_vec3(raw_reflected)?.into_inner()
            + (self.fuzz * UnitVec3::random_unit_vector().into_inner());
        Some((
            self.albedo,
            Ray::new_with_time(rec.p, reflected, *r_in.time()),
        ))
    }
}

pub struct Dielectric {
    refraction_index: f64,
}

impl Dielectric {
    pub fn new(refraction_index: f64) -> Dielectric {
        Dielectric { refraction_index }
    }

    fn reflectance(cosine: f64, refraction_index: f64) -> f64 {
        let r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
        let r0_squared = r0 * r0;
        r0_squared + (1.0 - r0_squared) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let ri = if rec.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };
        let unit_direction = UnitVec3::from_vec3(*r_in.direction()).unwrap();
        let cos_theta = (-unit_direction).dot(&rec.normal).min(1.0);
        let sin_thera = (1.0 - cos_theta * cos_theta).sqrt();
        let cannot_refract = ri * sin_thera > 1.0;

        Some((
            Color::WHITE,
            Ray::new_with_time(
                rec.p,
                if cannot_refract || Dielectric::reflectance(cos_theta, ri) > Random::f64() {
                    unit_direction.reflect(&rec.normal)
                } else {
                    unit_direction
                        .refract(&rec.normal, ri)
                        .unwrap()
                        .into_inner()
                },
                *r_in.time(),
            ),
        ))
    }
}

pub struct DiffuseLight {
    texture: Rc<dyn Texture>,
}

impl DiffuseLight {
    pub fn from_color(emit: Color) -> DiffuseLight {
        DiffuseLight {
            texture: Rc::new(SolidColor::new(emit)),
        }
    }

    pub fn from_tex(texture: Rc<dyn Texture>) -> DiffuseLight {
        DiffuseLight { texture }
    }
}

impl Material for DiffuseLight {
    fn emitted(&self, u: f64, v: f64, p: &Point3) -> Color {
        self.texture.value(u, v, p)
    }
}

pub struct Isotropic {
    texture: Rc<dyn Texture>,
}

impl Isotropic {
    pub fn new(texture: Rc<dyn Texture>) -> Isotropic {
        Isotropic { texture }
    }

    pub fn from_color(albedo: Color) -> Isotropic {
        Isotropic {
            texture: Rc::new(SolidColor::new(albedo)),
        }
    }
}

impl Material for Isotropic {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        Some((
            self.texture.value(rec.u, rec.v, &rec.p),
            Ray::new_with_time(
                rec.p,
                UnitVec3::random_unit_vector().into_inner(),
                *r_in.time(),
            ),
        ))
    }
}
