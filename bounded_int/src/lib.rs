#![warn(missing_docs, clippy::missing_docs_in_private_items)]

//! Leverages const generics to enforce compile time bounds on integers.
//!
//! **Note:** due to current limitations in Rust, the [`BoundedInt`] type uses
//! an [`i32`] behind the scenes. This could be wasteful if you limit the bounds
//! to smaller than, say, an [`i16`], or may not be enough if you need a number
//! larger than an [`i32`] can hold.

pub use crate::iter::{BoundedIntRange, BoundedIntRangeInclusive};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

/// Contains the [`Iterator`] implementations for the range types.
pub mod iter;
/// Contains the various [`std::ops`] trait implementations for [`BoundedInt`].
pub mod ops;

/// An integer bound between two points, inclusive on both ends.
///
/// # Examples
///
/// ```rust
/// use std::ops::{Add, Sub};
/// use bounded_int::{BoundedInt, BoundedIntError};
///
/// let b = BoundedInt::<-10, 10>::new(0).unwrap();
///
/// assert_eq!(b.add(5).unwrap(), BoundedInt::<-10, 10>::new(5).unwrap());
///
/// assert_eq!(b.sub(5).unwrap(), BoundedInt::<-10, 10>::new(-5).unwrap());
///
/// assert_eq!(
///     b.saturating_add(100),
///     BoundedInt::<-10, 10>::new(10).unwrap()
/// );
///
/// assert_eq!(
///     b.saturating_sub(100),
///     BoundedInt::<-10, 10>::new(-10).unwrap()
/// );
///
/// let invalid = BoundedInt::<-10, 10>::new(100);
///
/// assert_eq!(invalid, Err(BoundedIntError::TooHigh(100)));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BoundedInt</* T: Integer, */ const LOW: i32, const HIGH: i32>(pub(crate) i32);

/// Error returned when trying to convert an [`i32`] to a [`BoundedInt`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BoundedIntError {
    /// The number was too high for the bounds.
    ///
    /// Contains the value that was too high.
    ///
    /// # Examples
    /// ```rust
    /// use bounded_int::{BoundedInt, BoundedIntError};
    ///
    /// let too_high = BoundedInt::<0, 10>::new(15);
    ///
    /// assert_eq!(too_high, Err(BoundedIntError::TooHigh(15)));
    ///```
    TooHigh(i32),

    /// The number was too low for the bounds.
    ///
    /// Contains the value that was too low.
    ///
    /// # Examples
    /// ```rust
    /// use bounded_int::{BoundedInt, BoundedIntError};
    ///
    /// let too_low = BoundedInt::<20, 30>::new(15);
    ///
    /// assert_eq!(too_low, Err(BoundedIntError::TooLow(15)));
    ///```
    TooLow(i32),
}

// TODO: Move assertions to where clause once `const_evaluatable_checked` is
// stabilized
impl<const LOW: i32, const HIGH: i32> BoundedInt<{ LOW }, { HIGH }> {
    /// The lower bounds of the `BoundedInt`, inclusive.
    pub const LOW: i32 = LOW;
    /// The upper bounds of the `BoundedInt`, inclusive.
    pub const HIGH: i32 = HIGH;

    /// Returns the input as a [`BoundedInt`], clamped at the bounds.
    ///
    /// # Examples
    /// ```rust
    /// use bounded_int::{BoundedInt, BoundedIntError};
    ///
    /// let clamped = BoundedInt::<20, 30>::new_clamped(15);
    ///
    /// assert_eq!(clamped, BoundedInt::<20, 30>::new(20).unwrap());
    ///```
    #[must_use = "returned value will be immediately dropped if not used"]
    pub fn new_clamped(n: i32) -> Self {
        assert!(
            LOW < HIGH,
            "LOW must be less than HIGH. LOW: {}, HIGH: {}",
            LOW,
            HIGH
        );
        Self(n.min(Self::HIGH).max(Self::LOW))
    }

    /// Attempts to create a new [`BoundedInt`] with the provided value.
    ///
    /// # Errors
    /// This function will error if the supplied value doesn't fit within
    /// the bounds required.
    pub fn new(n: i32) -> Result<Self, BoundedIntError> {
        assert!(
            LOW < HIGH,
            "LOW must be less than HIGH. LOW: {}, HIGH: {}",
            LOW,
            HIGH
        );

        n.try_into()
    }

    /// Returns the inner value of the [`BoundedInt`].
    #[inline]
    #[must_use = "`as_unbounded` does not mutate the original value"]
    pub const fn as_unbounded(self) -> i32 {
        self.0
    }

    /// Returns a [`BoundedIntRange`] from `self` to `to`.
    ///
    /// If `to` <= `self`, the iterator will be empty.
    ///
    /// # Examples
    /// ```rust
    /// use bounded_int::BoundedInt;
    ///
    /// let start = BoundedInt::<10, 15>::new(10).unwrap();
    /// let end = BoundedInt::<10, 15>::new(15).unwrap();
    ///
    /// assert_eq!(
    ///     start.range_to(end).collect::<Vec<_>>(),
    ///     &[
    ///         BoundedInt::<10, 15>::new(10).unwrap(),
    ///         BoundedInt::<10, 15>::new(11).unwrap(),
    ///         BoundedInt::<10, 15>::new(12).unwrap(),
    ///         BoundedInt::<10, 15>::new(13).unwrap(),
    ///         BoundedInt::<10, 15>::new(14).unwrap(),
    ///     ]
    /// );
    /// ```
    pub const fn range_to(self, to: Self) -> BoundedIntRange<{ LOW }, { HIGH }> {
        BoundedIntRange {
            end: to,
            pointer: self,
        }
    }

    /// Returns a [`BoundedIntRange`] from `from` to `self`.
    ///
    /// If `self` <= `from`, the iterator will be empty.
    ///
    /// # Examples
    /// ```rust
    /// use bounded_int::BoundedInt;
    ///
    /// let start = BoundedInt::<10, 15>::new(10).unwrap();
    /// let end = BoundedInt::<10, 15>::new(15).unwrap();
    ///
    /// assert_eq!(
    ///     end.range_from(start).collect::<Vec<_>>(),
    ///     &[
    ///         BoundedInt::<10, 15>::new(10).unwrap(),
    ///         BoundedInt::<10, 15>::new(11).unwrap(),
    ///         BoundedInt::<10, 15>::new(12).unwrap(),
    ///         BoundedInt::<10, 15>::new(13).unwrap(),
    ///         BoundedInt::<10, 15>::new(14).unwrap(),
    ///     ]
    /// );
    /// ```
    pub const fn range_from(self, from: Self) -> BoundedIntRange<{ LOW }, { HIGH }> {
        BoundedIntRange {
            end: self,
            pointer: from,
        }
    }

    /// Returns a [`BoundedIntRangeInclusive`] from `self` to `to`.
    ///
    /// If `to` <= `self`, the iterator will be empty.
    /// If `to` == `self`, the iterator will produce one item equal to `self`.
    ///
    /// # Examples
    /// ```rust
    /// use bounded_int::BoundedInt;
    ///
    /// let start = BoundedInt::<10, 15>::new(10).unwrap();
    /// let end = BoundedInt::<10, 15>::new(15).unwrap();
    ///
    /// assert_eq!(
    ///     start.range_to_inclusive(end).collect::<Vec<_>>(),
    ///     &[
    ///         BoundedInt::<10, 15>::new(10).unwrap(),
    ///         BoundedInt::<10, 15>::new(11).unwrap(),
    ///         BoundedInt::<10, 15>::new(12).unwrap(),
    ///         BoundedInt::<10, 15>::new(13).unwrap(),
    ///         BoundedInt::<10, 15>::new(14).unwrap(),
    ///         BoundedInt::<10, 15>::new(15).unwrap(),
    ///     ]
    /// );
    /// ```
    pub const fn range_to_inclusive(self, to: Self) -> BoundedIntRangeInclusive<{ LOW }, { HIGH }> {
        BoundedIntRangeInclusive {
            end: to,
            pointer: self,
            finished: false,
        }
    }

    /// Returns a [`BoundedIntRangeInclusive`] from `from` to `self`.
    ///
    /// If `self` < `from`, the iterator will be empty.
    /// If `self` == `from`, the iterator will produce one item equal to `self`.
    ///
    /// # Examples
    /// ```rust
    /// use bounded_int::BoundedInt;
    ///
    /// let start = BoundedInt::<10, 15>::new(10).unwrap();
    /// let end = BoundedInt::<10, 15>::new(15).unwrap();
    ///
    /// assert_eq!(
    ///     end.range_from_inclusive(start).collect::<Vec<_>>(),
    ///     &[
    ///         BoundedInt::<10, 15>::new(10).unwrap(),
    ///         BoundedInt::<10, 15>::new(11).unwrap(),
    ///         BoundedInt::<10, 15>::new(12).unwrap(),
    ///         BoundedInt::<10, 15>::new(13).unwrap(),
    ///         BoundedInt::<10, 15>::new(14).unwrap(),
    ///         BoundedInt::<10, 15>::new(15).unwrap(),
    ///     ]
    /// );
    /// ```
    pub const fn range_from_inclusive(
        self,
        from: Self,
    ) -> BoundedIntRangeInclusive<{ LOW }, { HIGH }> {
        BoundedIntRangeInclusive {
            end: self,
            pointer: from,
            finished: false,
        }
    }

    /// Raises the upper bounds of the `BoundedInt` to `HIGHER`.
    /// # Examples
    /// ```rust
    /// use bounded_int::BoundedInt;
    ///
    /// let small_bounds = BoundedInt::<10, 15>::new(10).unwrap();
    ///
    /// fn requires_larger_bounds(b: BoundedInt::<10, 20>) { }
    ///
    /// // type inference makes this very simple to call
    /// requires_larger_bounds(small_bounds.expand_upper());
    /// ```
    #[must_use = "`expand_upper` does not mutate the original value"]
    pub fn expand_upper<const HIGHER: i32>(self) -> BoundedInt<{ LOW }, { HIGHER }> {
        assert!(
            HIGHER >= HIGH,
            "HIGHER must be greater than or equal to HIGH. HIGHER: {}, HIGH: {}",
            HIGHER,
            HIGH
        );
        BoundedInt(self.0)
    }

    /// Increases the lower bounds of the `BoundedInt` to `LOWER`.
    /// # Examples
    /// ```rust
    /// use bounded_int::BoundedInt;
    ///
    /// let small_bounds = BoundedInt::<10, 15>::new(10).unwrap();
    ///
    /// fn requires_larger_bounds(b: BoundedInt::<5, 15>) { }
    ///
    /// // type inference makes this very simple to call
    /// requires_larger_bounds(small_bounds.expand_lower());
    /// ```
    #[must_use = "`expand_lower` does not mutate the original value"]
    pub fn expand_lower<const LOWER: i32>(self) -> BoundedInt<{ LOWER }, { HIGH }> {
        assert!(
            LOWER <= LOW,
            "LOWER must be less than or equal to LOW. LOWER: {}, LOW: {}",
            LOWER,
            LOW
        );
        BoundedInt(self.0)
    }

    /// Expands the bounds of the `BoundedInt` to `LOWER` and `HIGHER`.
    /// # Examples
    /// ```rust
    /// use bounded_int::BoundedInt;
    ///
    /// let small_bounds = BoundedInt::<10, 15>::new(10).unwrap();
    ///
    /// fn requires_larger_bounds(b: BoundedInt::<5, 20>) { }
    ///
    /// // type inference makes this very simple to call
    /// requires_larger_bounds(small_bounds.expand_bounds());
    /// ```
    #[must_use = "`expand_bounds` does not mutate the original value"]
    pub fn expand_bounds<const LOWER: i32, const HIGHER: i32>(
        self,
    ) -> BoundedInt<{ LOWER }, { HIGHER }> {
        assert!(
            LOWER <= LOW,
            "LOWER must be less than or equal to LOW. LOWER: {}, LOW: {}",
            LOWER,
            LOW
        );
        assert!(
            HIGHER >= HIGH,
            "HIGHER must be greater than or equal to HIGH. HIGHER: {}, HIGH: {}",
            HIGHER,
            HIGH
        );
        assert!(
            LOWER < HIGHER,
            "LOWER must be less than HIGHER. LOWER: {}, HIGHER: {}",
            LOWER,
            HIGHER
        );
        BoundedInt(self.0)
    }

    /// Performs subtraction that saturates at the numeric bounds instead of
    /// overflowing.
    #[must_use = "`saturating_sub` does not mutate the original value"]
    pub fn saturating_sub(self, rhs: i32) -> Self {
        Self::new_clamped(self.0 - rhs)
    }

    /// Performs addition that saturates at the numeric bounds instead of
    /// overflowing.
    #[must_use = "`saturating_add` does not mutate the original value"]
    pub fn saturating_add(self, rhs: i32) -> Self {
        Self::new_clamped(self.0 + rhs)
    }
}

impl<const LOW: i32, const HIGH: i32> TryFrom<i32> for BoundedInt<{ LOW }, { HIGH }> {
    type Error = BoundedIntError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        assert!(
            LOW < HIGH,
            "LOW must be less than HIGH. LOW: {}, HIGH: {}",
            LOW,
            HIGH
        );
        if value < Self::LOW {
            Err(BoundedIntError::TooLow(value))
        } else if value > Self::HIGH {
            Err(BoundedIntError::TooHigh(value))
        } else {
            Ok(Self(value))
        }
    }
}

#[cfg(test)]
mod test_bounded_int {
    use super::*;

    #[test]
    fn test_range_to() {
        let start = BoundedInt::<20, 25>::new(20).unwrap();
        let end = BoundedInt::<20, 25>::new(25).unwrap();

        let v: Vec<_> = start.range_to(end).collect();

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

        let v: Vec<_> = start.range_to_inclusive(end).collect();

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

        let v: Vec<_> = end.range_from(start).collect();

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
    fn test_range_from_inclusive() {
        let start = BoundedInt::<0, 25>::new(20).unwrap();
        let end = BoundedInt::<0, 25>::new(25).unwrap();

        let v: Vec<_> = end.range_from_inclusive(start).collect();

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
