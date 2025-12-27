use std::{
    iter::Sum,
    ops::{Add, Mul, Sub},
};

mod fixed_point_arithmetic;
pub use fixed_point_arithmetic::ScaledDistance;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Distance(pub i32);

impl Distance {
    pub const MAX: Distance = Distance(i32::MAX);
    pub const MIN: Distance = Distance(i32::MIN);
}

impl Add for Distance {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Distance(self.0 + other.0)
    }
}

impl Sub for Distance {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Distance(self.0 - other.0)
    }
}

impl Sum<Distance> for Distance {
    fn sum<I: Iterator<Item = Distance>>(iter: I) -> Self {
        iter.fold(Distance(0), |acc, d| acc + d)
    }
}

impl<'a> Sum<&'a Distance> for Distance {
    fn sum<I: Iterator<Item = &'a Distance>>(iter: I) -> Self {
        iter.fold(Distance(0), |acc, d| acc + *d)
    }
}

impl Mul<Distance> for i32 {
    type Output = Distance;

    fn mul(self, rhs: Distance) -> Self::Output {
        Distance(self * rhs.0)
    }
}
