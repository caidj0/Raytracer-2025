use std::{
    env::{self, current_dir},
    path::{Path, PathBuf},
};

use image::{ImageReader, Rgb32FImage};
use palette::{LinSrgb, Srgb};

#[derive(Debug)]
pub struct Image {
    img: Option<Rgb32FImage>,
}

impl Image {
    pub const EMPTY: Image = Image { img: None };

    pub fn new(image_filename: &str) -> Image {
        if let Ok(specified_dir) = env::var("RTW_IMAGES") {
            let mut path = PathBuf::new();
            path.push(specified_dir);
            path.push(image_filename);
            return Image {
                img: Image::load(path),
            };
        }

        let Ok(pathbuf) = current_dir() else {
            return Image::EMPTY;
        };

        let mut path = pathbuf;
        path.push("assets");
        path.push(image_filename);

        Image {
            img: Image::load(path),
        }
    }

    fn load<P>(path: P) -> Option<Rgb32FImage>
    where
        P: AsRef<Path>,
    {
        let ir = ImageReader::open(path).ok()?;
        Some(ir.decode().ok()?.into_rgb32f())
    }

    pub fn width(&self) -> u32 {
        self.img.as_ref().map_or(0, |x| x.width())
    }

    pub fn height(&self) -> u32 {
        self.img.as_ref().map_or(0, |x| x.height())
    }

    pub fn pixel_data(&self, x: u32, y: u32) -> LinSrgb {
        if self.img.is_none() {
            return LinSrgb::from([1.0, 0.0, 1.0]);
        }

        let x = x.clamp(0, self.width() - 1);
        let y = y.clamp(0, self.height() - 1);
        Srgb::from(self.img.as_ref().unwrap().get_pixel(x, y).0).into_linear()
    }

    pub fn pixel_data_raw(&self, x: u32, y: u32) -> [f32; 3] {
        if self.img.is_none() {
            return [1.0, 0.0, 1.0];
        }

        let x = x.clamp(0, self.width() - 1);
        let y = y.clamp(0, self.height() - 1);
        self.img.as_ref().unwrap().get_pixel(x, y).0
    }
}
