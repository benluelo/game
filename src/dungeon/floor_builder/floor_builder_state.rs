use std::collections::{HashMap, HashSet};

use crate::dungeon::{BorderId, Point};

pub trait FloorBuilderState {}
pub trait Smoothable: FloorBuilderState {}

pub(super) struct Drawable {
    pub(super) to_draw: Vec<Point>,
}
impl FloorBuilderState for Drawable {}

pub(super) struct Smoothed {}
impl FloorBuilderState for Smoothed {}

pub struct New {}
impl FloorBuilderState for New {}

pub(super) struct Buildable {}
impl FloorBuilderState for Buildable {}

pub(super) struct HasBorders {
    pub(super) borders: Vec<HashSet<Point>>,
}
impl FloorBuilderState for HasBorders {}

pub(super) struct HasConnections {
    pub(super) connections: HashMap<(BorderId, Point), (BorderId, Point)>,
}
impl FloorBuilderState for HasConnections {}

/// A blank floor builder, with all values in the floor map and the noise map set to their default.
pub(super) struct Blank {}
impl FloorBuilderState for Blank {}

// The final state of the floor builder.
pub(super) struct Filled {}
impl FloorBuilderState for Filled {}
impl Smoothable for Filled {}

// The final state of the floor builder.
pub struct Done {}
impl FloorBuilderState for Done {}
impl Smoothable for Done {}
