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
pub enum ImageInterpMethod {
    None,
    Linear,
}

#[derive(Debug)]
pub struct ImageTexture {
    image: Image,
    interp: ImageInterpMethod,
}

impl ImageTexture {
    pub fn new(file_name: &str) -> ImageTexture {
        ImageTexture {
            image: Image::new(file_name, false),
            interp: ImageInterpMethod::None,
        }
    }

    pub fn new_raw_image(file_name: &str) -> ImageTexture {
        ImageTexture {
            image: Image::new(file_name, true),
            interp: ImageInterpMethod::Linear,
        }
    }

    pub fn alpha(&self, u: f64, v: f64, _p: &Point3) -> f64 {
        if self.image.height() == 0 {
            return 1.0;
        }

        let pixel = self.get_pixel(u, v);
        pixel[3] as f64
    }

    fn get_none_interp_pixel(&self, u: f64, v: f64) -> [f32; 4] {
        let u = abs_fract(u);
        let v = 1.0 - abs_fract(v);

        let i = (u * self.image.width() as f64) as u32;
        let j = (v * self.image.height() as f64) as u32;

        self.image.pixel_data(i, j)
    }

    fn get_linear_interp_pixel(&self, u: f64, v: f64) -> [f32; 4] {
        let u = abs_fract(u);
        let v = 1.0 - abs_fract(v);

        let width = self.image.width() as f64;
        let height = self.image.height() as f64;

        let x = u * width - 0.5;
        let y = v * height - 0.5;

        let x0 = x.floor().max(0.0) as u32;
        let y0 = y.floor().max(0.0) as u32;
        let x1 = (x0 + 1).min(self.image.width() - 1);
        let y1 = (y0 + 1).min(self.image.height() - 1);

        let dx = x - x0 as f64;
        let dy = y - y0 as f64;

        let p00 = self.image.pixel_data(x0, y0);
        let p10 = self.image.pixel_data(x1, y0);
        let p01 = self.image.pixel_data(x0, y1);
        let p11 = self.image.pixel_data(x1, y1);

        let mut result: [f32; 4] = [0.0; 4];
        for i in 0..4 {
            let v0 = p00[i] * (1.0 - dx as f32) + p10[i] * (dx as f32);
            let v1 = p01[i] * (1.0 - dx as f32) + p11[i] * (dx as f32);
            result[i] = v0 * (1.0 - dy as f32) + v1 * (dy as f32);
        }
        result
    }

    fn get_pixel(&self, u: f64, v: f64) -> [f32; 4] {
        match self.interp {
            ImageInterpMethod::Linear => self.get_linear_interp_pixel(u, v),
            _ => self.get_none_interp_pixel(u, v),
        }
    }
}

fn abs_fract(x: f64) -> f64 {
    x - x.floor()
}

impl Texture for ImageTexture {
    fn value(&self, u: f64, v: f64, _p: &Point3) -> Color {
        if self.image.height() == 0 {
            return Color::new(0.0, 1.0, 1.0);
        }

        let pixel = self.get_pixel(u, v);
        Color::new(pixel[0] as f64, pixel[1] as f64, pixel[2] as f64)
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
