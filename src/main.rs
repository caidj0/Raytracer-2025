use std::sync::Arc;

use console::style;
use image::RgbImage;
use raytracer::{
    bvh::BVH,
    camera::Camera,
    hits::Hittables,
    material::{
        Dielectric, DiffuseLight, EmptyMaterial, Lambertian, Metal, Mix, disney::Disney,
        portal::Portal,
    },
    shapes::{
        Transform,
        obj::Wavefont,
        quad::{Quad, build_box},
        sphere::Sphere,
    },
    texture::{ImageTexture, NoiseTexture, SolidColor},
    utils::{
        color::{Color, ToonMap},
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
        3 => obj_scene(),
        4 => background_scene(),
        5 => disney_scene(),
        _ => portal_scene(),
    };
    let path_string = format!("output/{}/{}.png", "final", "final");
    let path = std::path::Path::new(&path_string);
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).expect("Cannot create all the parents");
    println!(
        "Output image as \"{}\"",
        style(path.to_str().unwrap()).yellow()
    );
    img.save(path).expect("Cannot save the image to the file");
}

fn portal_scene() -> RgbImage {
    let mut world = Hittables::default();
    let portal_material = Arc::new(Portal::new(
        Color::WHITE,
        Vec3::new(2.0, 0.0, 0.0),
        Quaternion::identity(),
    ));

    let quad = Quad::new(
        Vec3::new(-1.0, 0.0, -1.0),
        Vec3::new(0.0, 0.0, 2.0),
        Vec3::new(2.0, 0.0, 0.0),
        portal_material,
    );

    let sphere_material = Arc::new(Lambertian::new(Arc::new(SolidColor::new(Color::WHITE))));
    let sphere = Sphere::new(Vec3::new(2.0, -1.5, 0.0), 1.0, sphere_material);

    world.add(Box::new(quad));
    world.add(Box::new(sphere));

    let mut camera = Camera::default();

    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = 1920;
    camera.samples_per_pixel = 500;
    camera.max_depth = 10;

    camera.vertical_fov_in_degrees = 40.0;
    camera.look_from = Point3::new(0.0, 2.0, 1.0) * 2.0;
    camera.look_at = Point3::new(0.0, 0.0, 0.0);
    camera.vec_up = Vec3::new(0.0, 1.0, 0.0);

    camera.defocus_angle_in_degrees = 0.0;
    camera.toon_map = ToonMap::ACES;

    let back_tex = ImageTexture::new("rogland_clear_night_4k.exr");
    camera.background.texture = Arc::new(back_tex);

    camera.render(&world, None)
}

fn disney_scene() -> RgbImage {
    let mut world = Hittables::default();

    let disney = Arc::new(
        Disney::builder()
            .base_color(Color::WHITE)
            .roughness(0.0)
            .anisotropic(0.0)
            .sheen(0.0)
            .sheen_tint(0.0)
            .clearcoat(0.0)
            .clearcoat_gloss(0.0)
            .specular_tint(0.0)
            .metallic(1.0)
            .ior(1.5)
            .flatness(0.0)
            .spec_trans(0.0)
            .diff_trans(0.0)
            .thin(false)
            .build(),
    );

    // let lab = Arc::new(Lambertian::new(Arc::new(SolidColor::new(Color::WHITE))));
    // let metal = Arc::new(Metal::new(Color::WHITE, 0.5));
    // let die = Arc::new(Dielectric::new(1.5));

    // world.add(Box::new(Quad::new(
    //     Vec3::new(-1.0, 0.0, -1.0),
    //     Vec3::new(0.0, 0.0, 2.0),
    //     Vec3::new(2.0, 0.0, 0.0),
    //     disney,
    // )));
    world.add(Box::new(Sphere::new(Vec3::ZERO, 1.0, disney)));
    // let light = Sphere::new(
    //     Vec3::new(0.0, -0.3, 0.0),
    //     0.2,
    //     Arc::new(DiffuseLight::new(Arc::new(SolidColor::new(Color::new(
    //         3.0, 3.0, 3.0,
    //     ))))),
    // );
    // world.add(Box::new(light.clone()));

    let mut camera = Camera::default();

    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = 1920;
    camera.samples_per_pixel = 500;
    camera.max_depth = 10;

    camera.vertical_fov_in_degrees = 40.0;
    camera.look_from = Point3::new(-2.0, 1.0, 0.0) * 2.0;
    camera.look_at = Point3::new(0.0, 0.0, 0.0);
    camera.vec_up = Vec3::new(0.0, 1.0, 0.0);

    camera.defocus_angle_in_degrees = 0.0;
    camera.toon_map = ToonMap::ACES;

    let back_tex = ImageTexture::new("rogland_clear_night_4k.exr");
    camera.background.texture = Arc::new(back_tex);

    camera.render(&world, None)
}

fn background_scene() -> RgbImage {
    let mut world = Hittables::default();
    // world.add(Box::new(Sphere::new(
    //     Vec3::new(0.0, 0.0, 0.0),
    //     1.0,
    //     Arc::new(Dielectric::new(1.5)),
    // )));

    let metal_mat = Arc::new(Metal::new(Color::WHITE, 0.0));
    let lam_mat = Arc::new(Lambertian::new(Arc::new(SolidColor::new(Color::new(
        0.8, 0.8, 0.8,
    )))));
    let mix_mat = Arc::new(Mix::new(metal_mat.clone(), lam_mat.clone(), 0.5));
    world.add(Box::new(Quad::new(
        Vec3::new(-2.0, -2.0, -2.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 4.0),
        mix_mat,
    )));

    let light_mat = Arc::new(DiffuseLight::new(Arc::new(SolidColor::new(
        Color::new(0.75, 1.0, 0.58) * 1.0,
    ))));
    let mix_mat2 = Arc::new(Mix::new(light_mat.clone(), metal_mat.clone(), 0.5));
    let light = Sphere::new(Vec3::new(1.5, -1.5, 0.0), 0.2, mix_mat2);
    world.add(Box::new(light.clone()));

    let mut camera = Camera::default();

    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = 1920;
    camera.samples_per_pixel = 100;
    camera.max_depth = 10;

    camera.vertical_fov_in_degrees = 40.0;
    camera.look_from = Point3::new(-2.0, 1.0, 0.0) * 2.0;
    camera.look_at = Point3::new(0.0, 0.0, 0.0);
    camera.vec_up = Vec3::new(0.0, 1.0, 0.0);

    camera.defocus_angle_in_degrees = 0.0;
    camera.toon_map = ToonMap::ACES;

    let back_tex = ImageTexture::new("rogland_clear_night_4k.exr");
    camera.background.texture = Arc::new(back_tex);

    let img = camera.render(&world, Some(&light));

    drop(world);

    img
}

fn obj_scene() -> RgbImage {
    let miku = Wavefont::new("初音未来.obj", "Final", false).unwrap();
    let ball = Wavefont::new("玻璃球.obj", "Final", false).unwrap();
    let frame = Wavefont::new("外框.obj", "Final", false).unwrap();
    let sound_box = Wavefont::new("声匣.obj", "Final", false).unwrap();
    let mirror_door = Wavefont::new("镜子门.obj", "Final", false).unwrap();
    let mirror = Wavefont::new("镜子.obj", "Final", true).unwrap();
    let ring = Wavefont::new("环.obj", "Final", false).unwrap();
    let portal_frame = Wavefont::new("传送门框.obj", "Final", false).unwrap();
    let under_water = Wavefont::new("水下.obj", "Final", false).unwrap();
    let water = Wavefont::new("水面.obj", "Final", true).unwrap();
    let text = Wavefont::new("文字.obj", "Final", false).unwrap();
    let mc = Wavefont::new("mc.obj", "Final", false).unwrap();
    let umbralla = Wavefont::new("伞.obj", "Final", false).unwrap();
    let checker = Wavefont::new("卒.obj", "Final", false).unwrap();

    let forg = Wavefont::new("雾.obj", "Final", false).unwrap();
    let forg = ConstantMedium::new_with_tex(
        Box::new(forg),
        0.05,
        Arc::new(SolidColor::new(Color::new(1.0, 0.936, 0.381))),
    );

    let portal_material = Arc::new(Portal::new(
        Color::WHITE,
        Vec3::new(0.0, -6.3, 1.1),
        Quaternion::identity(),
    ));

    let portal_anchor = Vec3::new(-5.8035, -0.9983, -7.7198);
    let portal_u = Vec3::new(-3.8206, -0.9983, -8.3722) - portal_anchor;
    let portal_v = Vec3::new(-5.8035, 3.1159, -7.7198) - portal_anchor;
    let portal = Quad::new(portal_anchor, portal_u, portal_v, portal_material);

    let translucent_material = Arc::new(
        Disney::builder()
            .diff_trans(1.0)
            .roughness(1.0)
            .thin(true)
            .build(),
    );
    let translucent_board = Quad::new(
        Vec3::new(-1.0, 0.0, -1.0),
        Vec3::new(0.0, 0.0, 2.0),
        Vec3::new(2.0, 0.0, 0.0),
        translucent_material,
    );
    let translucent_board = Transform::new(
        Box::new(translucent_board),
        Some(Vec3::new(2.8145, -0.23603, -19.501)),
        Some(Quaternion::from_axis_angle(
            Vec3::new(0.993, -0.082, 0.082),
            90.4,
        )),
        Some(Vec3::new(2.616, 1.0, 1.0)),
    );

    let light_material = Arc::new(DiffuseLight::new(Arc::new(SolidColor::new(Color::new(
        4.0, 4.0, 4.0,
    )))));
    let light_board = Quad::new(
        Vec3::new(-1.0, 0.0, -1.0),
        Vec3::new(0.0, 0.0, 2.0),
        Vec3::new(2.0, 0.0, 0.0),
        light_material,
    );
    let light_board = Transform::new(
        Box::new(light_board),
        Some(Vec3::new(-0.44579, 5.2955, 0.89889)),
        Some(Quaternion::from_axis_angle(
            Vec3::new(0.921, 0.021, 0.389),
            34.7,
        )),
        Some(Vec3::new(3.415, 3.415, 3.415)),
    );
    let yellow_material = Arc::new(DiffuseLight::new(Arc::new(SolidColor::new(
        5.0 * Color::new(1.0, 0.687, 0.0),
    ))));
    let yellow_board = Quad::new(
        Vec3::new(-1.0, 0.0, -1.0),
        Vec3::new(0.0, 0.0, 2.0),
        Vec3::new(2.0, 0.0, 0.0),
        yellow_material,
    );
    let yellow_board = Transform::new(
        Box::new(yellow_board),
        Some(Vec3::new(-1.0053, -1.9655, -4.242)),
        Some(Quaternion::from_axis_angle(
            Vec3::new(0.766, 0.483, -0.423),
            85.7,
        )),
        Some(Vec3::new(1.0, 1.0, 1.0) * 1.499),
    );

    let black_box = Transform::new(
        Box::new(build_box(
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Arc::new(DiffuseLight::new(Arc::new(SolidColor::new(Color::BLACK)))),
        )),
        Some(Vec3::new(-4.9891, -6.4998, -8.3939)),
        None,
        Some(Vec3::new(1.0, 1.0, 1.0) * 6.244),
    );

    let mut world = Hittables::default();
    world.add(Box::new(miku));
    world.add(Box::new(light_board));
    world.add(Box::new(ball));
    world.add(Box::new(frame));
    world.add(Box::new(sound_box));
    world.add(Box::new(mirror_door));
    world.add(Box::new(mirror));
    world.add(Box::new(ring));
    world.add(Box::new(portal_frame));
    world.add(Box::new(under_water));
    world.add(Box::new(water));
    world.add(Box::new(text));
    world.add(Box::new(translucent_board));
    world.add(Box::new(mc));
    world.add(Box::new(portal));
    world.add(Box::new(umbralla));
    world.add(Box::new(yellow_board));
    world.add(Box::new(black_box));
    world.add(Box::new(forg));
    world.add(Box::new(checker));

    let mut camera = Camera::from_json("Final/camera.json").unwrap();

    camera.samples_per_pixel = 5000;
    camera.max_depth = 30;

    let backtex = ImageTexture::new("13.hdr");
    camera.background.texture = Arc::new(backtex);

    let light_board = Quad::new(
        Vec3::new(-1.0, 0.0, -1.0),
        Vec3::new(0.0, 0.0, 2.0),
        Vec3::new(2.0, 0.0, 0.0),
        Arc::new(EmptyMaterial),
    );
    let light_board = Transform::new(
        Box::new(light_board),
        Some(Vec3::new(-0.44579, 5.2955, 0.89889)),
        Some(Quaternion::from_axis_angle(
            Vec3::new(0.921, 0.021, 0.389),
            34.7,
        )),
        Some(Vec3::new(3.415, 3.415, 3.415)),
    );
    let yellow_board = Quad::new(
        Vec3::new(-1.0, 0.0, -1.0),
        Vec3::new(0.0, 0.0, 2.0),
        Vec3::new(2.0, 0.0, 0.0),
        Arc::new(EmptyMaterial),
    );
    let yellow_board = Transform::new(
        Box::new(yellow_board),
        Some(Vec3::new(-1.0053, -1.9655, -4.242)),
        Some(Quaternion::from_axis_angle(
            Vec3::new(0.766, 0.483, -0.423),
            85.7,
        )),
        Some(Vec3::new(1.0, 1.0, 1.0) * 1.499),
    );

    let mut lights = Hittables::default();
    lights.add(Box::new(light_board));
    lights.add(Box::new(yellow_board));

    let img = camera.render(&world, Some(&lights));

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

    let glass_material = Arc::new(Dielectric::new(
        Arc::new(SolidColor::new(Color::WHITE)),
        1.5,
    ));
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

    lights.add(Box::new(Quad::new(
        Point3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        light,
    )));

    let mut camera = Camera::default();
    camera.aspect_ratio = 1.0;
    camera.image_width = 1080;
    camera.samples_per_pixel = 100;
    camera.max_depth = 10;

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
