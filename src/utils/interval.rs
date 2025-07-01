use std::ops::{Add, Range};

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct Interval {
    min: f64,
    max: f64,
}

impl Interval {
    pub const fn new(a: f64, b: f64) -> Interval {
        Interval {
            min: f64::min(a, b),
            max: f64::max(a, b),
        }
    }

    pub fn from_range(range: Range<f64>) -> Interval {
        Interval {
            min: range.start,
            max: range.end,
        }
    }

    pub fn clamp(&self, x: f64) -> f64 {
        x.clamp(self.min, self.max)
    }

    pub fn expand(&self, delta: f64) -> Interval {
        let padding = delta / 2.0;
        Interval {
            min: self.min - padding,
            max: self.max + padding,
        }
    }

    pub fn min(&self) -> &f64 {
        &self.min
    }

    pub fn max(&self) -> &f64 {
        &self.max
    }

    pub fn size(&self) -> f64 {
        f64::max(self.max - self.min, 0.0)
    }

    pub fn intersect(&self, rhs: &Interval) -> Option<Interval> {
        let max = f64::min(self.max, rhs.max);
        let min = f64::max(self.min, rhs.min);
        if min <= max {
            Some(Interval { min, max })
        } else {
            None
        }
    }

    pub fn union(self, rhs: Interval) -> Interval {
        Interval {
            min: f64::min(self.min, rhs.min),
            max: f64::max(self.max, rhs.max),
        }
    }

    pub fn contains(&self, x: f64) -> bool {
        x >= self.min && x <= self.max
    }

    pub const EMPTY: Interval = Interval {
        min: f64::INFINITY,
        max: -f64::INFINITY,
    };
    pub const UNIVERSE: Interval = Interval {
        min: -f64::INFINITY,
        max: f64::INFINITY,
    };
}

impl Add<f64> for Interval {
    type Output = Interval;

    fn add(self, displacement: f64) -> Self::Output {
        Interval {
            min: self.min + displacement,
            max: self.max + displacement,
        }
    }
}

impl Add<Interval> for f64 {
    type Output = Interval;

    fn add(self, displacement: Interval) -> Self::Output {
        Interval {
            min: displacement.min + self,
            max: displacement.max + self,
        }
    }
}
