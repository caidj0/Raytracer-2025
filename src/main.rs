use std::f64::consts::PI;

use console::style;
use raytracer::{
    camera::Camera,
    hits::Hittables,
    material::Lambertian,
    shapes::sphere::Sphere,
    utils::{color::Color, vec3::Point3},
};

fn main() {
    let path_string = format!("output/book1/{}.png", "image19");
    let path = std::path::Path::new(&path_string);
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).expect("Cannot create all the parents");

    let aspect_ratio: f64 = 16.0 / 9.0;
    let image_width: u32 = 400;
    let mut camera = Camera::default();
    camera.aspect_ratio = aspect_ratio;
    camera.image_width = image_width;
    camera.samples_per_pixel = 100;
    camera.max_depth = 50;
    camera.vertical_fov_in_degree = 90.0;

    let mut world: Hittables = Default::default();

    let r = f64::cos(PI / 4.0);
    let material_left = Box::new(Lambertian::new(&Color::BLUE));
    let material_right = Box::new(Lambertian::new(&Color::RED));

    world.add(Box::new(Sphere::new(
        Point3::new(-r, 0.0, -1.0),
        r,
        material_left,
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(r, 0.0, -1.0),
        r,
        material_right,
    )));

    let img = camera.render(&world);

    println!(
        "Output image as \"{}\"",
        style(path.to_str().unwrap()).yellow()
    );
    img.save(path).expect("Cannot save the image to the file");
}
