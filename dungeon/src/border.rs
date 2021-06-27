use std::{collections::HashSet, fmt};

use crate::Point;

#[derive(Clone)]
pub(crate) struct Border {
    pub(crate) id: BorderId,
    pub(crate) points: HashSet<Point>,
}

impl fmt::Debug for Border {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Border").field("id", &self.id).finish()
    }
}

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub(crate) struct BorderId(pub(crate) usize);

impl fmt::Debug for BorderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&*format!("{}", self.0))
    }
}
