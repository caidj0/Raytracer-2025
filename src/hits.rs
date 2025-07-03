use rand::seq::IndexedRandom;

use crate::{
    aabb::AABB,
    hit::Hittable,
    utils::{interval::Interval, random::Random},
};

#[derive(Default)]
pub struct Hittables {
    pub objects: Vec<Box<dyn Hittable>>,
    bbox: AABB,
}

impl Hittables {
    pub fn new(object: Box<dyn Hittable>) -> Hittables {
        Hittables {
            bbox: *object.bounding_box(),
            objects: vec![object],
        }
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn add(&mut self, object: Box<dyn Hittable>) {
        self.bbox = self.bbox.union(*object.bounding_box());
        self.objects.push(object);
    }
}

impl Hittable for Hittables {
    fn hit(
        &self,
        r: &crate::utils::ray::Ray,
        interval: &Interval,
    ) -> Option<crate::hit::HitRecord> {
        self.objects
            .iter()
            .filter_map(|x| x.hit(r, interval))
            .min_by(|x, y| {
                x.t.partial_cmp(&y.t)
                    .expect("The length of ray should not be NaN!")
            })
    }

    fn bounding_box(&self) -> &AABB {
        &self.bbox
    }

    fn pdf_value(
        &self,
        origin: &crate::utils::vec3::Point3,
        direction: &crate::utils::vec3::Vec3,
    ) -> f64 {
        let sum: f64 = self
            .objects
            .iter()
            .map(|object| object.pdf_value(origin, direction))
            .sum();

        let ret = sum / self.objects.len() as f64;
        assert!(!ret.is_nan(), "The sum of pdf is NaN!");

        ret
    }

    fn random(&self, origin: &crate::utils::vec3::Point3) -> crate::utils::vec3::Vec3 {
        let object = self
            .objects
            .choose(&mut Random::rng())
            .expect("The collection of objects is empty!");
        object.random(origin)
    }
}
