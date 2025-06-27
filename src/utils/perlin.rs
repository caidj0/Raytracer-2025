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
        let i = (4.0 * p.x()) as i32 & 255;
        let j = (4.0 * p.y()) as i32 & 255;
        let k = (4.0 * p.z()) as i32 & 255;

        self.randfloat[self.perm_x[i as usize] ^ self.perm_y[j as usize] ^ self.perm_z[k as usize]]
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
