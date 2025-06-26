use std::rc::Rc;

use console::style;
use image::RgbImage;
use raytracer::{
    bvh::BVH,
    camera::Camera,
    hits::Hittables,
    material::{Dielectric, Lambertian, Metal},
    shapes::sphere::Sphere,
    texture::CheckerTexture,
    utils::{
        color::Color,
        random::Random,
        vec3::{Point3, Vec3},
    },
};

fn main() {
    let img = match 2 {
        2 => checkered_spheres(),
        _ => boncing_spheres(),
    };

    let path_string = format!("output/{}/{}.png", "book2", "image3");
    let path = std::path::Path::new(&path_string);
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).expect("Cannot create all the parents");
    println!(
        "Output image as \"{}\"",
        style(path.to_str().unwrap()).yellow()
    );
    img.save(path).expect("Cannot save the image to the file");
}

fn checkered_spheres() -> RgbImage {
    let mut world = Hittables::default();

    let checker_tex = Rc::new(CheckerTexture::from_colors(
        0.32,
        Color::new(0.2, 0.3, 0.1),
        Color::new(0.9, 0.9, 0.9),
    ));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -10.0, 0.0),
        10.0,
        Rc::new(Lambertian::from_tex(checker_tex.clone())),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 10.0, 0.0),
        10.0,
        Rc::new(Lambertian::from_tex(checker_tex)),
    )));

    let mut camera = Camera::default();
    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = 400;
    camera.samples_per_pixel = 100;
    camera.max_depth = 50;

    camera.vertical_fov_in_degrees = 20.0;
    camera.look_from = Point3::new(13.0, 2.0, 3.0);
    camera.look_at = Point3::new(0.0, 0.0, 0.0);
    camera.vec_up = Vec3::new(0.0, 1.0, 0.0);

    camera.defocus_angle_in_degrees = 0.0;

    camera.render(&world)
}

fn boncing_spheres() -> RgbImage {
    let mut world: Hittables = Default::default();

    let checker_material = Rc::new(Lambertian::from_tex(Rc::new(CheckerTexture::from_colors(
        0.32,
        Color::new(0.2, 0.3, 0.1),
        Color::new(0.9, 0.9, 0.9),
    ))));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        checker_material,
    )));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat: f64 = Random::f64();
            let center = Point3::new(
                a as f64 + 0.9 * Random::f64(),
                0.2,
                b as f64 + 0.9 * Random::f64(),
            );

            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                match choose_mat {
                    ..0.8 => {
                        let albedo = Color::random() * Color::random();
                        let center2 = center + Vec3::new(0.0, Random::random_range(0.0..0.5), 0.0);
                        let shpere_material = Rc::new(Lambertian::new(albedo));
                        world.add(Box::new(Sphere::new_with_motion(
                            center,
                            center2,
                            0.2,
                            shpere_material,
                        )));
                    }
                    ..0.95 => {
                        let albedo = Color::random_range(0.5..1.0);
                        let fuzz: f64 = Random::random_range(0.0..0.5);
                        let shpere_material = Rc::new(Metal::new(albedo, fuzz));
                        world.add(Box::new(Sphere::new(center, 0.2, shpere_material)));
                    }
                    _ => {
                        let shpere_material = Rc::new(Dielectric::new(1.5));
                        world.add(Box::new(Sphere::new(center, 0.2, shpere_material)));
                    }
                };
            }
        }
    }

    let material1 = Rc::new(Dielectric::new(1.5));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        material1,
    )));

    let material2 = Rc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
    world.add(Box::new(Sphere::new(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        material2,
    )));

    let material3 = Rc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));
    world.add(Box::new(Sphere::new(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        material3,
    )));

    let bvh = BVH::from_vec(world.objects);
    let world = Hittables::new(Box::new(bvh));

    let mut camera = Camera::default();
    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = 400;
    camera.samples_per_pixel = 100;
    camera.max_depth = 50;

    camera.vertical_fov_in_degrees = 20.0;
    camera.look_from = Point3::new(13.0, 2.0, 3.0);
    camera.look_at = Point3::new(0.0, 0.0, 0.0);
    camera.vec_up = Vec3::new(0.0, 1.0, 0.0);

    camera.defocus_angle_in_degrees = 0.6;
    camera.focus_distance = 10.0;

    camera.render(&world)
}
