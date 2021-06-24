use crate::dungeon::floor_builder::FloorBuilderState;

use super::Smoothable;

// The final state of the floor builder.
#[derive(Debug)]
pub struct Done {}
impl FloorBuilderState for Done {}
impl Smoothable for Done {}
