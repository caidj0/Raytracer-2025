use std::{
    env::{self, current_dir},
    path::{Path, PathBuf},
};

use image::{ImageFormat, ImageReader, Rgba32FImage};
use palette::Srgba;

#[derive(Debug)]
pub struct Image {
    img: Option<(Rgba32FImage, ImageFormat)>,
    raw: bool,
}

impl Image {
    pub const EMPTY: Image = Image {
        img: None,
        raw: false,
    };

    pub fn new(image_filename: &str, raw: bool) -> Image {
        if let Ok(specified_dir) = env::var("RTW_IMAGES") {
            let mut path = PathBuf::new();
            path.push(specified_dir);
            path.push(image_filename);
            return Image {
                img: Image::load(path),
                raw,
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
            raw,
        }
    }

    fn load<P>(path: P) -> Option<(Rgba32FImage, ImageFormat)>
    where
        P: AsRef<Path>,
    {
        let ir = ImageReader::open(path).ok()?;
        let fmt = ir.format()?;
        Some((ir.decode().ok()?.into_rgba32f(), fmt))
    }

    pub fn width(&self) -> u32 {
        self.img.as_ref().map_or(0, |x| x.0.width())
    }

    pub fn height(&self) -> u32 {
        self.img.as_ref().map_or(0, |x| x.0.height())
    }

    pub fn pixel_data(&self, x: u32, y: u32) -> [f32; 4] {
        let Some((img, fmt)) = &self.img else {
            return [1.0, 0.0, 1.0, 1.0];
        };

        let x = x.clamp(0, self.width() - 1);
        let y = y.clamp(0, self.height() - 1);

        let raw_pixel = img.get_pixel(x, y).0;
        if self.raw {
            return raw_pixel;
        }

        match fmt {
            ImageFormat::Hdr => raw_pixel,
            ImageFormat::OpenExr => raw_pixel,
            ImageFormat::Avif => raw_pixel,
            _ => Srgba::from(raw_pixel).into_linear().into(),
        }
    }
}
