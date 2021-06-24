use crate::dungeon::floor_builder::filled::Filled;
use crate::dungeon::{
    point_index::PointIndex, ConnectionPath, ConnectionPathLength, DungeonTile, FloorBuilder, Point,
};

use super::FloorBuilderState;

#[derive(Debug)]
pub(in crate::dungeon::floor_builder) struct Drawable {
    pub(super) to_draw: Vec<ConnectionPath>,
}
impl FloorBuilderState for Drawable {}

impl FloorBuilder<Drawable> {
    pub(in crate::dungeon::floor_builder) fn draw(
        mut self,
        draw_with: fn(
            // is_first
            bool,
            // is_last
            bool,
            Point,
        ) -> DungeonTile,
    ) -> FloorBuilder<Filled> {
        for path in self.extra.to_draw.into_iter() {
            use ConnectionPathLength::*;
            match &path.path {
                Length1 { point } => {
                    *self.map.at_mut(*point, self.width) = draw_with(true, true, *point)
                }
                Length2 { start, end } => {
                    *self.map.at_mut(*start, self.width) = draw_with(true, false, *start);
                    *self.map.at_mut(*end, self.width) = draw_with(false, true, *end);
                }
                Length3Plus { points, start, end } => {
                    assert!(!points.contains(start));
                    assert!(!points.contains(end));
                    *self.map.at_mut(*start, self.width) = draw_with(true, false, *start);
                    *self.map.at_mut(*end, self.width) = draw_with(false, true, *end);

                    for point in points {
                        *self.map.at_mut(*point, self.width) = draw_with(false, false, *point);
                    }
                }
            };
        }

        FloorBuilder {
            width: self.width,
            height: self.height,
            map: self.map,
            noise_map: self.noise_map,
            extra: Filled {},
        }
    }
}
