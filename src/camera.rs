use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use image::{ImageBuffer, RgbImage};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use crate::{
    hit::Hittable,
    material::ScatterType,
    pdf::{HittablePDF, MixturePDF, PDF},
    shapes::environment::Environment,
    texture::SolidColor,
    utils::{
        color::{Color, ToonMap},
        interval::Interval,
        random::Random,
        ray::Ray,
        vec3::{Point3, UnitVec3, Vec3},
    },
};

#[derive(Debug)]
pub struct Camera {
    pub aspect_ratio: f64,
    pub image_width: u32,
    pub samples_per_pixel: usize,
    pub max_depth: u32,
    pub background: Environment,

    pub vertical_fov_in_degrees: f64,
    pub look_from: Point3,
    pub look_at: Point3,
    pub vec_up: Vec3,

    pub defocus_angle_in_degrees: f64,
    pub focus_distance: f64,

    pub toon_map: ToonMap,

    image_height: u32,
    sqrt_spp: u32,
    recip_sqrt_spp: f64,
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
            background: Environment {
                texture: Arc::new(SolidColor::new(Color::BLACK)),
            },
            vertical_fov_in_degrees: 90.0,
            look_from: Point3::new(0.0, 0.0, 0.0),
            look_at: Point3::new(0.0, 0.0, -1.0),
            vec_up: Vec3::new(0.0, 1.0, 0.0),
            defocus_angle_in_degrees: 0.0,
            focus_distance: 10.0,
            toon_map: ToonMap::None,
            image_height: Default::default(),
            sqrt_spp: Default::default(),
            recip_sqrt_spp: Default::default(),
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

    pub fn render(&mut self, world: &dyn Hittable, lights: Option<&dyn Hittable>) -> RgbImage {
        self.initilize();

        let mut img: RgbImage = ImageBuffer::new(self.image_width, self.image_height);

        let progress = if option_env!("CI").unwrap_or_default() == "true" {
            ProgressBar::hidden()
        } else {
            let pb = ProgressBar::new((self.image_height * self.image_width) as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] [{wide_bar}] {pos}/{len} ({eta_precise})")
                    .unwrap(),
            );
            pb
        };

        let counter = Arc::new(AtomicUsize::new(0));
        img.enumerate_pixels_mut()
            .par_bridge()
            .for_each(|(i, j, pixel)| {
                let mut pixel_color = Color::BLACK;
                for s_i in 0..self.sqrt_spp {
                    for s_j in 0..self.sqrt_spp {
                        pixel_color += self.ray_color(
                            &self.get_ray(i, j, s_i, s_j),
                            self.max_depth,
                            world,
                            lights,
                        );
                    }
                }
                let pixel_color = pixel_color * self.pixel_sample_scale;
                *pixel = image::Rgb(pixel_color.to_rgb(&self.toon_map));
                let prev = counter.fetch_add(1, Ordering::SeqCst);
                progress.set_position((prev + 1) as u64);
            });

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

        self.sqrt_spp = f64::sqrt(self.samples_per_pixel as f64) as u32;
        self.pixel_sample_scale = 1.0 / (self.sqrt_spp * self.sqrt_spp) as f64;
        self.recip_sqrt_spp = 1.0 / self.sqrt_spp as f64;

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

    fn get_ray(&self, i: u32, j: u32, s_i: u32, s_j: u32) -> Ray {
        let offset = self.sample_square_stratified(s_i, s_j);
        let pixel_sample = self.pixel00_loc
            + ((i as f64 + offset.x()) * self.pixel_delta_u)
            + ((j as f64 + offset.y()) * self.pixel_delta_v);
        let ray_origin = if self.defocus_angle_in_degrees <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };
        let ray_direction = pixel_sample - ray_origin;
        let ray_time = Random::f64();

        Ray::new_with_time(ray_origin, ray_direction, ray_time)
    }

    fn sample_square_stratified(&self, s_i: u32, s_j: u32) -> Vec3 {
        let px = ((s_i as f64 + Random::f64()) * self.recip_sqrt_spp) - 0.5;
        let py = ((s_j as f64 + Random::f64()) * self.recip_sqrt_spp) - 0.5;

        Vec3::new(px, py, 0.0)
    }

    fn defocus_disk_sample(&self) -> Point3 {
        let p = Vec3::random_in_unit_disk();
        self.center + (p[0] * self.defocus_disk_u) + (p[1] * self.defocus_disk_v)
    }

    fn ray_color(
        &self,
        r: &Ray,
        depth: u32,
        world: &dyn Hittable,
        lights: Option<&dyn Hittable>,
    ) -> Color {
        if depth == 0 {
            return Color::BLACK;
        }

        let Some(rec) = world.hit(r, &Interval::from_range(0.001..f64::INFINITY)) else {
            return self.background.value(r);
        };

        let color_from_emission = rec.mat.emitted(r, &rec);

        let Some(scatter_record) = rec.mat.scatter(r, &rec) else {
            return color_from_emission;
        };

        let color_from_scatter = match scatter_record.scatter_type {
            ScatterType::PDF(pdf_ptr) => {
                let light_ptr =
                    lights.map(|lights_hit| Box::new(HittablePDF::new(lights_hit, rec.p)));
                let mixed_pdf: Box<dyn PDF> = if let Some(ref light) = light_ptr {
                    Box::new(MixturePDF::new(pdf_ptr.as_ref(), light.as_ref()))
                } else {
                    pdf_ptr
                };

                let scattered = Ray::new_with_time(rec.p, mixed_pdf.generate().into_inner(), *r.time());
                let pdf_value = mixed_pdf.value(scattered.direction());
                assert_ne!(pdf_value, 0.0);

                let scattering_pdf = rec.mat.scattering_pdf(r, &rec, &scattered);

                let sample_color = self.ray_color(&scattered, depth - 1, world, lights);
                (scatter_record.attenuation * scattering_pdf * sample_color) / pdf_value
            }
            ScatterType::Ray(skip_pdf_ray) => {
                scatter_record.attenuation * self.ray_color(&skip_pdf_ray, depth - 1, world, lights)
            }
        };

        let ret = color_from_emission + color_from_scatter;
        assert!(!ret.e().iter().any(|x| x.is_nan()));
        ret
    }
}
