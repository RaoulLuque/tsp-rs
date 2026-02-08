use std::{
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
};

use super::Distance;

pub(crate) const FIXED_POINT_FRACTIONAL_BITS: u32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ScaledDistance(pub i32);

impl ScaledDistance {
    /// The maximum representable scaled distance.
    pub const MAX: ScaledDistance = ScaledDistance(i32::MAX);
    /// The minimum representable scaled distance.
    pub const MIN: ScaledDistance = ScaledDistance(i32::MIN);
}

impl ScaledDistance {
    /// Creates a `ScaledDistance` from a `Distance` by scaling it.
    ///
    /// Panics if the input distance is negative or so large that scaling would cause overflow.
    pub fn from_distance(value: Distance) -> Self {
        // Benchmarks seem to suggest that these asserts are negligible.
        assert!(value.0 >= 0);
        assert!(value.0 <= (i32::MAX >> FIXED_POINT_FRACTIONAL_BITS));
        ScaledDistance(value.0 << FIXED_POINT_FRACTIONAL_BITS)
    }

    /// Converts the `ScaledDistance` to a `Distance` by truncating the fractional part.
    pub fn to_distance(self) -> Distance {
        Distance(self.0 >> FIXED_POINT_FRACTIONAL_BITS)
    }

    /// Converts the `ScaledDistance` to a `Distance`, rounding up to the nearest integer.
    pub fn to_distance_rounded_up(self) -> Distance {
        let adjusted = self.0 + (1 << FIXED_POINT_FRACTIONAL_BITS) - 1;
        Distance(adjusted >> FIXED_POINT_FRACTIONAL_BITS)
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
