use crate::dungeon::{Floor, FloorBuilder};

use super::{FloorBuilderState, Smoothable};

// The final state of the floor builder.
#[derive(Debug)]
pub(in crate::dungeon::floor_builder) struct Filled {}
impl FloorBuilderState for Filled {}
impl Smoothable for Filled {}

impl FloorBuilder<Filled> {
    pub(in crate::dungeon::floor_builder) fn finish(self) -> Floor {
        Floor {
            height: self.height,
            width: self.width,
            data: self.map,
        }
    }
}
