use std::rc::Rc;

use console::style;
use image::RgbImage;
use raytracer::{
    camera::Camera,
    hit::{RotateY, Translate},
    hits::Hittables,
    material::{Dielectric, DiffuseLight, Lambertian, Metal},
    shapes::{
        quad::{Quad, build_box},
        sphere::Sphere,
    },
    utils::{
        color::Color,
        vec3::{Point3, Vec3},
    },
};

fn main() {
    let img = cornell_box();
    let path_string = format!("output/{}/{}.png", "book3", "image14");
    let path = std::path::Path::new(&path_string);
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).expect("Cannot create all the parents");
    println!(
        "Output image as \"{}\"",
        style(path.to_str().unwrap()).yellow()
    );
    img.save(path).expect("Cannot save the image to the file");
}

fn cornell_box() -> RgbImage {
    let mut world = Hittables::default();

    let red = Rc::new(Lambertian::new(Color::new(0.65, 0.05, 0.05)));
    let white = Rc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    let green = Rc::new(Lambertian::new(Color::new(0.12, 0.45, 0.15)));
    let light = Rc::new(DiffuseLight::from_color(Color::new(15.0, 15.0, 15.0)));

    world.add(Box::new(Quad::new(
        Point3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        green,
    )));
    world.add(Box::new(Quad::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        red,
    )));
    world.add(Box::new(Quad::new(
        Point3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        light.clone(),
    )));
    world.add(Box::new(Quad::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        white.clone(),
    )));
    world.add(Box::new(Quad::new(
        Point3::new(555.0, 555.0, 555.0),
        Vec3::new(-555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -555.0),
        white.clone(),
    )));
    world.add(Box::new(Quad::new(
        Point3::new(0.0, 0.0, 555.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        white.clone(),
    )));

    let _aluminum = Rc::new(Metal::new(Color::new(0.8, 0.85, 0.88), 0.0));
    let box1 = Box::new(build_box(
        Point3::ZERO,
        Point3::new(165.0, 330.0, 165.0),
        white.clone(),
    ));
    let box1 = Box::new(RotateY::new(box1, 15.0));
    let box1 = Box::new(Translate::new(box1, Vec3::new(265.0, 0.0, 295.0)));

    let box2 = Box::new(build_box(
        Point3::ZERO,
        Point3::new(165.0, 165.0, 165.0),
        white,
    ));
    let box2 = Box::new(RotateY::new(box2, -18.0));
    let _box2 = Box::new(Translate::new(box2, Vec3::new(130.0, 0.0, 65.0)));

    let glass = Rc::new(Dielectric::new(1.5));

    world.add(box1);
    // world.add(box2);
    world.add(Box::new(Sphere::new(
        Point3::new(190.0, 90.0, 190.0),
        90.0,
        glass.clone(),
    )));

    let mut lights = Hittables::default();
    lights.add(Box::new(Quad::new(
        Point3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        light,
    )));
    lights.add(Box::new(Sphere::new(
        Point3::new(190.0, 90.0, 190.0),
        90.0,
        glass,
    )));

    let mut camera = Camera::default();
    camera.aspect_ratio = 1.0;
    camera.image_width = 600;
    camera.samples_per_pixel = 1000;
    camera.max_depth = 50;
    camera.background = Color::BLACK;

    camera.vertical_fov_in_degrees = 40.0;
    camera.look_from = Point3::new(278.0, 278.0, -800.0);
    camera.look_at = Point3::new(278.0, 278.0, 0.0);
    camera.vec_up = Vec3::new(0.0, 1.0, 0.0);

    camera.defocus_angle_in_degrees = 0.0;

    camera.render(&world, &lights)
}
