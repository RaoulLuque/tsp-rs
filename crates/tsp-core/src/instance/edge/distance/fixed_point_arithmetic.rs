use std::{
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
};

use super::Distance;

pub(crate) const FIXED_POINT_FRACTIONAL_BITS: u32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ScaledDistance(pub i32);

impl ScaledDistance {
    pub const MAX: ScaledDistance = ScaledDistance(i32::MAX);
    pub const MIN: ScaledDistance = ScaledDistance(i32::MIN);
}

impl ScaledDistance {
    /// TODO: Ensure that value does not overflow when shifted?
    pub fn from_i32(value: i32) -> Self {
        ScaledDistance(value << FIXED_POINT_FRACTIONAL_BITS)
    }

    /// TODO: Ensure that value does not overflow when shifted?
    pub fn from_distance(value: Distance) -> Self {
        ScaledDistance(value.0 << FIXED_POINT_FRACTIONAL_BITS)
    }

    pub fn to_distance(self) -> Distance {
        Distance(self.0 >> FIXED_POINT_FRACTIONAL_BITS)
    }

    pub fn to_distance_rounded_up(self) -> Distance {
        let adjusted = self.0 + (1 << FIXED_POINT_FRACTIONAL_BITS) - 1;
        Distance(adjusted >> FIXED_POINT_FRACTIONAL_BITS)
    }
}

impl From<i32> for ScaledDistance {
    fn from(value: i32) -> Self {
        ScaledDistance::from_i32(value)
    }
}

impl From<Distance> for ScaledDistance {
    fn from(value: Distance) -> Self {
        ScaledDistance::from_distance(value)
    }
}

impl Add for ScaledDistance {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        ScaledDistance(self.0 + other.0)
    }
}

impl Sub for ScaledDistance {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        ScaledDistance(self.0 - other.0)
    }
}

impl Div<i32> for ScaledDistance {
    type Output = ScaledDistance;

    fn div(self, rhs: i32) -> Self::Output {
        ScaledDistance(self.0 / rhs)
    }
}

impl<'a> Sum<&'a ScaledDistance> for ScaledDistance {
    fn sum<I: Iterator<Item = &'a ScaledDistance>>(iter: I) -> Self {
        iter.fold(ScaledDistance(0), |acc, d| acc + *d)
    }
}

impl Mul<ScaledDistance> for i32 {
    type Output = ScaledDistance;

    fn mul(self, rhs: ScaledDistance) -> Self::Output {
        ScaledDistance(self * rhs.0)
    }
}

impl AddAssign for ScaledDistance {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl SubAssign for ScaledDistance {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}
