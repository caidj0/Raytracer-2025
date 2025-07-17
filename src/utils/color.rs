use crate::utils::vec3::Vec3;

pub type Color = Vec3;

impl Color {
    pub fn to_rgb(&self) -> [u8; 3] {
        self.e()
            .map(|x| -> u8 { (256.0 * x.clamp(0.0, 0.999)) as u8 })
    }
}
