use crate::{
    bounded_int::BoundedInt,
    dungeon::{
        floor_builder::{MAX_FLOOR_SIZE, MIN_FLOOR_SIZE},
        DungeonTile, Floor, FloorBuilder,
    },
};

use super::{blank::Blank, has_borders::BuildConnectionIterations, FloorBuilderState};
#[derive(Debug)]
pub struct New {}
impl FloorBuilderState for New {}

impl FloorBuilder<New> {
    pub(in crate::dungeon) fn create(
        width: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
        height: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
    ) -> Floor {
        FloorBuilder::<Blank>::blank(width, height)
            .random_fill()
            .smoothen(3, |r| r < 4)
            .get_cave_borders()
            .build_connections(BuildConnectionIterations::Finite(20))
            .trace_connection_paths(true, true)
            .draw(|_, _, _| DungeonTile::Empty)
            .smoothen(7, |_| false)
            .check_for_secret_passages()
            .finish()
    }
}
