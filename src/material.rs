use std::f64::consts::PI;

use crate::{
    hit::HitRecord,
    pdf::{CosinePDF, PDF, SpherePDF},
    texture::Texture,
    utils::{
        color::Color,
        random::Random,
        ray::Ray,
        vec3::{Point3, UnitVec3},
    },
};

pub enum PDForRay {
    PDF(Box<dyn PDF>),
    Ray(Ray),
}

pub struct ScatterRecord {
    pub attenuation: Color,
    pub pdf_or_ray: PDForRay,
}

pub trait Material: Sync {
    // 返回值依次为 三原色反射率、反射射线、该反射射线的 pdf
    #[allow(unused_variables)]
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        None
    }

    #[allow(unused_variables)]
    fn emitted(&self, r_in: &Ray, rec: &HitRecord, u: f64, v: f64, p: &Point3) -> Color {
        Color::BLACK
    }

    #[allow(unused_variables)]
    fn scattering_pdf(&self, r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        unimplemented!("If using pdf, this function should be overloaded!")
    }
}

pub struct EmptyMaterial;

impl Material for EmptyMaterial {
    // 从 Lambertian 复制而来

    fn scatter(&self, _r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let attenuation = Color::new(0.75, 0.75, 0.75);
        let pdf_ptr = Box::new(CosinePDF::new(&rec.normal));

        Some(ScatterRecord {
            attenuation,
            pdf_or_ray: PDForRay::PDF(pdf_ptr),
        })
    }


    fn scattering_pdf(&self, _r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        let cos_theta = rec
            .normal
            .dot(&UnitVec3::from_vec3(*scattered.direction()).unwrap());
        if cos_theta < 0.0 { 0.0 } else { cos_theta / PI }
    }
}

pub struct Lambertian<'a> {
    texture: &'a dyn Texture,
}

impl<'a> Lambertian<'a> {
    pub fn new(texture: &'a dyn Texture) -> Lambertian<'a> {
        Lambertian { texture }
    }
}

impl<'a> Material for Lambertian<'a> {
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let attenuation = self.texture.value(rec.u, rec.v, &rec.p);
        let pdf_ptr = Box::new(CosinePDF::new(&rec.normal));

        Some(ScatterRecord {
            attenuation,
            pdf_or_ray: PDForRay::PDF(pdf_ptr),
        })
    }

    fn scattering_pdf(&self, _r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        let cos_theta = rec
            .normal
            .dot(&UnitVec3::from_vec3(*scattered.direction()).unwrap());
        if cos_theta < 0.0 { 0.0 } else { cos_theta / PI }
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
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let raw_reflected = UnitVec3::from_vec3(*r_in.direction())?.reflect(&rec.normal);
        let reflected = UnitVec3::from_vec3(raw_reflected)?.into_inner()
            + (self.fuzz * UnitVec3::random_unit_vector().into_inner());

        let attenuation = self.albedo;

        Some(ScatterRecord {
            attenuation,
            pdf_or_ray: PDForRay::Ray(Ray::new_with_time(rec.p, reflected, *r_in.time())),
        })
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
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let attenuation = Color::WHITE;

        let ri = if rec.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };
        let unit_direction = UnitVec3::from_vec3(*r_in.direction()).unwrap();
        let cos_theta = (-unit_direction).dot(&rec.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let cannot_refract = ri * sin_theta > 1.0;

        let direction = if cannot_refract || Dielectric::reflectance(cos_theta, ri) > Random::f64()
        {
            unit_direction.reflect(&rec.normal)
        } else {
            unit_direction
                .refract(&rec.normal, ri)
                .unwrap()
                .into_inner()
        };

        Some(ScatterRecord {
            attenuation,
            pdf_or_ray: PDForRay::Ray(Ray::new_with_time(rec.p, direction, *r_in.time())),
        })
    }
}

pub struct DiffuseLight<'a> {
    texture: &'a dyn Texture,
}

impl<'a> DiffuseLight<'a> {
    pub fn new(texture: &'a dyn Texture) -> DiffuseLight<'a> {
        DiffuseLight { texture }
    }
}

impl<'a> Material for DiffuseLight<'a> {
    fn emitted(&self, _ray: &Ray, rec: &HitRecord, u: f64, v: f64, p: &Point3) -> Color {
        if rec.front_face {
            self.texture.value(u, v, p)
        } else {
            Color::BLACK
        }
    }
}

pub struct Isotropic<'a> {
    texture: &'a dyn Texture,
}

impl<'a> Isotropic<'a> {
    pub fn new(texture: &'a dyn Texture) -> Isotropic<'a> {
        Isotropic { texture }
    }
}

impl<'a> Material for Isotropic<'a> {
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let attenuation = self.texture.value(rec.u, rec.v, &rec.p);
        let pdf_ptr = Box::new(SpherePDF);

        Some(ScatterRecord {
            attenuation,
            pdf_or_ray: PDForRay::PDF(pdf_ptr),
        })
    }

    fn scattering_pdf(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f64 {
        1.0 / (4.0 * PI)
    }
}
