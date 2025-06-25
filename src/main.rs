use console::style;
use raytracer::{camera::Camera, hits::Hittables, material::{Lambertian, Metal}, shapes::sphere::Sphere, utils::{color::Color, vec3::Point3}};

fn main() {
    let path_string = format!("output/book1/image13.png");
    let path = std::path::Path::new(&path_string);
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).expect("Cannot create all the parents");

    let aspect_ratio: f64 = 16.0 / 9.0;
    let image_width: u32 = 400;
    let mut camera = Camera::new(aspect_ratio, image_width);

    camera.samples_per_pixel = 100;
    camera.max_depth = 50;


    let mut world: Hittables = Default::default();

    let material_ground = Box::new(Lambertian::new(&Color::new(0.8, 0.8, 0.0)));
    let material_center = Box::new(Lambertian::new(&Color::new(0.1, 0.2, 0.5)));
    let material_left = Box::new(Metal::new(&Color::new(0.8, 0.8, 0.8)));
    let material_right = Box::new(Metal::new(&Color::new(0.8, 0.6, 0.2)));

    world.add(Box::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, material_ground)));
    world.add(Box::new(Sphere::new(Point3::new(0.0, 0.0, -1.2),0.5, material_center)));
    world.add(Box::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.5, material_left)));
    world.add(Box::new(Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.5, material_right)));


    let img = camera.render(&world);

    println!(
        "Output image as \"{}\"",
        style(path.to_str().unwrap()).yellow()
    );
    img.save(path).expect("Cannot save the image to the file");
}
