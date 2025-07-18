use std::{
    env::{self, current_dir},
    path::PathBuf,
    sync::Arc,
};

use crate::{
    bvh::BVH,
    hit::{HitRecord, Hittable},
    hits::Hittables,
    material::{
        Dielectric, DiffuseLight, EmptyMaterial, Material, Metal, Mix, Transparent,
        disney::{Disney, DisneyParameters},
    },
    shapes::triangle::Triangle,
    texture::{ImageTexture, SolidColor, Texture},
    utils::vec3::{Point3, UnitVec3, Vec3},
};

struct RemappedMaterial {
    pub material: Arc<dyn Material>,
    pub tex_ori: Point3,
    pub tex_u: Vec3,
    pub tex_v: Vec3,
    pub u_vec: Option<UnitVec3>,
    pub v_vec: Option<UnitVec3>,
    pub normal: [Vec3; 3],
    pub normal_tex: Option<Arc<ImageTexture>>,
}

impl RemappedMaterial {
    fn remap_record<'a>(&self, rec: &'a HitRecord) -> HitRecord<'a> {
        let tex_coord = self.tex_ori + rec.u * self.tex_u + rec.v * self.tex_v;
        let normal = UnitVec3::from_vec3(
            (1.0 - rec.u - rec.v) * self.normal[0]
                + rec.u * self.normal[1]
                + rec.v * self.normal[2],
        )
        .unwrap();

        let normal = if let Some(normal_tex) = &self.normal_tex {
            let normal_color = normal_tex.value(tex_coord.x(), tex_coord.y(), &rec.p);
            let normal_color = normal_color * 2.0 - Vec3::new(1.0, 1.0, 1.0);

            let normal_raw = self.u_vec.unwrap().as_inner() * normal_color[0]
                + self.v_vec.unwrap().as_inner() * normal_color[1]
                + normal.as_inner() * normal_color[2];
            UnitVec3::from_vec3(normal_raw).expect("The mapped normal can't normalized!")
        } else {
            normal
        };

        HitRecord {
            p: rec.p,
            normal,
            mat: rec.mat,
            t: rec.t,
            u: tex_coord.x(),
            v: tex_coord.y(),
            front_face: rec.front_face,
        }
    }
}

impl Material for RemappedMaterial {
    fn scatter(
        &self,
        r_in: &crate::utils::ray::Ray,
        rec: &crate::hit::HitRecord,
    ) -> Option<crate::material::ScatterRecord> {
        self.material.scatter(r_in, &self.remap_record(rec))
    }

    fn emitted(
        &self,
        r_in: &crate::utils::ray::Ray,
        rec: &HitRecord,
    ) -> crate::utils::color::Color {
        self.material.emitted(r_in, &self.remap_record(rec))
    }
}

pub struct Wavefont {
    objects: Hittables,
}

impl Wavefont {
    fn load(file_name: &str) -> tobj::LoadResult {
        if let Ok(specified_dir) = env::var("RTW_OBJS") {
            let mut path = PathBuf::new();
            path.push(specified_dir);
            path.push(file_name);
            return tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS);
        };

        let Ok(pathbuf) = current_dir() else {
            return Err(tobj::LoadError::OpenFileFailed);
        };

        let mut path = pathbuf;
        path.push("assets");
        path.push(file_name);

        tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS)
    }

    fn get_three_values(v: &[f64], index: usize) -> Vec3 {
        let index = index * 3;
        Vec3::from([v[index], v[index + 1], v[index + 2]])
    }

    fn get_two_values(v: &[f64], index: usize) -> Vec3 {
        let index = index * 2;
        Vec3::from([v[index], v[index + 1], 0.0])
    }

    pub fn new(file_name: &str, prefix: &str, vanilla_material: bool) -> Option<Wavefont> {
        let file_path = prefix.to_owned() + "/" + file_name;
        let (objects, materials) = Self::load(&file_path).ok()?;

        let mut mats: Vec<Arc<dyn Material>> = vec![];
        let mut normals: Vec<Option<Arc<ImageTexture>>> = vec![];
        if let Ok(materials) = materials {
            load_materials(&mut mats, &mut normals, materials, prefix, vanilla_material);
        }

        let mut obs = Hittables::default();

        for (object, normal) in objects.iter().zip(normals.iter()) {
            load_object(file_name, &mats, &mut obs, object, normal);
        }

        Some(Wavefont { objects: obs })
    }
}

fn load_object(
    file_name: &str,
    mats: &[Arc<dyn Material>],
    obs: &mut Hittables,
    object: &tobj::Model,
    normal_texture: &Option<Arc<ImageTexture>>,
) {
    let empty_material = Arc::new(EmptyMaterial);

    let mut v: Vec<Box<dyn Hittable>> = Vec::new();
    for indices in object.mesh.indices.chunks(3) {
        let p1 = Wavefont::get_three_values(&object.mesh.positions, indices[0] as usize);
        let p2 = Wavefont::get_three_values(&object.mesh.positions, indices[1] as usize);
        let p3 = Wavefont::get_three_values(&object.mesh.positions, indices[2] as usize);

        let tex_p1 = Wavefont::get_two_values(&object.mesh.texcoords, indices[0] as usize);
        let tex_p2 = Wavefont::get_two_values(&object.mesh.texcoords, indices[1] as usize);
        let tex_p3 = Wavefont::get_two_values(&object.mesh.texcoords, indices[2] as usize);

        let n_p1 = Wavefont::get_three_values(&object.mesh.normals, indices[0] as usize);
        let n_p2 = Wavefont::get_three_values(&object.mesh.normals, indices[1] as usize);
        let n_p3 = Wavefont::get_three_values(&object.mesh.normals, indices[2] as usize);

        let mat = if let Some(id) = object.mesh.material_id {
            mats[id].clone()
        } else {
            empty_material.clone()
        };

        let tex_u = tex_p2 - tex_p1;
        let tex_v = tex_p3 - tex_p1;

        let world_u = p2 - p1;
        let world_v = p3 - p1;

        let (u_vec, v_vec) = uv_local_to_world(tex_u, tex_v, world_u, world_v);

        let mat = Arc::new(RemappedMaterial {
            material: mat,
            tex_ori: tex_p1,
            tex_u,
            tex_v,
            u_vec,
            v_vec,
            normal: [n_p1, n_p2, n_p3],
            normal_tex: normal_texture.clone(),
        });

        if let Some(triangle) = Triangle::new(p1, world_u, world_v, mat) {
            v.push(Box::new(triangle));
        }
    }
    if !v.is_empty() {
        obs.add(Box::new(BVH::from_vec(v)));
    } else {
        println!("The object {} from {} is empty!", object.name, file_name);
    }
}

fn uv_local_to_world(
    tex_u: Vec3,
    tex_v: Vec3,
    world_u: Vec3,
    world_v: Vec3,
) -> (Option<UnitVec3>, Option<UnitVec3>) {
    let ua = tex_v.y() / (-tex_u.y() * tex_v.x() + tex_u.x() * tex_v.y());
    let ub = tex_u.y() / (tex_u.y() * tex_v.x() - tex_u.x() * tex_v.y());
    let va = tex_v.x() / (tex_u.y() * tex_v.x() - tex_u.x() * tex_v.y());
    let vb = tex_u.x() / (-tex_u.y() * tex_v.x() + tex_u.x() * tex_v.y());

    let u_vec = UnitVec3::from_vec3(world_u * ua + world_v * ub);
    let v_vec = UnitVec3::from_vec3(world_u * va + world_v * vb);
    (u_vec, v_vec)
}

fn load_materials(
    mats: &mut Vec<Arc<dyn Material>>,
    normals: &mut Vec<Option<Arc<ImageTexture>>>,
    materials: Vec<tobj::Material>,
    prefix: &str,
    vanilla_material: bool,
) {
    let transparent_mat = Arc::new(Transparent);

    for material in materials {
        let base_color: Arc<dyn Texture> = if let Some(tex_name) = material.diffuse_texture {
            let file_path = prefix.to_owned() + "/" + &tex_name;
            Arc::new(ImageTexture::new(&file_path))
        } else if let Some(color) = material.diffuse {
            Arc::new(SolidColor::from(color))
        } else {
            panic!("The material should at least have one diffuse!")
        };
        let roughness = material
            .unknown_param
            .get("Pr")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.5);
        let anisotropic = material
            .unknown_param
            .get("aniso")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        let sheen = material
            .unknown_param
            .get("Ps")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        let metallic = material
            .unknown_param
            .get("Pm")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        let clearcoat = material
            .unknown_param
            .get("Pc")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        let clearcoat_gloss = material
            .unknown_param
            .get("Pcr")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        let ior = material.optical_density.unwrap_or(1.45);
        let spec_trans: f64 = if let Some(tf) = material.unknown_param.get("Tf") {
            let spec_trans_vals: Vec<f64> = tf
                .split_whitespace()
                .filter_map(|s| s.parse::<f64>().ok())
                .collect();
            spec_trans_vals.iter().sum::<f64>() / spec_trans_vals.len() as f64
        } else {
            0.0
        };

        let mut mat: Arc<dyn Material> = if vanilla_material && metallic == 1.0 {
            Arc::new(Metal::new(
                base_color.value(0.0, 0.0, &Vec3::ZERO),
                roughness,
            ))
        } else if vanilla_material && spec_trans == 1.0 {
            Arc::new(Dielectric::new(base_color, ior))
        } else {
            Arc::new(Disney {
                param_fn: Box::new(move |u, v, p| DisneyParameters {
                    base_color: base_color.value(u, v, p),
                    roughness,
                    anisotropic,
                    sheen,
                    clearcoat,
                    clearcoat_gloss,
                    metallic,
                    ior,
                    spec_trans,
                    ..Default::default()
                }),
            })
        };

        if let Some(emit) = material.unknown_param.get("Ke") {
            let emit_vals: Vec<f64> = emit
                .split_whitespace()
                .filter_map(|s| s.parse::<f64>().ok())
                .collect();
            if emit_vals.len() == 3 {
                let color = SolidColor::from([emit_vals[0], emit_vals[1], emit_vals[2]]);
                mat = Arc::new(DiffuseLight::new_with_material(Arc::new(color), mat));
            }
        }

        if let Some(emit) = material.unknown_param.get("map_Ke") {
            let file_path = prefix.to_owned() + "/" + emit;
            let emit_tex = ImageTexture::new(&file_path);
            mat = Arc::new(DiffuseLight::new_with_material(Arc::new(emit_tex), mat));
        }

        if let Some(dissolve_tex) = material.dissolve_texture {
            let file_path = prefix.to_owned() + "/" + &dissolve_tex;
            mat = Arc::new(Mix::from_image(
                transparent_mat.clone(),
                mat,
                Arc::new(ImageTexture::new(&file_path)),
            ));
        }
        if let Some(dissolve) = material.dissolve {
            if dissolve < 1.0 {
                mat = Arc::new(Mix::new(transparent_mat.clone(), mat, dissolve));
            }
        }

        mats.push(mat);

        if let Some(tex_name) = material.normal_texture {
            // 处理 tex_name 可能为 "文件名" 或 "-bm 1.000000 文件名"
            let file_path = if let Some(stripped) = tex_name.strip_prefix("-bm") {
                let parts: Vec<&str> = stripped.split_whitespace().collect();
                if let Some(fname) = parts.last() {
                    prefix.to_owned() + "/" + fname
                } else {
                    prefix.to_owned() + "/" + &tex_name
                }
            } else {
                prefix.to_owned() + "/" + &tex_name
            };
            normals.push(Some(Arc::new(ImageTexture::new_raw_image(&file_path))));
        } else {
            normals.push(None);
        }
    }
}

impl Hittable for Wavefont {
    fn hit(
        &self,
        r: &crate::utils::ray::Ray,
        interval: &crate::utils::interval::Interval,
    ) -> Option<crate::hit::HitRecord> {
        self.objects.hit(r, interval)
    }

    fn bounding_box(&self) -> &crate::aabb::AABB {
        self.objects.bounding_box()
    }

    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {
        self.objects.pdf_value(origin, direction)
    }

    fn random(&self, origin: &Point3) -> UnitVec3 {
        self.objects.random(origin)
    }
}
