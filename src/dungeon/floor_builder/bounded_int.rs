use crate::dungeon::floor_builder::bounded_int::iter::{BoundedIntRange, BoundedIntRangeInclusive};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

mod iter;
mod ops;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BoundedInt</* T: Integer,  */ const LOW: i32, const HIGH: i32>(pub(crate) i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BoundedIntError {
    TooHigh(i32),
    TooLow(i32),
}

impl<const LOW: i32, const HIGH: i32> BoundedInt<{ LOW }, { HIGH }> {
    pub const LOW: i32 = LOW;
    pub const HIGH: i32 = HIGH;

    pub fn new_clamped(n: i32) -> Self {
        assert!(LOW < HIGH);
        BoundedInt(n.min(Self::HIGH).max(Self::LOW))
    }

    pub fn new(n: i32) -> Result<Self, BoundedIntError> {
        assert!(LOW < HIGH);
        match n {
            n if n < Self::LOW => Err(BoundedIntError::TooLow(n)),
            n if n > Self::HIGH => Err(BoundedIntError::TooHigh(n)),
            n => Ok(BoundedInt(n)),
        }
    }

    pub fn as_unbounded(&self) -> i32 {
        self.0
    }

    pub fn range_to(&self, to: &Self) -> BoundedIntRange<{ LOW }, { HIGH }> {
        BoundedIntRange {
            end: *to,
            pointer: *self,
        }
    }

    pub fn range_from(&self, from: &Self) -> BoundedIntRange<{ LOW }, { HIGH }> {
        BoundedIntRange {
            end: *self,
            pointer: *from,
        }
    }

    pub fn range_to_inclusive(&self, to: &Self) -> BoundedIntRangeInclusive<{ LOW }, { HIGH }> {
        BoundedIntRangeInclusive {
            end: *to,
            pointer: *self,
            finished: false,
        }
    }

    pub fn range_from_inclusive(&self, from: &Self) -> BoundedIntRangeInclusive<{ LOW }, { HIGH }> {
        BoundedIntRangeInclusive {
            end: *self,
            pointer: *from,
            finished: false,
        }
    }

    pub fn expand_upper<const HIGHER: i32>(self) -> BoundedInt<{ LOW }, { HIGHER }> {
        assert!(HIGHER > HIGH);
        BoundedInt(self.0)
    }

    pub fn expand_lower<const LOWER: i32>(self) -> BoundedInt<{ LOWER }, { HIGH }> {
        assert!(LOWER < LOW);
        BoundedInt(self.0)
    }

    pub fn saturating_sub(&self, rhs: i32) -> Self {
        Self::new_clamped(self.0 - rhs)
    }

    pub fn saturating_add(&self, rhs: i32) -> Self {
        Self::new_clamped(self.0 + rhs)
    }
}

impl<const LOW: i32, const HIGH: i32> TryFrom<i32> for BoundedInt<{ LOW }, { HIGH }> {
    type Error = BoundedIntError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        assert!(LOW < HIGH);
        match value {
            n if n < Self::LOW => Err(BoundedIntError::TooLow(n)),
            n if n > Self::HIGH => Err(BoundedIntError::TooHigh(n)),
            n => Ok(BoundedInt(n)),
        }
    }
}

#[cfg(test)]
mod test_bounded_int {
    use super::*;

    #[test]
    fn test_range_to() {
        let start = BoundedInt::<0, 100>::new(20).unwrap();
        let end = BoundedInt::<0, 100>::new(25).unwrap();

        let v: Vec<_> = start.range_to(&end).collect();

        assert!(matches!(
            *v,
            [
                BoundedInt(20),
                BoundedInt(21),
                BoundedInt(22),
                BoundedInt(23),
                BoundedInt(24)
            ]
        ));
    }

    #[test]
    fn test_range_to_inclusive() {
        let start = BoundedInt::<0, 25>::new(20).unwrap();
        let end = BoundedInt::<0, 25>::new(25).unwrap();

        let v: Vec<_> = start.range_to_inclusive(&end).collect();

        assert!(matches!(
            *v,
            [
                BoundedInt(20),
                BoundedInt(21),
                BoundedInt(22),
                BoundedInt(23),
                BoundedInt(24),
                BoundedInt(25)
            ]
        ));
    }

    #[test]
    fn test_range_from() {
        let start = BoundedInt::<0, 100>::new(20).unwrap();
        let end = BoundedInt::<0, 100>::new(25).unwrap();

        let v: Vec<_> = end.range_from(&start).collect();

        assert!(matches!(
            *dbg!(v),
            [
                BoundedInt(20),
                BoundedInt(21),
                BoundedInt(22),
                BoundedInt(23),
                BoundedInt(24)
            ]
        ));
    }

    #[test]
    fn test_range_from_inclusive() {
        let start = BoundedInt::<0, 25>::new(20).unwrap();
        let end = BoundedInt::<0, 25>::new(25).unwrap();

        let v: Vec<_> = end.range_from_inclusive(&start).collect();

        assert!(matches!(
            *v,
            [
                BoundedInt(20),
                BoundedInt(21),
                BoundedInt(22),
                BoundedInt(23),
                BoundedInt(24),
                BoundedInt(25)
            ]
        ));
    }
}

// https://github.com/rust-lang/rust/issues/76560
// big sad

// pub struct Assert<const L: usize, const R: usize>;
// impl<const L: usize, const R: usize> Assert<L, R> {
//     pub const GREATER_EQ: usize = L - R;
//     pub const LESS_EQ: usize = R - L;
//     pub const NOT_EQ: isize = 0 / (R as isize - L as isize);
//     pub const EQ: usize = (R - L) + (L - R);
//     pub const GREATER: usize = L - R - 1;
//     pub const LESS: usize = R - L - 1;
// }

// #[allow(path_statements)]
// pub fn greater_than_0<const N: usize>() {
//     Assert::<N, 0>::GREATER;
// }

// pub enum Bool<const CHECK: bool> {}

// pub trait IsValid {}

// impl IsValid for Bool<true> {}
