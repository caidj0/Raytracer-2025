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
}

impl Metal {
    pub fn new(albedo: &Color) -> Metal {
        Metal { albedo: *albedo }
    }
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let reflected = r_in.direction().reflect(&rec.normal);
        Some((self.albedo, Ray::new(rec.p, reflected)))
    }
}
