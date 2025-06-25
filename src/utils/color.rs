use crate::utils::vec3::Vec3;

pub type Color = Vec3;

impl Color {
    pub fn to_rgb(&self) -> [u8; 3] {
        self.e()
            .map(|x| -> u8 { (256.0 * Color::linear_to_gamma(x).clamp(0.0, 0.999)) as u8 })
    }

    fn linear_to_gamma(linear_component: f64) -> f64 {
        if linear_component > 0.0 {
            linear_component.sqrt()
        } else {
            0.0
        }
    }

    pub const BLACK: Color = Color::new(0.0, 0.0, 0.0);
    pub const WHITE: Color = Color::new(1.0, 1.0, 1.0);
}
