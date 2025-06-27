use std::rc::Rc;

use console::style;
use image::RgbImage;
use raytracer::{
    bvh::BVH,
    camera::Camera,
    hits::Hittables,
    material::{Dielectric, DiffuseLight, Lambertian, Metal},
    shapes::{quad::Quad, sphere::Sphere},
    texture::{CheckerTexture, ImageTexture, NoiseTexture},
    utils::{
        color::Color,
        random::Random,
        vec3::{Point3, Vec3},
    },
};

fn main() {
    let img = match 6 {
        2 => checkered_spheres(),
        3 => earth(),
        4 => perlin_spheres(),
        5 => quads(),
        6 => simple_light(),
        7 => cornell_box(),
        _ => boncing_spheres(),
    };

    let path_string = format!("output/{}/{}.png", "book2", "image18");
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
        light,
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
        white,
    )));

    let mut camera = Camera::default();
    camera.aspect_ratio = 1.0;
    camera.image_width = 600;
    camera.samples_per_pixel = 200;
    camera.max_depth = 50;
    camera.background = Color::BLACK;

    camera.vertical_fov_in_degrees = 40.0;
    camera.look_from = Point3::new(278.0, 278.0, -800.0);
    camera.look_at = Point3::new(278.0, 278.0, 0.0);
    camera.vec_up = Vec3::new(0.0, 1.0, 0.0);

    camera.defocus_angle_in_degrees = 0.0;

    camera.render(&world)
}

fn simple_light() -> RgbImage {
    let mut world = Hittables::default();

    let per_tex = Rc::new(NoiseTexture::new(4.0));
    let per_mat = Rc::new(Lambertian::from_tex(per_tex));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        per_mat.clone(),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 2.0, 0.0),
        2.0,
        per_mat,
    )));

    let difflight = Rc::new(DiffuseLight::from_color(Color::new(4.0, 4.0, 4.0)));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 7.0, 0.0),
        2.0,
        difflight.clone(),
    )));
    world.add(Box::new(Quad::new(
        Point3::new(3.0, 1.0, -2.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
        difflight,
    )));

    let mut camera = Camera::default();
    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = 400;
    camera.samples_per_pixel = 100;
    camera.max_depth = 50;
    camera.background = Color::BLACK;

    camera.vertical_fov_in_degrees = 20.0;
    camera.look_from = Point3::new(26.0, 3.0, 6.0);
    camera.look_at = Point3::new(0.0, 2.0, 0.0);
    camera.vec_up = Vec3::new(0.0, 1.0, 0.0);

    camera.defocus_angle_in_degrees = 0.0;

    camera.render(&world)
}

fn quads() -> RgbImage {
    let mut world = Hittables::default();

    let left_red = Rc::new(Lambertian::new(Color::new(1.0, 0.2, 0.2)));
    let back_green = Rc::new(Lambertian::new(Color::new(0.2, 1.0, 0.2)));
    let right_blue = Rc::new(Lambertian::new(Color::new(0.2, 0.2, 1.0)));
    let upper_orange = Rc::new(Lambertian::new(Color::new(1.0, 0.5, 0.0)));
    let lower_teal = Rc::new(Lambertian::new(Color::new(0.2, 0.8, 0.8)));

    world.add(Box::new(Quad::new(
        Point3::new(-3.0, -2.0, 5.0),
        Vec3::new(0.0, 0.0, -4.0),
        Vec3::new(0.0, 4.0, 0.0),
        left_red,
    )));
    world.add(Box::new(Quad::new(
        Point3::new(-2.0, -2.0, 0.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 4.0, 0.0),
        back_green,
    )));
    world.add(Box::new(Quad::new(
        Point3::new(3.0, -2.0, 1.0),
        Vec3::new(0.0, 0.0, 4.0),
        Vec3::new(0.0, 4.0, 0.0),
        right_blue,
    )));
    world.add(Box::new(Quad::new(
        Point3::new(-2.0, 3.0, 1.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 4.0),
        upper_orange,
    )));
    world.add(Box::new(Quad::new(
        Point3::new(-2.0, -3.0, 5.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -4.0),
        lower_teal,
    )));

    let mut camera = Camera::default();
    camera.aspect_ratio = 1.0;
    camera.image_width = 400;
    camera.samples_per_pixel = 100;
    camera.max_depth = 50;
    camera.background = Color::new(0.7, 0.8, 1.0);

    camera.vertical_fov_in_degrees = 80.0;
    camera.look_from = Point3::new(0.0, 0.0, 9.0);
    camera.look_at = Point3::new(0.0, 0.0, 0.0);
    camera.vec_up = Vec3::new(0.0, 1.0, 0.0);

    camera.defocus_angle_in_degrees = 0.0;

    camera.render(&world)
}

fn perlin_spheres() -> RgbImage {
    let mut world = Hittables::default();

    let perlin_tex = Rc::new(NoiseTexture::new(4.0));

    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        Rc::new(Lambertian::from_tex(perlin_tex.clone())),
    )));

    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 2.0, 0.0),
        2.0,
        Rc::new(Lambertian::from_tex(perlin_tex)),
    )));

    let mut camera = Camera::default();
    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = 400;
    camera.samples_per_pixel = 100;
    camera.max_depth = 50;
    camera.background = Color::new(0.7, 0.8, 1.0);

    camera.vertical_fov_in_degrees = 20.0;
    camera.look_from = Point3::new(13.0, 2.0, 3.0);
    camera.look_at = Point3::new(0.0, 0.0, 0.0);
    camera.vec_up = Vec3::new(0.0, 1.0, 0.0);

    camera.defocus_angle_in_degrees = 0.0;

    camera.render(&world)
}

fn earth() -> RgbImage {
    let earth_texture = Rc::new(ImageTexture::new("earthmap.jpg"));
    let earth_surface = Rc::new(Lambertian::from_tex(earth_texture));
    let globe = Box::new(Sphere::new(Vec3::ZERO, 2.0, earth_surface));

    let mut camera = Camera::default();
    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = 400;
    camera.samples_per_pixel = 100;
    camera.max_depth = 50;
    camera.background = Color::new(0.7, 0.8, 1.0);

    camera.vertical_fov_in_degrees = 20.0;
    camera.look_from = Point3::new(0.0, 0.0, 12.0);
    camera.look_at = Point3::new(0.0, 0.0, 0.0);
    camera.vec_up = Vec3::new(0.0, 1.0, 0.0);

    camera.defocus_angle_in_degrees = 0.0;

    camera.render(&Hittables::new(globe))
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
    camera.background = Color::new(0.7, 0.8, 1.0);

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
    camera.background = Color::new(0.7, 0.8, 1.0);

    camera.vertical_fov_in_degrees = 20.0;
    camera.look_from = Point3::new(13.0, 2.0, 3.0);
    camera.look_at = Point3::new(0.0, 0.0, 0.0);
    camera.vec_up = Vec3::new(0.0, 1.0, 0.0);

    camera.defocus_angle_in_degrees = 0.6;
    camera.focus_distance = 10.0;

    camera.render(&world)
}
