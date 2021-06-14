use crate::dungeon::floor_builder::{bounded_int::BoundedInt, MAX_FLOOR_SIZE};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord, /* , derive_more::Add, derive_more::Sub, */
)]
pub struct Point {
    /// width
    pub column: Column,
    /// height
    pub row: Row,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord, /* , derive_more::Add, derive_more::Sub, */
)]
pub struct Row(pub(super) BoundedInt<0, { MAX_FLOOR_SIZE }>);

impl Row {
    pub fn new(row: BoundedInt<0, { MAX_FLOOR_SIZE }>) -> Self {
        Row(row)
    }

    pub fn get(&self) -> BoundedInt<0, { MAX_FLOOR_SIZE }> {
        self.0
    }

    pub fn saturating_add(self, n: i32) -> Self {
        Row(self.0.saturating_add(n))
    }

    pub fn saturating_sub(self, n: i32) -> Self {
        Row(self.0.saturating_sub(n))
    }
}

// impl num_traits::SaturatingSub for Row {
//     fn saturating_sub(&self, v: &Self) -> Self {
//         Row(self.0.saturating_sub(v.0))
//     }
// }

// impl num_traits::SaturatingAdd for Row {
//     fn saturating_add(&self, v: &Self) -> Self {
//         Row(self.0.saturating_add(v.0))
//     }
// }

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord, /* , derive_more::Add, derive_more::Sub */
)]
pub struct Column(pub(super) BoundedInt<0, { MAX_FLOOR_SIZE }>);

impl Column {
    pub fn new(column: BoundedInt<0, { MAX_FLOOR_SIZE }>) -> Self {
        Column(column)
    }

    pub fn get(&self) -> BoundedInt<0, { MAX_FLOOR_SIZE }> {
        self.0
    }

    pub fn saturating_add(self, n: i32) -> Self {
        Column(self.0.saturating_add(n))
    }

    pub fn saturating_sub(self, n: i32) -> Self {
        Column(self.0.saturating_sub(n))
    }
}

// impl num_traits::SaturatingSub for Column {
//     fn saturating_sub(&self, v: &Self) -> Self {
//         Column(self.0.saturating_sub(v.0))
//     }
// }

// impl num_traits::SaturatingAdd for Column {
//     fn saturating_add(&self, v: &Self) -> Self {
//         Column(self.0.saturating_add(v.0))
//     }
// }

// TODO: use methods on row and col
impl Point {
    pub fn saturating_add_row(self, n: i32) -> Self {
        Self {
            row: self.row.saturating_add(n),
            column: self.column,
        }
    }

    pub fn saturating_sub_row(self, n: i32) -> Self {
        Self {
            row: Row(self.row.0.saturating_sub(n)),
            column: self.column,
        }
    }
    pub fn saturating_add_column(self, n: i32) -> Self {
        Self {
            column: Column(self.column.0.saturating_add(n)),
            row: self.row,
        }
    }

    pub fn saturating_sub_column(self, n: i32) -> Self {
        Self {
            column: Column(self.column.0.saturating_sub(n)),
            row: self.row,
        }
    }
}
