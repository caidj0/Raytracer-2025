use crate::{aabb::AABB, hit::Hittable, utils::interval::Interval};

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
        self.bbox = self.bbox.union(object.bounding_box());
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
}
