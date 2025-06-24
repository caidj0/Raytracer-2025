use std::ops::Range;

use crate::hit::Hittable;

#[derive(Default)]
pub struct Hittables {
    pub objects: Vec<Box<dyn Hittable>>,
}

impl Hittables {
    pub fn new(object: Box<dyn Hittable>) -> Hittables {
        Hittables {
            objects: vec![object],
        }
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn add(&mut self, object: Box<dyn Hittable>) {
        self.objects.push(object);
    }
}

impl Hittable for Hittables {
    fn hit(
        &self,
        r: &crate::utils::ray::Ray,
        interval: &Range<f64>,
    ) -> Option<crate::hit::HitRecord> {
        self.objects
            .iter()
            .filter_map(|x| x.hit(r, interval))
            .min_by(|x, y| {
                x.t.partial_cmp(&y.t)
                    .expect("The length of ray should not be NaN!")
            })
    }
}
