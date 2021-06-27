use std::{collections::HashSet, iter};

use crate::{border::BorderId, Point};

#[derive(Debug, Clone)]
pub(crate) struct ConnectionPath {
    pub(crate) start_border_id: BorderId,
    pub(crate) end_border_id: BorderId,
    pub(crate) path: ConnectionPathLength,
}

#[allow(dead_code)]
impl ConnectionPath {
    pub fn length(&self) -> usize {
        use ConnectionPathLength::*;
        match &self.path {
            Length1 { .. } => 1,
            Length2 { .. } => 2,
            // add 2 to include the start and end points
            Length3Plus { points, .. } => points.len() + 2,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Point> {
        use ConnectionPathLength::*;
        match &self.path {
            Length1 { point } => vec![*point],
            Length2 { start, end } => vec![*start, *end],
            Length3Plus { points, start, end } => iter::once(*start)
                .chain(points.clone())
                .chain(iter::once(*end))
                .collect(),
        }
        .into_iter()
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ConnectionPathLength {
    Length1 {
        point: Point,
    },
    Length2 {
        start: Point,
        end: Point,
    },
    Length3Plus {
        start: Point,
        end: Point,
        /// the points of the path between the start and the end, escluding start and end
        points: HashSet<Point>,
    },
}
