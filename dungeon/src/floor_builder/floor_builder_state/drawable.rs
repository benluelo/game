use crate::floor_builder::filled::Filled;
use crate::{
    connection_path::{ConnectionPath, ConnectionPathLength},
    point_index::PointIndex,
    DungeonTile, FloorBuilder, Point,
};

use super::FloorBuilderState;

#[derive(Debug)]
pub(in crate::floor_builder) struct Drawable {
    pub(super) to_draw: Vec<ConnectionPath>,
}
impl FloorBuilderState for Drawable {}

impl FloorBuilder<Drawable> {
    pub(in crate::floor_builder) fn draw(
        mut self,
        draw_with: fn(
            // is_first
            bool,
            // is_last
            bool,
            Point,
        ) -> DungeonTile,
    ) -> FloorBuilder<Filled> {
        for path in self.extra.to_draw.clone().into_iter() {
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

                        self.frame_from_current_state(1);
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
            frames: self.frames,
            id: self.id,
        }
    }
}
