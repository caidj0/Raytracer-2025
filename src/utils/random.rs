use std::ops::{Range, RangeInclusive};

pub struct Random;

impl Random {
    pub fn f64() -> f64 {
        rand::random()
    }

    pub fn random_range(interval: Range<f64>) -> f64 {
        rand::random_range(interval)
    }

    pub fn i32(interval: Range<i32>) -> i32 {
        rand::random_range(interval)
    }

    pub fn usize(interval: RangeInclusive<usize>) -> usize {
        rand::random_range(interval)
    }
}
