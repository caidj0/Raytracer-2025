use std::{f64::consts::PI, rc::Rc};

use crate::{
    hit::HitRecord,
    texture::{self, SolidColor, Texture},
    utils::{
        color::Color,
        onb::OrthonormalBasis,
        random::Random,
        ray::Ray,
        vec3::{Point3, UnitVec3, Vec3},
    },
};

pub trait Material {
    // 返回值依次为 三原色反射率、反射射线、该反射射线的 pdf
    #[allow(unused_variables)]
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray, f64)> {
        None
    }

    #[allow(unused_variables)]
    fn emitted(&self, u: f64, v: f64, p: &Point3) -> Color {
        Color::BLACK
    }

    #[allow(unused_variables)]
    fn scattering_pdf(&self, r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        0.0
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
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray, f64)> {
        let uvw = OrthonormalBasis::new(&rec.normal);
        let scatter_direction = uvw.transform(UnitVec3::random_cosine_direction().into_inner());
        let scatter_direction = UnitVec3::from_vec3(scatter_direction).unwrap();

        Some((
            self.texture.value(rec.u, rec.v, &rec.p),
            Ray::new_with_time(rec.p, *scatter_direction.as_inner(), *r_in.time()),
            Vec3::dot(&uvw.w(), &scatter_direction) / PI,
        ))
    }

    fn scattering_pdf(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f64 {
        // let cos_theta = Vec3::dot(
        //     &rec.normal,
        //     &UnitVec3::from_vec3(*scattered.direction()).expect("The length of scattered ray should be normalizable!"),
        // );
        // if cos_theta < 0.0 {
        //     0.0
        // } else {
        //     cos_theta / PI
        // }

        1.0 / (2.0 * PI)
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
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray, f64)> {
        let raw_reflected = UnitVec3::from_vec3(*r_in.direction())?.reflect(&rec.normal);
        let reflected = UnitVec3::from_vec3(raw_reflected)?.into_inner()
            + (self.fuzz * UnitVec3::random_unit_vector().into_inner());
        Some((
            self.albedo,
            Ray::new_with_time(rec.p, reflected, *r_in.time()),
            todo!(),
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
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray, f64)> {
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
            todo!()
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
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray, f64)> {
        Some((
            self.texture.value(rec.u, rec.v, &rec.p),
            Ray::new_with_time(
                rec.p,
                UnitVec3::random_unit_vector().into_inner(),
                *r_in.time(),
            ),
            1.0 / (4.0 * PI),
        ))
    }

    fn scattering_pdf(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f64 {
        1.0 / (4.0 * PI)
    }
}
