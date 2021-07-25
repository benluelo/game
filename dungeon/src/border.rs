use std::{collections::HashSet, fmt};

use crate::Point;

/// A border of points around a cave in a floor.
#[derive(Clone)]
pub(crate) struct Border {
    /// A unique, opaque ID assigned to each border in the floor.
    pub(crate) id: BorderId,
    /// The points of the border. Stored in a [`HashSet`] for easy access
    /// and so that there are no duplicates.
    pub(crate) points: HashSet<Point>,
}

impl fmt::Debug for Border {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Border").field("id", &self.id).finish()
    }
}

/// A unique, opaque ID assigned to each border in a floor.
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub(crate) struct BorderId(usize);

impl BorderId {
    /// Creates a new [`BorderId`] with the given id.
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

impl fmt::Debug for BorderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&*format!("{}", self.0))
    }
}
