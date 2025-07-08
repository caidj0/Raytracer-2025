pub mod disney;

use std::{f64::consts::PI, sync::Arc};

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

pub enum ScatterType<'a> {
    PDF(Box<dyn PDF + 'a>),
    Ray(Ray),
}

pub struct ScatterRecord<'a> {
    pub attenuation: Color, // 仅对 Ray 类型有效
    pub scatter_type: ScatterType<'a>,
}

pub trait Material: Send + Sync {
    // 返回值依次为 三原色反射率、反射射线、该反射射线的 pdf
    #[allow(unused_variables)]
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        None
    }

    #[allow(unused_variables)]
    fn emitted(&self, r_in: &Ray, rec: &HitRecord) -> Color {
        Color::BLACK
    }

    #[allow(unused_variables)]
    fn scattering_pdf(&self, r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        // unimplemented!("If using pdf, this function should be overloaded!")
        0.0
    }
}

pub struct EmptyMaterial;

impl Material for EmptyMaterial {
    // 从 Lambertian 复制而来

    fn scatter(&self, _r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let albedo = Color::new(0.75, 0.75, 0.75);
        let pdf_ptr = Box::new(CosinePDF::new(albedo, &rec.normal));

        Some(ScatterRecord {
            attenuation: Color::WHITE, // This value is not used, the real attenuation is calculated in the PDF value
            scatter_type: ScatterType::PDF(pdf_ptr),
        })
    }

    fn scattering_pdf(&self, _r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        let cos_theta = rec
            .normal
            .dot(&UnitVec3::from_vec3(*scattered.direction()).unwrap());
        if cos_theta < 0.0 { 0.0 } else { cos_theta / PI }
    }
}

pub struct Lambertian {
    texture: Arc<dyn Texture>,
}

impl Lambertian {
    pub fn new(texture: Arc<dyn Texture>) -> Lambertian {
        Lambertian { texture }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let albedo = self.texture.value(rec.u, rec.v, &rec.p);
        let pdf_ptr = Box::new(CosinePDF::new(albedo, &rec.normal));

        Some(ScatterRecord {
            attenuation: Color::WHITE, // This value is not used, the real attenuation is calculated in the PDF value
            scatter_type: ScatterType::PDF(pdf_ptr),
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
            scatter_type: ScatterType::Ray(Ray::new_with_time(rec.p, reflected, *r_in.time())),
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
            scatter_type: ScatterType::Ray(Ray::new_with_time(rec.p, direction, *r_in.time())),
        })
    }
}

pub struct DiffuseLight {
    texture: Arc<dyn Texture>,
}

impl DiffuseLight {
    pub fn new(texture: Arc<dyn Texture>) -> DiffuseLight {
        DiffuseLight { texture }
    }
}

impl Material for DiffuseLight {
    fn emitted(&self, _ray: &Ray, rec: &HitRecord) -> Color {
        if rec.front_face {
            self.texture.value(rec.u, rec.v, &rec.p)
        } else {
            Color::BLACK
        }
    }
}

pub struct Isotropic {
    texture: Arc<dyn Texture>,
}

impl Isotropic {
    pub fn new(texture: Arc<dyn Texture>) -> Isotropic {
        Isotropic { texture }
    }
}

impl Material for Isotropic {
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let albedo = self.texture.value(rec.u, rec.v, &rec.p);
        let pdf_ptr = Box::new(SpherePDF { attenuation: albedo });

        Some(ScatterRecord {
            attenuation: Color::WHITE, // This value is not used, the real attenuation is calculated in the PDF value
            scatter_type: ScatterType::PDF(pdf_ptr),
        })
    }

    fn scattering_pdf(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f64 {
        1.0 / (4.0 * PI)
    }
}

pub struct Transparent;

impl Material for Transparent {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        Some(ScatterRecord {
            attenuation: Color::WHITE,
            scatter_type: ScatterType::Ray(Ray::new_with_time(rec.p, *r_in.direction(), *r_in.time())),
        })
    }
}

enum MixRatioType {
    Num(f64),
    Tex(Arc<dyn Texture>),
}

pub struct Mix {
    mat1: Arc<dyn Material>,
    mat2: Arc<dyn Material>,
    ratio: MixRatioType,
}

impl Mix {
    pub fn new(mat1: Arc<dyn Material>, mat2: Arc<dyn Material>, ratio: f64) -> Mix {
        Mix {
            mat1,
            mat2,
            ratio: MixRatioType::Num(ratio),
        }
    }

    pub fn from_tex(
        mat1: Arc<dyn Material>,
        mat2: Arc<dyn Material>,
        tex: Arc<dyn Texture>,
    ) -> Mix {
        Mix {
            mat1,
            mat2,
            ratio: MixRatioType::Tex(tex),
        }
    }

    fn get_ratio(&self, u: f64, v: f64, p: &Point3) -> f64 {
        match &self.ratio {
            MixRatioType::Num(r) => *r,
            MixRatioType::Tex(tex) => tex.value(u, v, p).e().iter().sum::<f64>() / 3.0,
        }
    }
}

impl Material for Mix {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let ratio = self.get_ratio(rec.u, rec.v, &rec.p);
        if Random::f64() > ratio {
            self.mat1.scatter(r_in, rec)
        } else {
            self.mat2.scatter(r_in, rec)
        }
    }

    fn emitted(&self, r_in: &Ray, rec: &HitRecord) -> Color {
        let ratio = self.get_ratio(rec.u, rec.v, &rec.p);
        self.mat1.emitted(r_in, rec) * (1.0 - ratio) + self.mat2.emitted(r_in, rec) * ratio
    }

    fn scattering_pdf(&self, r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        let ratio = self.get_ratio(rec.u, rec.v, &rec.p);
        self.mat1.scattering_pdf(r_in, rec, scattered) * (1.0 - ratio)
            + self.mat2.scattering_pdf(r_in, rec, scattered) * ratio
    }
}