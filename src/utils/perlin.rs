use crate::utils::{random::Random, vec3::Point3};

pub struct Perlin {
    randfloat: [f64; Perlin::POINT_COUNT],
    perm_x: [usize; Perlin::POINT_COUNT],
    perm_y: [usize; Perlin::POINT_COUNT],
    perm_z: [usize; Perlin::POINT_COUNT],
}

impl Default for Perlin {
    fn default() -> Self {
        let randfloat: [f64; Perlin::POINT_COUNT] = (0..Perlin::POINT_COUNT)
            .map(|_| Random::f64())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let perm_x = Perlin::perlin_generate_perm();
        let perm_y = Perlin::perlin_generate_perm();
        let perm_z = Perlin::perlin_generate_perm();

        Perlin {
            randfloat,
            perm_x,
            perm_y,
            perm_z,
        }
    }
}

impl Perlin {
    const POINT_COUNT: usize = 256;

    pub fn noise(&self, p: &Point3) -> f64 {
        let i = p.x().floor() as i64;
        let j = p.y().floor() as i64;
        let k = p.z().floor() as i64;

        let (u, v, w) = p
            .e()
            .map(|x| x - x.floor())
            .map(|x| x * x * (3.0 - 2.0 * x))
            .into();

        let mut c: [[[f64; 2]; 2]; 2] = Default::default();

        for (di, item_i) in c.iter_mut().enumerate() {
            for (dj, item_j) in item_i.iter_mut().enumerate() {
                for (dk, item) in item_j.iter_mut().enumerate() {
                    *item = self.randfloat[self.perm_x[(i + di as i64) as usize & 255]
                        ^ self.perm_y[(j + dj as i64) as usize & 255]
                        ^ self.perm_z[(k + dk as i64) as usize & 255]];
                }
            }
        }

        Perlin::trilinear_interp(&c, u, v, w)
    }

    fn trilinear_interp(c: &[[[f64; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let mut accum = 0.0;
        for (i, item_i) in c.iter().enumerate() {
            for (j, item_j) in item_i.iter().enumerate() {
                for (k, item) in item_j.iter().enumerate() {
                    accum += (i as f64 * u + (1 - i) as f64 * (1.0 - u))
                        * (j as f64 * v + (1 - j) as f64 * (1.0 - v))
                        * (k as f64 * w + (1 - k) as f64 * (1.0 - w))
                        * item;
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
