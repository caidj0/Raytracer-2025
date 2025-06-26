use image::{ImageBuffer, RgbImage};
use indicatif::ProgressBar;

use crate::{
    hit::Hittable,
    hits::Hittables,
    utils::{
        color::Color,
        ray::Ray,
        vec3::{Point3, UnitVec3, Vec3},
    },
};

#[derive(Debug)]
pub struct Camera {
    pub aspect_ratio: f64,
    pub image_width: u32,
    pub samples_per_pixel: u32,
    pub max_depth: u32,

    pub vertical_fov_in_degrees: f64,
    pub look_from: Point3,
    pub look_at: Point3,
    pub vec_up: Vec3,

    pub defocus_angle_in_degrees: f64,
    pub focus_distance: f64,

    image_height: u32,
    center: Point3,
    pixel00_loc: Point3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    pixel_sample_scale: f64,
    camera_axis: (UnitVec3, UnitVec3, UnitVec3),
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            aspect_ratio: 1.0,
            image_width: 100,
            samples_per_pixel: 10,
            max_depth: 10,
            vertical_fov_in_degrees: 90.0,
            look_from: Point3::new(0.0, 0.0, 0.0),
            look_at: Point3::new(0.0, 0.0, -1.0),
            vec_up: Vec3::new(0.0, 1.0, 0.0),
            defocus_angle_in_degrees: 0.0,
            focus_distance: 10.0,
            image_height: Default::default(),
            center: Default::default(),
            pixel00_loc: Default::default(),
            pixel_delta_u: Default::default(),
            pixel_delta_v: Default::default(),
            pixel_sample_scale: Default::default(),
            camera_axis: Default::default(),
            defocus_disk_u: Default::default(),
            defocus_disk_v: Default::default(),
        }
    }
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
                let pixel_color_sum: Vec3 = (0..self.samples_per_pixel)
                    .map(|_| ray_color(&self.get_ray(i, j), self.max_depth, world))
                    .sum();
                let pixel_color = pixel_color_sum * self.pixel_sample_scale;
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

        self.pixel_sample_scale = 1.0 / self.samples_per_pixel as f64;

        self.center = self.look_from;

        let theta = self.vertical_fov_in_degrees.to_radians();
        let h = f64::tan(theta / 2.0);
        let viewport_height: f64 = 2.0 * h * self.focus_distance;
        let viewport_width = viewport_height * (self.image_width as f64 / self.image_height as f64);

        self.camera_axis.2 = UnitVec3::from_vec3(self.look_from - self.look_at)
            .expect("Camera axis w should be normalizable!");
        self.camera_axis.0 = UnitVec3::from_vec3(self.vec_up.cross(&self.camera_axis.2))
            .expect("Camera axis u should be normalizable!");
        self.camera_axis.1 = UnitVec3::from_vec3_raw(self.camera_axis.2.cross(&self.camera_axis.0));

        let viewport_u = viewport_width * self.camera_axis.0.as_inner();
        let viewport_v = viewport_height * (-self.camera_axis.1.as_inner());

        self.pixel_delta_u = viewport_u / self.image_width as f64;
        self.pixel_delta_v = viewport_v / self.image_height as f64;

        let viewport_upper_left = self.center
            - self.focus_distance * self.camera_axis.2.as_inner()
            - viewport_u / 2.0
            - viewport_v / 2.0;
        self.pixel00_loc = viewport_upper_left + 0.5 * (self.pixel_delta_u + self.pixel_delta_v);

        let defocus_radius =
            self.focus_distance * f64::tan((self.defocus_angle_in_degrees / 2.0).to_radians());
        self.defocus_disk_u = self.camera_axis.0.as_inner() * defocus_radius;
        self.defocus_disk_v = self.camera_axis.1.as_inner() * defocus_radius;
    }

    fn get_ray(&self, i: u32, j: u32) -> Ray {
        let offset = Vec3::new(
            rand::random::<f64>() - 0.5,
            rand::random::<f64>() - 0.5,
            0.0,
        );
        let pixel_sample = self.pixel00_loc
            + ((i as f64 + offset.x()) * self.pixel_delta_u)
            + ((j as f64 + offset.y()) * self.pixel_delta_v);
        let ray_origin = if self.defocus_angle_in_degrees <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };
        let ray_direction = pixel_sample - ray_origin;

        Ray::new(ray_origin, ray_direction)
    }

    fn defocus_disk_sample(&self) -> Point3 {
        let p = Vec3::random_in_unit_disk();
        self.center + (p[0] * self.defocus_disk_u) + (p[1] * self.defocus_disk_v)
    }
}

fn ray_color(r: &Ray, depth: u32, world: &dyn Hittable) -> Color {
    if depth == 0 {
        return Color::BLACK;
    }
    if let Some(rec) = world.hit(r, &(0.001..f64::INFINITY)) {
        if let Some((attenuation, scatter)) = rec.mat.scatter(r, &rec) {
            return attenuation * ray_color(&scatter, depth - 1, world);
        } else {
            return Color::BLACK;
        }
    }

    let unit_vec = UnitVec3::from_vec3(*r.direction()).unwrap();
    let a = 0.5 * (unit_vec.y() + 1.0);

    (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
}
