use crate::utils::{interval::Interval, ray::Ray, vec3::Point3};

#[derive(Default, Clone, Copy)]
pub struct AABB {
    x: Interval,
    y: Interval,
    z: Interval,
}

impl AABB {
    pub fn new(x: Interval, y: Interval, z: Interval) -> AABB {
        AABB { x, y, z }.pad_to_minimums()
    }

    pub fn from_points(a: Point3, b: Point3) -> AABB {
        AABB {
            x: Interval::new(a[0], b[0]),
            y: Interval::new(a[1], b[1]),
            z: Interval::new(a[2], b[2]),
        }
        .pad_to_minimums()
    }

    fn pad_to_minimums(self) -> AABB {
        const DELTA: f64 = 0.0001;

        let (x, y, z) = [self.x, self.y, self.z]
            .map(|t| if t.size() < DELTA { t.expand(DELTA) } else { t })
            .into();

        AABB { x, y, z }
    }

    pub fn axis_interval(&self, n: usize) -> &Interval {
        match n {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("The index of axis should between 0 and 2!"),
        }
    }

    pub fn hit(&self, r: &Ray, ray_t: Interval) -> bool {
        let ray_orig = r.origin();
        let ray_dir = r.direction();

        (0..3)
            .map(|axis| {
                let ax = self.axis_interval(axis);
                let adinv = 1.0 / ray_dir[axis];

                let t0 = (ax.min() - ray_orig[axis]) * adinv;
                let t1 = (ax.max() - ray_orig[axis]) * adinv;

                Interval::new(t0, t1)
            })
            .try_fold(ray_t, |x, y| Interval::intersect(&x, &y))
            .is_some()
    }

    pub fn longest_axis(&self) -> usize {
        let lx = self.x.size();
        let ly = self.y.size();
        let lz = self.z.size();

        if lx > ly {
            if lx > lz { 0 } else { 2 }
        } else if ly > lz {
            1
        } else {
            2
        }
    }

    pub fn union(&self, rhs: &AABB) -> AABB {
        AABB {
            x: Interval::union(&self.x, &rhs.x),
            y: Interval::union(&self.y, &rhs.y),
            z: Interval::union(&self.z, &rhs.z),
        }
    }

    pub const EMPTY: AABB = AABB {
        x: Interval::EMPTY,
        y: Interval::EMPTY,
        z: Interval::EMPTY,
    };
    pub const UNIVERSE: AABB = AABB {
        x: Interval::UNIVERSE,
        y: Interval::UNIVERSE,
        z: Interval::UNIVERSE,
    };
}
#[cfg(test)]
mod tests {
    use crate::utils::vec3::Vec3;

    use super::*;

    #[test]
    fn test_new_and_axis_interval() {
        let x = Interval::new(1.0, 2.0);
        let y = Interval::new(3.0, 4.0);
        let z = Interval::new(5.0, 6.0);
        let aabb = AABB::new(x, y, z);

        assert_eq!(aabb.axis_interval(0), &x);
        assert_eq!(aabb.axis_interval(1), &y);
        assert_eq!(aabb.axis_interval(2), &z);
    }

    #[test]
    #[should_panic]
    fn test_axis_interval_panic() {
        let aabb = AABB::default();
        aabb.axis_interval(3);
    }

    #[test]
    fn test_from_points() {
        let a = Point3::new(1.0, 2.0, 3.0);
        let b = Point3::new(4.0, 5.0, 6.0);
        let aabb = AABB::from_points(a, b);

        assert_eq!(aabb.x.min(), &1.0);
        assert_eq!(aabb.x.max(), &4.0);
        assert_eq!(aabb.y.min(), &2.0);
        assert_eq!(aabb.y.max(), &5.0);
        assert_eq!(aabb.z.min(), &3.0);
        assert_eq!(aabb.z.max(), &6.0);
    }

    #[test]
    fn test_hit_inside() {
        let aabb = AABB::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        let ray = Ray::new(Point3::new(0.5, 0.5, -1.0), Vec3::new(0.0, 0.0, 1.0));
        let interval = Interval::new(0.0, 100.0);
        assert!(aabb.hit(&ray, interval));
    }

    #[test]
    fn test_hit_outside() {
        let aabb = AABB::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        let ray = Ray::new(Point3::new(2.0, 2.0, 2.0), Vec3::new(1.0, 0.0, 0.0));
        let interval = Interval::new(0.0, 100.0);
        assert!(!aabb.hit(&ray, interval));
    }

    #[test]
    fn test_longest_axis() {
        let aabb = AABB::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 1.0, 1.0));
        assert_eq!(aabb.longest_axis(), 0);

        let aabb = AABB::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 3.0, 1.0));
        assert_eq!(aabb.longest_axis(), 1);

        let aabb = AABB::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 4.0));
        assert_eq!(aabb.longest_axis(), 2);
    }

    #[test]
    fn test_union() {
        let aabb1 = AABB::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        let aabb2 = AABB::from_points(Point3::new(1.0, 1.0, 1.0), Point3::new(2.0, 2.0, 2.0));
        let union = aabb1.union(&aabb2);

        assert_eq!(union.x.min(), &0.0);
        assert_eq!(union.x.max(), &2.0);
        assert_eq!(union.y.min(), &0.0);
        assert_eq!(union.y.max(), &2.0);
        assert_eq!(union.z.min(), &0.0);
        assert_eq!(union.z.max(), &2.0);
    }

    #[test]
    fn test_empty_and_universe() {
        let universe = AABB::UNIVERSE;
        assert!(universe.x.contains(0.0));
        assert!(universe.y.contains(1e10));
        assert!(universe.z.contains(-1e10));
    }
}
