use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use crate::dungeon::{Border, BorderId, Point};

pub trait FloorBuilderState {}
pub trait Smoothable: FloorBuilderState {}

pub trait DebugIterator: Iterator<Item = Point> + fmt::Debug {}

impl<T> DebugIterator for T where T: Iterator<Item = Point> + fmt::Debug {}

#[derive(Debug)]
pub(super) struct Drawable {
    pub(super) to_draw: Vec<Vec<Point>>,
}
impl FloorBuilderState for Drawable {}

#[derive(Debug)]
pub(super) struct Smoothed {}
impl FloorBuilderState for Smoothed {}

#[derive(Debug)]
pub struct New {}
impl FloorBuilderState for New {}

#[derive(Debug)]
pub(super) struct Buildable {}
impl FloorBuilderState for Buildable {}

#[derive(Debug)]
pub(super) struct HasBorders {
    pub(super) borders: Vec<Border>,
}
impl FloorBuilderState for HasBorders {}

#[derive(Debug)]
pub(super) struct HasConnections {
    pub(super) connections: HashMap<(BorderId, Point), (BorderId, Point)>,
}
impl FloorBuilderState for HasConnections {}

/// A blank floor builder, with all values in the floor map and the noise map set to their default.
#[derive(Debug)]
pub struct Blank {}
impl FloorBuilderState for Blank {}

// The final state of the floor builder.
#[derive(Debug)]
pub struct Filled {}
impl FloorBuilderState for Filled {}
impl Smoothable for Filled {}

// The final state of the floor builder.
#[derive(Debug)]
pub struct Done {}
impl FloorBuilderState for Done {}
impl Smoothable for Done {}
