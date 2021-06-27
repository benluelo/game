use crate::BoundedInt;
use std::{cmp::Ordering, convert::TryInto};

pub struct BoundedIntRangeInclusive<const LOW: i32, const HIGH: i32> {
    // start: BoundedInt<{ LOW }, { HIGH }>,
    pub(super) end: BoundedInt<{ LOW }, { HIGH }>,
    pub(super) pointer: BoundedInt<{ LOW }, { HIGH }>,
    pub(super) finished: bool,
}

impl<const LOW: i32, const HIGH: i32> Iterator for BoundedIntRangeInclusive<{ LOW }, { HIGH }> {
    type Item = BoundedInt<{ LOW }, { HIGH }>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.pointer.cmp(&self.end) {
            Ordering::Greater => None,
            Ordering::Equal => {
                if !self.finished {
                    self.finished = true;
                    Some(self.pointer)
                } else {
                    None
                }
            }
            Ordering::Less => {
                let to_return = self.pointer;
                self.pointer = (self.pointer.as_unbounded() + 1_i32).try_into().unwrap();
                Some(to_return)
            }
        }
    }
}

pub struct BoundedIntRange<const LOW: i32, const HIGH: i32> {
    // start: BoundedInt<{ LOW }, { HIGH }>,
    pub(super) end: BoundedInt<{ LOW }, { HIGH }>,
    pub(super) pointer: BoundedInt<{ LOW }, { HIGH }>,
}

impl<const LOW: i32, const HIGH: i32> Iterator for BoundedIntRange<{ LOW }, { HIGH }> {
    type Item = BoundedInt<{ LOW }, { HIGH }>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.pointer.cmp(&self.end) {
            Ordering::Greater | Ordering::Equal => None,
            Ordering::Less => {
                let to_return = self.pointer;
                self.pointer = (self.pointer.as_unbounded() + 1_i32).try_into().unwrap();
                Some(to_return)
            }
        }
    }
}
