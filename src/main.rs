use console::style;
use image::{ImageBuffer, RgbImage};
use indicatif::ProgressBar;
use raytracer::{
    hit::Hittable,
    hits::Hittables,
    shapes::sphere::Sphere,
    utils::{
        color::Color,
        ray::Ray,
        vec3::{Point3, Vec3},
    },
};

fn ray_color(r: &Ray, world: &dyn Hittable) -> Color {
    if let Some(t) = world.hit(r, 0.0, 1000.0) {
        return 0.5 * (t.normal + Vec3::new(1.0, 1.0, 1.0));
    }

    let unit_vec = r.direction().unit_vector();
    let a = 0.5 * (unit_vec.y() + 1.0);

    (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
}

fn main() {
    let path = std::path::Path::new("output/book1/image2.png");
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).expect("Cannot create all the parents");

    let aspect_ratio: f64 = 16.0 / 9.0;
    let image_width: u32 = 400;

    let image_height = (image_width as f64 / aspect_ratio) as u32;
    let image_height = if image_height < 1 { 1 } else { image_height };

    let mut world: Hittables = Default::default();
    world.add(Box::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5)));
    world.add(Box::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0)));

    let focal_length: f64 = 1.0;
    let viewport_height: f64 = 2.0;
    let viewport_width = viewport_height * (image_width as f64 / image_height as f64);
    let camera_center = Point3::new(0.0, 0.0, 0.0);

    let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
    let viewport_v = Vec3::new(0.0, -viewport_height, 0.0);

    let pixel_delta_u = viewport_u / image_width as f64;
    let pixel_delta_v = viewport_v / image_height as f64;

    let viewport_upper_left =
        camera_center - Vec3::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
    let pixell00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

    let mut img: RgbImage = ImageBuffer::new(image_width, image_height);
    let progress = if option_env!("CI").unwrap_or_default() == "true" {
        ProgressBar::hidden()
    } else {
        ProgressBar::new((image_height * image_width) as u64)
    };

    for j in 0..image_height {
        for i in 0..image_width {
            let pixel_center =
                pixell00_loc + (i as f64 * pixel_delta_u) + (j as f64 * pixel_delta_v);
            let ray_direction = pixel_center - camera_center;
            let r = Ray::new(camera_center, ray_direction);

            let pixel_color = ray_color(&r, &world);
            let pixel = img.get_pixel_mut(i, j);
            *pixel = image::Rgb(pixel_color.to_rgb());
        }
        progress.inc(1);
    }
    progress.finish();

    println!(
        "Output image as \"{}\"",
        style(path.to_str().unwrap()).yellow()
    );
    img.save(path).expect("Cannot save the image to the file");
}
