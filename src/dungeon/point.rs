#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, derive_more::Add, derive_more::Sub,
)]
pub struct Point {
    /// width
    pub column: Column,
    /// height
    pub row: Row,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, derive_more::Add, derive_more::Sub,
)]
pub struct Row(i32);

impl Row {
    pub fn new(row: i32) -> Self {
        Row(row)
    }

    pub fn get(&self) -> i32 {
        self.0
    }
}

impl num_traits::SaturatingSub for Row {
    fn saturating_sub(&self, v: &Self) -> Self {
        Row(self.0.saturating_sub(v.0))
    }
}

impl num_traits::SaturatingAdd for Row {
    fn saturating_add(&self, v: &Self) -> Self {
        Row(self.0.saturating_add(v.0))
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, derive_more::Add, derive_more::Sub,
)]
pub struct Column(i32);

impl Column {
    pub fn new(column: i32) -> Self {
        Column(column)
    }

    pub fn get(&self) -> i32 {
        self.0
    }
}

impl num_traits::SaturatingSub for Column {
    fn saturating_sub(&self, v: &Self) -> Self {
        Column(self.0.saturating_sub(v.0))
    }
}

impl num_traits::SaturatingAdd for Column {
    fn saturating_add(&self, v: &Self) -> Self {
        Column(self.0.saturating_add(v.0))
    }
}
