use image::{ImageBuffer, RgbImage};
use indicatif::ProgressBar;

use crate::{
    hit::Hittable,
    hits::Hittables,
    utils::{
        color::Color,
        ray::Ray,
        vec3::{Point3, Vec3},
    },
};

#[derive(Debug, Default)]
pub struct Camera {
    pub aspect_ratio: f64,
    pub image_width: u32,
    image_height: u32,
    center: Point3,
    pixel00_loc: Point3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
}

impl Camera {
    pub fn new(aspect_ratio: f64, image_width: u32) -> Camera {
        Camera {
            aspect_ratio,
            image_width,
            ..Default::default()
        }
    }

    pub fn render(&mut self, world: &Hittables) -> RgbImage {
        self.initilize();

        let mut img: RgbImage = ImageBuffer::new(self.image_width, self.image_height);
        let progress = if option_env!("CI").unwrap_or_default() == "true" {
            ProgressBar::hidden()
        } else {
            ProgressBar::new((self.image_height * self.image_width) as u64)
        };

        for j in 0..self.image_height {
            for i in 0..self.image_width {
                let pixel_center = self.pixel00_loc
                    + (i as f64 * self.pixel_delta_u)
                    + (j as f64 * self.pixel_delta_v);
                let ray_direction = pixel_center - self.center;
                let r = Ray::new(self.center, ray_direction);

                let pixel_color = ray_color(&r, world);
                let pixel = img.get_pixel_mut(i, j);
                *pixel = image::Rgb(pixel_color.to_rgb());
            }
            progress.inc(1);
        }
        progress.finish();

        img
    }

    fn initilize(&mut self) {
        self.image_height = (self.image_width as f64 / self.aspect_ratio) as u32;
        self.image_height = if self.image_height < 1 {
            1
        } else {
            self.image_height
        };

        self.center = Point3::new(0.0, 0.0, 0.0);

        let focal_length: f64 = 1.0;
        let viewport_height: f64 = 2.0;
        let viewport_width = viewport_height * (self.image_width as f64 / self.image_height as f64);

        let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
        let viewport_v = Vec3::new(0.0, -viewport_height, 0.0);

        self.pixel_delta_u = viewport_u / self.image_width as f64;
        self.pixel_delta_v = viewport_v / self.image_height as f64;

        let viewport_upper_left =
            self.center - Vec3::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
        self.pixel00_loc = viewport_upper_left + 0.5 * (self.pixel_delta_u + self.pixel_delta_v);
    }
}

fn ray_color(r: &Ray, world: &dyn Hittable) -> Color {
    if let Some(t) = world.hit(r, &(0.0..1000.0)) {
        return 0.5 * (t.normal + Vec3::new(1.0, 1.0, 1.0));
    }

    let unit_vec = r.direction().unit_vector();
    let a = 0.5 * (unit_vec.y() + 1.0);

    (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
}
