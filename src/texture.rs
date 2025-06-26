use crate::utils::{color::Color, vec3::Point3};

pub trait Texture {
    fn value(&self, u: f64, v: f64, p: &Point3) -> Color;
}

pub struct SolidColor {
    albedo: Color,
}

impl SolidColor {
    pub fn new(albedo: Color) -> SolidColor {
        Self { albedo }
    }

    pub fn from_rgb(red: f64, green: f64, bule: f64) -> SolidColor {
        Self {
            albedo: Color::new(red, green, bule),
        }
    }
}

impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _p: &Point3) -> Color {
        self.albedo
    }
}

pub struct CheckerTexture {
    inv_scale: f64,
    even: Box<dyn Texture>,
    odd: Box<dyn Texture>,
}

impl CheckerTexture {
    pub fn new(
        scale: f64,
        even_texture: Box<dyn Texture>,
        odd_texture: Box<dyn Texture>,
    ) -> CheckerTexture {
        CheckerTexture {
            inv_scale: 1.0 / scale,
            even: even_texture,
            odd: odd_texture,
        }
    }

    pub fn from_colors(scale: f64, even_color: Color, odd_color: Color) -> CheckerTexture {
        CheckerTexture {
            inv_scale: 1.0 / scale,
            even: Box::new(SolidColor::new(even_color)),
            odd: Box::new(SolidColor::new(odd_color)),
        }
    }
}

impl Texture for CheckerTexture {
    fn value(&self, u: f64, v: f64, p: &Point3) -> Color {
        let x_int = f64::floor(self.inv_scale * p.x()) as i32;
        let y_int = f64::floor(self.inv_scale * p.y()) as i32;
        let z_int = f64::floor(self.inv_scale * p.z()) as i32;

        let is_even = (x_int + y_int + z_int) % 2 == 0;

        if is_even {
            self.even.value(u, v, p)
        } else {
            self.odd.value(u, v, p)
        }
    }
}
