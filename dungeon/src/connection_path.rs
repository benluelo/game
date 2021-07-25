use std::{collections::HashSet, iter};

use crate::{border::BorderId, Point};

/// A connection between two borders, with
#[derive(Debug, Clone)]
pub(crate) struct ConnectionPath {
    /// The [`BorderId`] of the [`Border`] that the path starts at.
    pub(crate) start_border_id: BorderId,
    /// The [`BorderId`] of the [`Border`] that the path ends at.
    pub(crate) end_border_id: BorderId,
    /// The path itself. See the type-level docs for more information.
    pub(crate) path: ConnectionPathLength,
}

#[allow(dead_code)]
impl ConnectionPath {
    /// Returns the length of the path.
    pub fn length(&self) -> usize {
        match &self.path {
            ConnectionPathLength::Length1 { .. } => 1,
            ConnectionPathLength::Length2 { .. } => 2,
            // add 2 to include the start and end points
            ConnectionPathLength::Length3Plus { points, .. } => points.len() + 2,
        }
    }

    /// Returns an iterator over the points of the path.
    pub fn iter(&self) -> impl Iterator<Item = Point> {
        match &self.path {
            ConnectionPathLength::Length1 { point } => vec![*point],
            ConnectionPathLength::Length2 { start, end } => vec![*start, *end],
            ConnectionPathLength::Length3Plus { points, start, end } => iter::once(*start)
                .chain(points.clone())
                .chain(iter::once(*end))
                .collect(),
        }
        .into_iter()
    }
}

/// Different lengths of paths.
///
/// Split up into an enum like this to allow for certain optimizations with
/// shorter paths, and to make the [`ConnectionPath`] construct slightly more
/// type-safe.
#[derive(Debug, Clone)]
pub(crate) enum ConnectionPathLength {
    /// Paths of length 1 only have one [`Point`]; i.e. the borders are one tile
    /// apart and their borders share this point.
    Length1 {
        /// TODO: Figure out what (if anything) to write for this field for docs
        point: Point,
    },
    /// Paths of length 2 have a start and an end [`Point`].
    Length2 {
        /// The start [`Point`]. This is the point in the
        /// [`ConnectionPath::start_border_id`] [`Border`].
        start: Point,
        /// The end [`Point`]. This is the point in the
        /// [`ConnectionPath::end_border_id`] [`Border`].
        end: Point,
    },
    /// All other paths have a start, an end, and 1 or more [`Point`]s in
    /// between.
    Length3Plus {
        /// The start [`Point`]. This is the point in the
        /// [`ConnectionPath::start_border_id`] [`Border`].
        start: Point,
        /// The end [`Point`]. This is the point in the
        /// [`ConnectionPath::end_border_id`] [`Border`].
        end: Point,
        /// the points of the path between the start and the end, excluding
        /// start and end.
        points: HashSet<Point>,
    },
}
