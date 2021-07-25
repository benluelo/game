use crate::{
    floor_builder::{MAX_FLOOR_SIZE, MIN_FLOOR_SIZE},
    DungeonTile, Floor, FloorBuilder, FloorId,
};

use bounded_int::BoundedInt;

use super::{blank::Blank, has_borders::BuildConnectionIterations, FloorBuilderState};

/// The initial state of the floor builder. Entry point to the state machine.
#[derive(Debug)]
pub struct New {}
impl FloorBuilderState for New {}

impl FloorBuilder<New> {
    /// Creates a new floor builder with the provided values.
    pub(in crate) fn create(
        id: FloorId,
        width: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
        height: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
        gif_output: bool,
    ) -> Floor {
        FloorBuilder::<Blank>::blank(id, width, height, gif_output)
            .random_fill()
            .smoothen(3, |r| r < 4)
            .get_cave_borders()
            .build_connections(BuildConnectionIterations::Finite(20))
            .trace_connection_paths(true, true)
            .draw(|_, _, _| DungeonTile::Empty)
            .smoothen(7, |_| false)
            .check_for_secret_passages()
            .place_treasure_chests()
            .finish()
    }
}
