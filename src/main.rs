use console::style;
use raytracer::{camera::Camera, hits::Hittables, shapes::sphere::Sphere, utils::vec3::Point3};

fn main() {
    let path = std::path::Path::new("output/book1/image2.png");
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).expect("Cannot create all the parents");

    let aspect_ratio: f64 = 16.0 / 9.0;
    let image_width: u32 = 400;
    let mut camera = Camera::new(aspect_ratio, image_width);

    let mut world: Hittables = Default::default();
    world.add(Box::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5)));
    world.add(Box::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0)));

    let img = camera.render(&world);

    println!(
        "Output image as \"{}\"",
        style(path.to_str().unwrap()).yellow()
    );
    img.save(path).expect("Cannot save the image to the file");
}
