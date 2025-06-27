use std::iter::repeat_with;

use crate::utils::{
    random::Random,
    vec3::{Point3, UnitVec3, Vec3},
};

pub struct Perlin {
    randvec: [UnitVec3; Perlin::POINT_COUNT],
    perm_x: [usize; Perlin::POINT_COUNT],
    perm_y: [usize; Perlin::POINT_COUNT],
    perm_z: [usize; Perlin::POINT_COUNT],
}

impl Default for Perlin {
    fn default() -> Self {
        let randvec: [UnitVec3; Perlin::POINT_COUNT] = repeat_with(UnitVec3::random_unit_vector)
            .take(Perlin::POINT_COUNT)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let perm_x = Perlin::perlin_generate_perm();
        let perm_y = Perlin::perlin_generate_perm();
        let perm_z = Perlin::perlin_generate_perm();

        Perlin {
            randvec,
            perm_x,
            perm_y,
            perm_z,
        }
    }
}

impl Perlin {
    const POINT_COUNT: usize = 256;

    pub fn noise(&self, p: &Point3) -> f64 {
        let (i, j, k) = p.e().map(|x| x.floor() as i64).into();

        let (u, v, w) = p.e().map(|x| x - x.floor()).into();

        let mut c: [[[UnitVec3; 2]; 2]; 2] = Default::default();

        for (di, item_i) in c.iter_mut().enumerate() {
            for (dj, item_j) in item_i.iter_mut().enumerate() {
                for (dk, item) in item_j.iter_mut().enumerate() {
                    *item = self.randvec[self.perm_x[(i + di as i64) as usize & 255]
                        ^ self.perm_y[(j + dj as i64) as usize & 255]
                        ^ self.perm_z[(k + dk as i64) as usize & 255]];
                }
            }
        }

        Perlin::perlin_interp(&c, u, v, w)
    }

    fn perlin_interp(c: &[[[UnitVec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let (uu, vv, ww) = [u, v, w].map(|x| x * x * (3.0 - 2.0 * x)).into();
        let mut accum = 0.0;

        for (i, item_i) in c.iter().enumerate() {
            for (j, item_j) in item_i.iter().enumerate() {
                for (k, item) in item_j.iter().enumerate() {
                    let weight_v = Vec3::new(u - i as f64, v - j as f64, w - k as f64);
                    accum += (i as f64 * uu + (1 - i) as f64 * (1.0 - uu))
                        * (j as f64 * vv + (1 - j) as f64 * (1.0 - vv))
                        * (k as f64 * ww + (1 - k) as f64 * (1.0 - ww))
                        * item.dot(&weight_v);
                }
            }
        }

        accum
    }

    fn perlin_generate_perm() -> [usize; Perlin::POINT_COUNT] {
        let mut p: [usize; Perlin::POINT_COUNT] = (0..Perlin::POINT_COUNT)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        Perlin::permute(&mut p, Perlin::POINT_COUNT);

        p
    }

    fn permute(p: &mut [usize; Perlin::POINT_COUNT], n: usize) {
        for i in (1..n).rev() {
            let target = Random::usize(0..=i);
            p.swap(i, target);
        }
    }
}
