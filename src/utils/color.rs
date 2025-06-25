use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use crate::utils::vec3::Vec3;

pub struct Color(pub Vec3);

impl Deref for Color {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Color {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let to_255 = |x: f64| -> i32 { (255.999 * x) as i32 };
        write!(
            f,
            "{} {} {}",
            to_255(self[0]),
            to_255(self[1]),
            to_255(self[2])
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_display() {
        let black = Color(Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(format!("{}", black), "0 0 0");

        let white = Color(Vec3::new(1.0, 1.0, 1.0));
        assert_eq!(format!("{}", white), "255 255 255");

        let red = Color(Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(format!("{}", red), "255 0 0");

        let mixed = Color(Vec3::new(0.1, 0.5, 0.9));
        // 0.1 * 255.999 = 25.5999 -> 25
        // 0.5 * 255.999 = 127.9995 -> 127
        // 0.9 * 255.999 = 230.3991 -> 230
        assert_eq!(format!("{}", mixed), "25 127 230");
    }

    #[test]
    fn test_color_deref() {
        let mut color = Color(Vec3::new(0.2, 0.4, 0.6));
        assert_eq!(color.x(), 0.2);
        assert_eq!(color.y(), 0.4);
        assert_eq!(color.z(), 0.6);
        assert_eq!(color[0], 0.2);
        assert_eq!(color[1], 0.4);
        assert_eq!(color[2], 0.6);

        color[0] = 0.3;
        assert_eq!(color[0], 0.3);
    }
}
