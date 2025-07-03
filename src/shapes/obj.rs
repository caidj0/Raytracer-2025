use std::{
    env::{self, current_dir},
    path::PathBuf,
    sync::Arc,
};

use crate::{
    bvh::BVH,
    hit::Hittable,
    hits::Hittables,
    material::{EmptyMaterial, Lambertian, Material},
    shapes::triangle::Triangle,
    texture::SolidColor,
    utils::vec3::Vec3,
};

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

    fn get_position(positions: &[f64], index: usize) -> Vec3 {
        let index = index * 3;
        Vec3::from(positions[index..(index + 3)].try_into().unwrap())
    }

    pub fn new(file_name: &str) -> Option<Wavefont> {
        let (objects, materials) = Self::load(file_name).ok()?;

        let mut obs = Hittables::default();
        let mut mats: Vec<Arc<dyn Material>> = vec![];
        if let Some(materials) = materials.ok() {
            for material in materials {
                let mat = Arc::new(Lambertian::new(Arc::new(SolidColor::from(
                    material.diffuse.unwrap(),
                ))));
                mats.push(mat);
            }
        }

        let empty_material = Arc::new(EmptyMaterial);

        for object in objects {
            let mut v: Vec<Box<dyn Hittable>> = Vec::new();
            for vs in object.mesh.indices.chunks(3) {
                let p1 = Wavefont::get_position(&object.mesh.positions, vs[0] as usize);
                let p2 = Wavefont::get_position(&object.mesh.positions, vs[1] as usize);
                let p3 = Wavefont::get_position(&object.mesh.positions, vs[2] as usize);

                let mat = if let Some(id) = object.mesh.material_id {
                    mats[id].clone()
                } else {
                    empty_material.clone()
                };

                v.push(Box::new(Triangle::new(p1, p2 - p1, p3 - p1, mat)));
            }
            obs.add(Box::new(BVH::from_vec(v)));
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
