use crate::{
    hit::HitRecord,
    utils::{color::Color, ray::Ray, vec3::Vec3},
};

pub trait Material {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)>;
}

pub struct Lambertian {
    albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: &Color) -> Lambertian {
        Lambertian { albedo: *albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let raw_scatter_direction = rec.normal + Vec3::random_unit_vector();
        let scatter_direction = if raw_scatter_direction.near_zero() {
            rec.normal
        } else {
            raw_scatter_direction
        };
        Some((self.albedo, Ray::new(rec.p, scatter_direction)))
    }
}

pub struct Metal {
    albedo: Color,
    fuzz: f64,
}

impl Metal {
    pub fn new(albedo: &Color, fuzz: f64) -> Metal {
        Metal {
            albedo: *albedo,
            fuzz: fuzz.clamp(0.0, 1.0),
        }
    }
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let raw_reflected = r_in.direction().reflect(&rec.normal);
        let reflected = raw_reflected.unit_vector() + (self.fuzz * Vec3::random_unit_vector());
        Some((self.albedo, Ray::new(rec.p, reflected)))
    }
}
