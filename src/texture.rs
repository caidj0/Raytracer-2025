use std::{fmt::Debug, sync::Arc};

use crate::utils::{color::Color, image::Image, perlin::Perlin, vec3::Point3};

pub trait Texture: Send + Sync + Debug {
    fn value(&self, u: f64, v: f64, p: &Point3) -> Color;
}

#[derive(Debug)]
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

    pub fn from(color: [f64; 3]) -> SolidColor {
        Self {
            albedo: Color::from(color),
        }
    }
}

impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _p: &Point3) -> Color {
        self.albedo
    }
}

#[derive(Debug)]
pub struct CheckerTexture {
    inv_scale: f64,
    even: Arc<dyn Texture>,
    odd: Arc<dyn Texture>,
}

impl CheckerTexture {
    pub fn new(
        scale: f64,
        even_texture: Arc<dyn Texture>,
        odd_texture: Arc<dyn Texture>,
    ) -> CheckerTexture {
        CheckerTexture {
            inv_scale: 1.0 / scale,
            even: even_texture,
            odd: odd_texture,
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

#[derive(Debug)]
pub struct ImageTexture {
    image: Image,
    pub raw: bool,
}

impl ImageTexture {
    pub fn new(file_name: &str) -> ImageTexture {
        ImageTexture {
            image: Image::new(file_name),
            raw: false,
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
        if self.raw {
            let pixel = self.image.pixel_data_raw(i, j);
            Color::new(pixel[0] as f64, pixel[1] as f64, pixel[2] as f64)
        } else {
            let pixel = self.image.pixel_data(i, j);
            Color::new(pixel.red as f64, pixel.green as f64, pixel.blue as f64)
        }
    }
}

#[derive(Debug)]
pub struct NoiseTexture {
    noise: Perlin,
    scale: f64,
}

impl NoiseTexture {
    pub fn new(scale: f64) -> NoiseTexture {
        NoiseTexture {
            noise: Perlin::default(),
            scale,
        }
    }
}

impl Texture for NoiseTexture {
    fn value(&self, _u: f64, _v: f64, p: &Point3) -> Color {
        Color::new(0.5, 0.5, 0.5)
            * (1.0 + f64::sin(self.scale * p.z() + 10.0 * self.noise.turb(p, 7)))
    }
}
