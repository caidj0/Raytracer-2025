use std::rc::Rc;

use crate::utils::{color::Color, image::Image, vec3::Point3};

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
    even: Rc<dyn Texture>,
    odd: Rc<dyn Texture>,
}

impl CheckerTexture {
    pub fn new(
        scale: f64,
        even_texture: Rc<dyn Texture>,
        odd_texture: Rc<dyn Texture>,
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
            even: Rc::new(SolidColor::new(even_color)),
            odd: Rc::new(SolidColor::new(odd_color)),
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

pub struct ImageTexture {
    image: Image,
}

impl ImageTexture {
    pub fn new(file_name: &str) -> ImageTexture {
        ImageTexture {
            image: Image::new(file_name),
        }
    }
}

impl Texture for ImageTexture {
    fn value(&self, u: f64, v: f64, _p: &Point3) -> Color {
        if self.image.height() == 0 {
            return Color::new(0.0, 1.0, 1.0);
        }

        let u = u.clamp(0.0, 1.0);
        let v = 1.0 - v.clamp(0.0, 1.0);

        let i = (u * self.image.width() as f64) as u32;
        let j = (v * self.image.height() as f64) as u32;
        let pixel = self.image.pixel_data(i, j);

        Color::new(pixel.red as f64, pixel.green as f64, pixel.blue as f64)
    }
}
