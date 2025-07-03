use std::cmp::Ordering;

use crate::{aabb::AABB, hit::Hittable, hits::Hittables, utils::interval::Interval};

pub struct BVH {
    left: Option<Box<dyn Hittable>>,
    right: Option<Box<dyn Hittable>>,
    bbox: AABB,
}

impl BVH {
    pub fn new(world: Hittables) -> BVH {
        BVH::from_vec(world.objects)
    }

    pub fn from_vec(mut objects: Vec<Box<dyn Hittable>>) -> BVH {
        let bbox = objects
            .iter()
            .fold(AABB::EMPTY, |x, y| AABB::union(x, *y.bounding_box()));

        let axis = bbox.longest_axis();

        let len = objects.len();

        let (left, right) = match len {
            0 => panic!("BVH node must contain at least one object"),
            1 => (Some(objects.into_iter().next().unwrap()), None),
            2 => {
                let mut iter = objects.into_iter();
                (Some(iter.next().unwrap()), Some(iter.next().unwrap()))
            }
            _ => {
                objects.sort_by(|a, b| BVH::box_compare(a.as_ref(), b.as_ref(), axis));

                let mid = len / 2;
                let right_vec = objects.split_off(mid);
                let left_vec = objects;
                let left: Option<Box<dyn Hittable>> = Some(Box::new(BVH::from_vec(left_vec)));
                let right: Option<Box<dyn Hittable>> = Some(Box::new(BVH::from_vec(right_vec)));

                (left, right)
            }
        };

        BVH { left, right, bbox }
    }

    fn box_compare(a: &dyn Hittable, b: &dyn Hittable, axis_index: usize) -> Ordering {
        let a_axis_interval = a.bounding_box().axis_interval(axis_index);
        let b_axis_interval = b.bounding_box().axis_interval(axis_index);

        f64::total_cmp(a_axis_interval.min(), b_axis_interval.min())
    }
}

impl Hittable for BVH {
    fn hit(
        &self,
        r: &crate::utils::ray::Ray,
        interval: &Interval,
    ) -> Option<crate::hit::HitRecord> {
        if !self.bbox.hit(r, *interval) {
            return None;
        }

        let mut hit_left = None;
        let mut closest_so_far = *interval.max();

        if let Some(left) = &self.left {
            if let Some(rec) = left.hit(r, interval) {
                closest_so_far = rec.t;
                hit_left = Some(rec);
            }
        }

        let mut hit_right = None;
        if let Some(right) = &self.right {
            let right_interval = Interval::new(*interval.min(), closest_so_far);
            if let Some(rec) = right.hit(r, &right_interval) {
                hit_right = Some(rec);
            }
        }

        hit_right.or(hit_left)
    }

    fn bounding_box(&self) -> &AABB {
        &self.bbox
    }
}
