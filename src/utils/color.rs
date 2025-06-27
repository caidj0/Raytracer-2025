use palette::{LinSrgb, Srgb};

use crate::utils::vec3::Vec3;

pub type Color = Vec3;

impl Color {
    pub fn to_rgb(&self) -> [u8; 3] {
        Srgb::from_linear(LinSrgb::from(self.e())).into()
    }

    pub const BLACK: Color = Color::new(0.0, 0.0, 0.0);
    pub const WHITE: Color = Color::new(1.0, 1.0, 1.0);
    pub const BLUE: Color = Color::new(0.0, 0.0, 1.0);
    pub const RED: Color = Color::new(1.0, 0.0, 0.0);
}
