use std::ops::{Add, Sub};

use bounded_int::{
    ops::{BoundedIntOverflow, BoundedIntUnderflow},
    BoundedInt,
};

use crate::floor_builder::MAX_FLOOR_SIZE;

/// A point somewhere in a [`Floor`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Point {
    /// How many columns across the point is.
    ///
    /// Corresponds to the width of the [`Floor`].
    pub column: Column,
    /// How many rows down the point is.
    ///
    /// Corresponds to the height of the [`Floor`].
    pub row: Row,
}

/// Wrapper type around a [`BoundedInt`] that represents the row position of a [`Point`].
/// Note that the [`BoundedInt`] is bound on the maximum and minimum that a point can be,
/// not the floor size itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Row(pub(super) BoundedInt<0, { MAX_FLOOR_SIZE }>);

/// Wrapper type around a [`BoundedInt`] that represents the column position of a [`Point`].
/// Note that the [`BoundedInt`] is bound on the maximum and minimum that a point can be,
/// not the floor size itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Column(pub(super) BoundedInt<0, { MAX_FLOOR_SIZE }>);

impl Point {
    /// Adds the provided value to the row, saturating on the numeric bounds instead of
    /// overflowing or panicking.
    pub fn saturating_add_row(self, n: i32) -> Self {
        Self {
            row: self.row.saturating_add(n),
            column: self.column,
        }
    }

    /// Subtracts the provided value from the row, saturating on the numeric bounds instead of
    /// overflowing or panicking.
    pub fn saturating_sub_row(self, n: i32) -> Self {
        Self {
            row: Row(self.row.0.saturating_sub(n)),
            column: self.column,
        }
    }

    /// Adds the provided value to the column, saturating on the numeric bounds instead of
    /// overflowing or panicking.
    pub fn saturating_add_column(self, n: i32) -> Self {
        Self {
            column: Column(self.column.0.saturating_add(n)),
            row: self.row,
        }
    }

    /// Subtracts the provided value from the column, saturating on the numeric bounds instead of
    /// overflowing or panicking.
    pub fn saturating_sub_column(self, n: i32) -> Self {
        Self {
            column: Column(self.column.0.saturating_sub(n)),
            row: self.row,
        }
    }

    /// Adds the provided value to the row, returning the error if it would overflow.
    pub fn add_row(self, n: u16) -> Result<Self, BoundedIntOverflow> {
        Ok(Self {
            row: Row(self.row.0.add(n)?),
            column: self.column,
        })
    }

    /// Subtracts the provided value from the row, returning the error if it would overflow.
    pub fn sub_row(self, n: u16) -> Result<Self, BoundedIntUnderflow> {
        Ok(Self {
            row: Row(self.row.0.sub(n)?),
            column: self.column,
        })
    }
    /// Adds the provided value to the column, returning the error if it would overflow.
    pub fn add_column(self, n: u16) -> Result<Self, BoundedIntOverflow> {
        Ok(Self {
            column: Column(self.column.0.add(n)?),
            row: self.row,
        })
    }

    /// Subtracts the provided value from the column, returning the error if it would overflow.
    pub fn sub_column(self, n: u16) -> Result<Self, BoundedIntUnderflow> {
        Ok(Self {
            column: Column(self.column.0.sub(n)?),
            row: self.row,
        })
    }
}

impl Add for Point {
    type Output = Option<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        let column = if let Ok(value) = self.column + rhs.column {
            value
        } else {
            return None;
        };

        let row = if let Ok(value) = self.row + rhs.row {
            value
        } else {
            return None;
        };

        Some(Self { column, row })
    }
}

impl Sub for Point {
    type Output = Option<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        let column = if let Ok(value) = self.column - rhs.column {
            value
        } else {
            return None;
        };

        let row = if let Ok(value) = self.row - rhs.row {
            value
        } else {
            return None;
        };

        Some(Self { column, row })
    }
}

macro_rules! impl_row_col {
    ($t:ty) => {
        impl $t {
            /// Creates a new [`$t`] with the provided value.
            pub fn new(
                value: ::bounded_int::BoundedInt<0, { crate::floor_builder::MAX_FLOOR_SIZE }>,
            ) -> Self {
                Self(value)
            }

            /// Returns the inner value.
            pub fn get(
                &self,
            ) -> ::bounded_int::BoundedInt<0, { crate::floor_builder::MAX_FLOOR_SIZE }> {
                self.0
            }

            /// Performs addition, saturating on the bounds instead of overflowing or panicking.
            pub fn saturating_add(self, n: i32) -> Self {
                Self(self.0.saturating_add(n))
            }

            /// Performs subtraction, saturating on the bounds instead of overflowing or panicking.
            pub fn saturating_sub(self, n: i32) -> Self {
                Self(self.0.saturating_sub(n))
            }
        }

        impl ::std::ops::Add<u16> for $t {
            type Output = ::std::result::Result<Self, ::bounded_int::ops::BoundedIntOverflow>;

            fn add(self, rhs: u16) -> Self::Output {
                Ok(Self((self.0.add(rhs))?))
            }
        }

        impl ::std::ops::Sub<u16> for $t {
            type Output = ::std::result::Result<Self, ::bounded_int::ops::BoundedIntUnderflow>;

            fn sub(self, rhs: u16) -> Self::Output {
                Ok(Self((self.0.sub(rhs))?))
            }
        }

        impl ::std::ops::Add for $t {
            type Output = ::std::result::Result<Self, ::bounded_int::ops::BoundedIntOverflow>;

            fn add(self, rhs: Self) -> Self::Output {
                Ok(Self((self.0.add(rhs.0))?))
            }
        }

        impl ::std::ops::Sub for $t {
            type Output = ::std::result::Result<Self, ::bounded_int::ops::BoundedIntUnderflow>;

            fn sub(self, rhs: Self) -> Self::Output {
                Ok(Self((self.0.sub(rhs.0))?))
            }
        }
    };
}

impl_row_col!(Row);
impl_row_col!(Column);
