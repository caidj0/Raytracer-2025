use std::{
    env::{self, current_dir},
    path::PathBuf,
    sync::Arc,
};

use crate::{
    bvh::BVH,
    hit::{HitRecord, Hittable},
    hits::Hittables,
    material::{EmptyMaterial, Lambertian, Material},
    shapes::triangle::Triangle,
    texture::{ImageTexture, SolidColor, Texture},
    utils::vec3::{Point3, Vec3},
};

struct RemappedMaterial {
    pub material: Arc<dyn Material>,
    pub tex_ori: Point3,
    pub tex_u: Vec3,
    pub tex_v: Vec3,
}

impl RemappedMaterial {
    fn remap_record<'a>(&self, rec: &'a HitRecord) -> HitRecord<'a> {
        let tex_coord = self.tex_ori + rec.u * self.tex_u + rec.v * self.tex_v;
        HitRecord {
            p: rec.p,
            normal: rec.normal,
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

    fn scattering_pdf(
        &self,
        r_in: &crate::utils::ray::Ray,
        rec: &crate::hit::HitRecord,
        scattered: &crate::utils::ray::Ray,
    ) -> f64 {
        self.material
            .scattering_pdf(r_in, &self.remap_record(rec), scattered)
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

    pub fn new(file_name: &str) -> Option<Wavefont> {
        let (objects, materials) = Self::load(file_name).ok()?;

        let mut obs = Hittables::default();
        let mut mats: Vec<Arc<dyn Material>> = vec![];
        if let Ok(materials) = materials {
            for material in materials {
                let tex: Arc<dyn Texture> = if let Some(tex_name) = material.diffuse_texture {
                    Arc::new(ImageTexture::new(&tex_name))
                } else if let Some(color) = material.diffuse {
                    Arc::new(SolidColor::from(color))
                } else {
                    panic!("The material should at least have one diffuse!")
                };

                let mat = Arc::new(Lambertian::new(tex));
                mats.push(mat);
            }
        }

        let empty_material = Arc::new(EmptyMaterial);

        for object in objects {
            let mut v: Vec<Box<dyn Hittable>> = Vec::new();
            for indices in object.mesh.indices.chunks(3) {
                let p1 = Wavefont::get_three_values(&object.mesh.positions, indices[0] as usize);
                let p2 = Wavefont::get_three_values(&object.mesh.positions, indices[1] as usize);
                let p3 = Wavefont::get_three_values(&object.mesh.positions, indices[2] as usize);

                let tex_p1 = Wavefont::get_two_values(&object.mesh.texcoords, indices[0] as usize);
                let tex_p2 = Wavefont::get_two_values(&object.mesh.texcoords, indices[1] as usize);
                let tex_p3 = Wavefont::get_two_values(&object.mesh.texcoords, indices[2] as usize);

                let mat = if let Some(id) = object.mesh.material_id {
                    mats[id].clone()
                } else {
                    empty_material.clone()
                };

                let mat = Arc::new(RemappedMaterial {
                    material: mat,
                    tex_ori: tex_p1,
                    tex_u: tex_p2 - tex_p1,
                    tex_v: tex_p3 - tex_p1,
                });

                v.push(Box::new(Triangle::new(p1, p2 - p1, p3 - p1, mat)));
            }
            if !v.is_empty() {
                obs.add(Box::new(BVH::from_vec(v)));
            } else {
                println!("The object {} from {} is empty!", object.name, file_name);
            }
        }

        Some(Wavefont { objects: obs })
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
}
