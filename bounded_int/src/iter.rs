use crate::BoundedInt;
use std::{cmp::Ordering, convert::TryInto};

/// A range between two [`BoundedInt`]s, inclusive on both ends.
#[must_use = "range will do nothing unless iterated over"]
pub struct BoundedIntRangeInclusive<const LOW: i32, const HIGH: i32> {
    // REVIEW: Should the ranges keep track of their start points?
    // start: BoundedInt<{ LOW }, { HIGH }>,
    /// Where the iterator currently is. Initially equal to the start of the
    /// range.
    pub(super) pointer: BoundedInt<{ LOW }, { HIGH }>,
    /// The end of the range. This is equal to the last item that will be
    /// returned,
    pub(super) end: BoundedInt<{ LOW }, { HIGH }>,
    /// Whether or not the last item (`end`) has been iterated over.
    pub(super) finished: bool,
}

impl<const LOW: i32, const HIGH: i32> Iterator for BoundedIntRangeInclusive<{ LOW }, { HIGH }> {
    type Item = BoundedInt<{ LOW }, { HIGH }>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.pointer.cmp(&self.end) {
            Ordering::Greater => None,
            Ordering::Equal =>
            {
                #[allow(clippy::if_not_else)]
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
/// A range between two [`BoundedInt`]s, inclusive at the start and exclusive at
/// the end.
#[must_use = "range will do nothing unless iterated over"]
pub struct BoundedIntRange<const LOW: i32, const HIGH: i32> {
    // REVIEW: Should the ranges keep track of their start points?
    // start: BoundedInt<{ LOW }, { HIGH }>,
    /// Where the iterator currently is. Initially equal to the start of the
    /// range.
    pub(super) pointer: BoundedInt<{ LOW }, { HIGH }>,
    /// The end of the range. This is one higher than the last item that will be
    /// returned,
    pub(super) end: BoundedInt<{ LOW }, { HIGH }>,
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
