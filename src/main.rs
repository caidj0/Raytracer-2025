use std::rc::Rc;

use console::style;
use image::RgbImage;
use raytracer::{
    bvh::BVH,
    camera::Camera,
    hit::{RotateY, Translate},
    hits::Hittables,
    material::{Dielectric, DiffuseLight, Lambertian, Metal},
    shapes::{
        quad::{Quad, build_box},
        sphere::Sphere,
    },
    texture::{CheckerTexture, ImageTexture, NoiseTexture},
    utils::{
        color::Color,
        random::Random,
        vec3::{Point3, Vec3},
    },
    volume::ConstantMedium,
};

fn main() {
    let img = match 9 {
        1 => boncing_spheres(),
        2 => checkered_spheres(),
        3 => earth(),
        4 => perlin_spheres(),
        5 => quads(),
        6 => simple_light(),
        7 => cornell_box(),
        8 => cornell_box_smoke(),
        9 => final_scene(800, 10000, 40),
        _ => final_scene(400, 250, 4),
    };

    let path_string = format!("output/{}/{}.png", "book2", "image23");
    let path = std::path::Path::new(&path_string);
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).expect("Cannot create all the parents");
    println!(
        "Output image as \"{}\"",
        style(path.to_str().unwrap()).yellow()
    );
    img.save(path).expect("Cannot save the image to the file");
}

fn final_scene(image_width: u32, samples_per_pixel: usize, max_depth: u32) -> RgbImage {
    let mut boxes1 = Hittables::default();
    let ground = Rc::new(Lambertian::new(Color::new(0.48, 0.83, 0.53)));

    const BOXES_PER_SIDE: usize = 20;
    for i in 0..BOXES_PER_SIDE {
        for j in 0..BOXES_PER_SIDE {
            let w = 100.0;
            let x0 = -1000.0 + i as f64 * w;
            let z0 = -1000.0 + j as f64 * w;
            let y0 = 0.0;
            let x1 = x0 + w;
            let y1 = Random::random_range(1.0..101.0);
            let z1 = z0 + w;

            boxes1.add(Box::new(build_box(
                Point3::new(x0, y0, z0),
                Point3::new(x1, y1, z1),
                ground.clone(),
            )));
        }
    }

    let mut world = Hittables::default();

    world.add(Box::new(BVH::from_vec(boxes1.objects)));

    let light = Rc::new(DiffuseLight::from_color(Color::new(7.0, 7.0, 7.0)));
    world.add(Box::new(Quad::new(
        Point3::new(123.0, 554.0, 147.0),
        Vec3::new(300.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 265.0),
        light,
    )));

    let center1 = Point3::new(400.0, 400.0, 200.0);
    let center2 = center1 + Vec3::new(30.0, 0.0, 0.0);
    let sphere_material = Rc::new(Lambertian::new(Color::new(0.7, 0.3, 0.1)));
    world.add(Box::new(Sphere::new_with_motion(
        center1,
        center2,
        50.0,
        sphere_material,
    )));

    world.add(Box::new(Sphere::new(
        Point3::new(260.0, 150.0, 45.0),
        50.0,
        Rc::new(Dielectric::new(1.5)),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 150.0, 145.0),
        50.0,
        Rc::new(Metal::new(Color::new(0.8, 0.8, 0.9), 1.0)),
    )));

    let boundary = Sphere::new(
        Point3::new(360.0, 150.0, 145.0),
        70.0,
        Rc::new(Dielectric::new(1.5)),
    );
    world.add(Box::new(boundary.clone()));
    world.add(Box::new(ConstantMedium::new_with_color(
        Box::new(boundary),
        0.2,
        Color::new(0.2, 0.4, 0.9),
    )));
    let boundary = Box::new(Sphere::new(
        Point3::new(0.0, 0.0, 0.0),
        5000.0,
        Rc::new(Dielectric::new(1.5)),
    ));
    world.add(Box::new(ConstantMedium::new_with_color(
        boundary,
        0.0001,
        Color::WHITE,
    )));

    let emat = Rc::new(Lambertian::from_tex(Rc::new(ImageTexture::new(
        "earthmap.jpg",
    ))));
    world.add(Box::new(Sphere::new(
        Point3::new(400.0, 200.0, 400.0),
        100.0,
        emat,
    )));
    let pertext = Rc::new(NoiseTexture::new(0.2));
    world.add(Box::new(Sphere::new(
        Point3::new(220.0, 280.0, 300.0),
        80.0,
        Rc::new(Lambertian::from_tex(pertext)),
    )));

    let mut boxes2 = Hittables::default();
    let white = Rc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    const NS: usize = 1000;
    for _ in 0..NS {
        boxes2.add(Box::new(Sphere::new(
            Point3::random_range(0.0..165.0),
            10.0,
            white.clone(),
        )));
    }

    world.add(Box::new(Translate::new(
        Box::new(RotateY::new(Box::new(BVH::from_vec(boxes2.objects)), 15.0)),
        Vec3::new(-100.0, 270.0, 395.0),
    )));

    let mut camera = Camera::default();

    camera.aspect_ratio = 1.0;
    camera.image_width = image_width;
    camera.samples_per_pixel = samples_per_pixel;
    camera.max_depth = max_depth;
    camera.background = Color::BLACK;

    camera.vertical_fov_in_degrees = 40.0;
    camera.look_from = Point3::new(478.0, 278.0, -600.0);
    camera.look_at = Point3::new(278.0, 278.0, 0.0);
    camera.vec_up = Vec3::new(0.0, 1.0, 0.0);

    camera.defocus_angle_in_degrees = 0.0;

    camera.render(&world)
}

fn cornell_box_smoke() -> RgbImage {
    let mut world = Hittables::default();

    let red = Rc::new(Lambertian::new(Color::new(0.65, 0.05, 0.05)));
    let white = Rc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    let green = Rc::new(Lambertian::new(Color::new(0.12, 0.45, 0.15)));
    let light = Rc::new(DiffuseLight::from_color(Color::new(7.0, 7.0, 7.0)));

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
        Point3::new(113.0, 554.0, 127.0),
        Vec3::new(330.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 305.0),
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
        white.clone(),
    )));

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
    let box2 = Box::new(Translate::new(box2, Vec3::new(130.0, 0.0, 65.0)));

    world.add(Box::new(ConstantMedium::new_with_color(
        box1,
        0.01,
        Color::BLACK,
    )));
    world.add(Box::new(ConstantMedium::new_with_color(
        box2,
        0.01,
        Color::WHITE,
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
        white.clone(),
    )));

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
    let box2 = Box::new(Translate::new(box2, Vec3::new(130.0, 0.0, 65.0)));

    world.add(box1);
    world.add(box2);

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
