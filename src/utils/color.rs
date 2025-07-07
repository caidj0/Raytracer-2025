use palette::{LinSrgb, Srgb};

use crate::utils::vec3::Vec3;

pub type Color = Vec3;

#[derive(Debug)]
pub enum ToonMap {
    None,
    ACES,
}

impl Color {
    fn aces_tonemap(&self) -> Color {
        const A: f64 = 2.51;
        const B: Vec3 = Vec3::new(0.03, 0.03, 0.03);
        const C: f64 = 2.43;
        const D: Vec3 = Vec3::new(0.59, 0.59, 0.59);
        const E: Vec3 = Vec3::new(0.14, 0.14, 0.14);
        Vec3::clamp(
            (self * (A * self + B)) / (self * (C * self + D) + E),
            0.0,
            1.0,
        )
    }

    pub fn to_rgb(&self, toon_map: &ToonMap) -> [u8; 3] {
        assert!(!self.e().iter().any(|&x| x.is_nan()));

        let mapped_color = match toon_map {
            ToonMap::None => *self,
            ToonMap::ACES => self.aces_tonemap(),
        };

        Srgb::from_linear(LinSrgb::from(mapped_color.e())).into()
    }

    pub fn luminance(&self) -> f64 {
        0.2126 * self.x() + 0.7152 * self.y() + 0.0722 * self.z()
    }

    pub const BLACK: Color = Color::new(0.0, 0.0, 0.0);
    pub const WHITE: Color = Color::new(1.0, 1.0, 1.0);
    pub const BLUE: Color = Color::new(0.0, 0.0, 1.0);
    pub const RED: Color = Color::new(1.0, 0.0, 0.0);
}
