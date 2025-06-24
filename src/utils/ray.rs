use crate::utils::vec3::{Point3, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    orig: Point3,
    dir: Vec3,
}

impl Default for Ray {
    fn default() -> Self {
        Self { orig: Default::default(), dir: Default::default() }
    }
}

impl Ray {
    pub fn new(origin: Point3, direction: Vec3) -> Ray {
        Ray { orig: origin, dir: direction }
    }

    pub fn origin(&self) -> &Point3 {
        &self.orig
    }

    pub fn direction(&self) -> &Vec3 {
        &self.dir
    }

    pub fn at(&self, t: f64) -> Point3 {
        self.orig + t * self.dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ray() {
        let origin = Point3::new(1.0, 2.0, 3.0);
        let direction = Vec3::new(4.0, 5.0, 6.0);
        let r = Ray::new(origin, direction);
        assert_eq!(*r.origin(), origin);
        assert_eq!(*r.direction(), direction);
    }

    #[test]
    fn test_ray_at() {
        let origin = Point3::new(1.0, 2.0, 3.0);
        let direction = Vec3::new(4.0, 5.0, 6.0);
        let r = Ray::new(origin, direction);
        let p = r.at(2.0);
        assert_eq!(p, Point3::new(9.0, 12.0, 15.0));
    }

    #[test]
    fn test_default_ray() {
        let r = Ray::default();
        assert_eq!(*r.origin(), Point3::default());
        assert_eq!(*r.direction(), Vec3::default());
    }
}