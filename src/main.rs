use std::sync::Arc;

use console::style;
use image::RgbImage;
use raytracer::{
    bvh::BVH,
    camera::Camera,
    hits::Hittables,
    material::{Dielectric, DiffuseLight, EmptyMaterial, Lambertian, Metal},
    shapes::{
        Transform,
        obj::Wavefont,
        quad::{Quad, build_box},
        sphere::Sphere,
        triangle::Triangle,
    },
    texture::{ImageTexture, NoiseTexture, SolidColor},
    utils::{
        color::Color,
        quaternion::Quaternion,
        random::Random,
        vec3::{Point3, Vec3},
    },
    volume::ConstantMedium,
};

fn main() {
    let img = match 3 {
        0 => cornell_box(),
        1 => final_scene(400, 250, 4),
        2 => final_scene(800, 5000, 40),
        _ => obj_scene(),
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

fn obj_scene() -> RgbImage {
    let obj = Wavefont::new("uv_test_sphere.obj").unwrap();
    let light_tex = Arc::new(SolidColor::new(Color::new(3.0, 3.0, 3.0)));
    let light_material = Arc::new(DiffuseLight::new(light_tex));
    let light = Quad::new(
        Vec3::new(-1.0, 0.0, -1.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 2.0),
        light_material,
    );
    let transfromed_light = Transform::new(
        Box::new(light),
        Some(Vec3::new(0.0, 2.0, 0.0)),
        Some(Quaternion::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), 20.0)),
        Some(Vec3::new(3.0, 3.0, 3.0)),
    );

    let mut world = Hittables::default();
    world.add(Box::new(obj));
    world.add(Box::new(transfromed_light));

    let mut camera = Camera::default();

    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = 1920;
    camera.samples_per_pixel = 1000;
    camera.max_depth = 10;

    camera.vertical_fov_in_degrees = 40.0;
    camera.look_from = Point3::new(-10.0, 4.5, -10.0);
    camera.look_at = Point3::new(0.0, 0.0, 0.0);
    camera.vec_up = Vec3::new(0.0, 1.0, 0.0);

    camera.defocus_angle_in_degrees = 0.0;

    camera.background = Color::new(0.1, 0.1, 0.1);

    let img = camera.render(&world, None);

    drop(world);

    img
}

fn final_scene(image_width: u32, samples_per_pixel: usize, max_depth: u32) -> RgbImage {
    let mut boxes1 = Hittables::default();
    let ground_tex = Arc::new(SolidColor::new(Color::new(0.48, 0.83, 0.53)));
    let ground = Arc::new(Lambertian::new(ground_tex));

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

    let earth_tex = Arc::new(ImageTexture::new("earthmap.jpg"));
    let earth_material = Arc::new(Lambertian::new(earth_tex));
    let earth = Sphere::new(Point3::new(400.0, 200.0, 400.0), 100.0, earth_material);

    let mut world = Hittables::default();

    world.add(Box::new(earth));

    world.add(Box::new(BVH::new(boxes1)));

    let light_tex = Arc::new(SolidColor::new(Color::new(7.0, 7.0, 7.0)));
    let light_material = Arc::new(DiffuseLight::new(light_tex));
    let light = Box::new(Quad::new(
        Point3::new(123.0, 554.0, 147.0),
        Vec3::new(300.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 265.0),
        light_material,
    ));
    world.add(light);

    let center1 = Point3::new(400.0, 400.0, 200.0);
    let center2 = center1 + Vec3::new(30.0, 0.0, 0.0);
    let sphere_tex = Arc::new(SolidColor::new(Color::new(0.7, 0.3, 0.1)));
    let sphere_material = Arc::new(Lambertian::new(sphere_tex));
    world.add(Box::new(Sphere::new_with_motion(
        center1,
        center2,
        50.0,
        sphere_material,
    )));

    let glass_material = Arc::new(Dielectric::new(1.5));
    world.add(Box::new(Sphere::new(
        Point3::new(260.0, 150.0, 45.0),
        50.0,
        glass_material.clone(),
    )));

    let metal_material = Arc::new(Metal::new(Color::new(0.8, 0.8, 0.9), 1.0));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 150.0, 145.0),
        50.0,
        metal_material,
    )));

    let boundary = Box::new(Sphere::new(
        Point3::new(360.0, 150.0, 145.0),
        70.0,
        glass_material,
    ));

    world.add(boundary);

    let boundary = Box::new(Sphere::new(
        Point3::new(360.0, 150.0, 145.0),
        70.0,
        Arc::new(EmptyMaterial),
    ));

    let smoke_tex = Arc::new(SolidColor::new(Color::new(0.2, 0.4, 0.9)));
    world.add(Box::new(ConstantMedium::new_with_tex(
        boundary, 0.2, smoke_tex,
    )));
    let boundary = Box::new(Sphere::new(
        Point3::new(0.0, 0.0, 0.0),
        5000.0,
        Arc::new(EmptyMaterial),
    ));
    let white_tex = Arc::new(SolidColor::new(Color::WHITE));
    world.add(Box::new(ConstantMedium::new_with_tex(
        boundary, 0.0001, white_tex,
    )));

    let pertext = Arc::new(NoiseTexture::new(0.2));
    let noise_tex = Arc::new(Lambertian::new(pertext));
    world.add(Box::new(Sphere::new(
        Point3::new(220.0, 280.0, 300.0),
        80.0,
        noise_tex,
    )));

    let mut boxes2 = Hittables::default();
    let dim_white_color = Arc::new(SolidColor::new(Color::new(0.73, 0.73, 0.73)));
    let white = Arc::new(Lambertian::new(dim_white_color));
    const NS: usize = 1000;
    for _ in 0..NS {
        boxes2.add(Box::new(Sphere::new(
            Point3::random_range(0.0..165.0),
            10.0,
            white.clone(),
        )));
    }

    world.add(Box::new(Transform::new(
        Box::new(BVH::new(boxes2)),
        Some(Vec3::new(-100.0, 270.0, 395.0)),
        Some(Quaternion::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 15.0)),
        None,
    )));

    let mut lights = Hittables::default();
    lights.add(Box::new(Quad::new(
        Point3::new(123.0, 554.0, 147.0),
        Vec3::new(300.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 265.0),
        Arc::new(EmptyMaterial),
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

    let img = camera.render(&world, Some(&lights));

    drop(world);
    drop(lights);

    img
}

fn cornell_box() -> RgbImage {
    let mut world = Hittables::default();
    let mut lights = Hittables::default();

    let red_tex = Arc::new(SolidColor::new(Color::new(0.65, 0.05, 0.05)));
    let white_tex = Arc::new(SolidColor::new(Color::new(0.73, 0.73, 0.73)));
    let green_tex = Arc::new(SolidColor::new(Color::new(0.12, 0.45, 0.15)));
    let light_tex = Arc::new(SolidColor::new(Color::new(15.0, 15.0, 15.0)));

    let red = Arc::new(Lambertian::new(red_tex));
    let white = Arc::new(Lambertian::new(white_tex));
    let green = Arc::new(Lambertian::new(green_tex));
    let light = Arc::new(DiffuseLight::new(light_tex));

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
    world.add(Box::new(Triangle::new(
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

    let box1 = Box::new(build_box(
        Point3::ZERO,
        Point3::new(165.0, 330.0, 165.0),
        white.clone(),
    ));
    let box1 = Box::new(Transform::new(
        box1,
        Some(Vec3::new(265.0, 0.0, 295.0)),
        Some(Quaternion::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 15.0)),
        None,
    ));

    world.add(box1);

    // let glass = Dielectric::new(1.5);
    // world.add(Box::new(Sphere::new(
    //     Point3::new(190.0, 90.0, 190.0),
    //     90.0,
    //     &glass,
    // )));

    lights.add(Box::new(Triangle::new(
        Point3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        light,
    )));

    let mut camera = Camera::default();
    camera.aspect_ratio = 1.0;
    camera.image_width = 300;
    camera.samples_per_pixel = 10;
    camera.max_depth = 10;
    camera.background = Color::BLACK;

    camera.vertical_fov_in_degrees = 40.0;
    camera.look_from = Point3::new(278.0, 278.0, -800.0);
    camera.look_at = Point3::new(278.0, 278.0, 0.0);
    camera.vec_up = Vec3::new(0.0, 1.0, 0.0);

    camera.defocus_angle_in_degrees = 0.0;

    let img = camera.render(&world, Some(&lights));

    drop(world);
    drop(lights);

    img
}
