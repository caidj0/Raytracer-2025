use crate::utils::vec3::Vec3;

pub type Color = Vec3;

impl Color {
    pub fn to_rgb(&self) -> [u8; 3] {
        self.e.map(|x| -> u8 {(255.999 * x) as u8})
    }
}

