use crate::{
    connection_path::{ConnectionPath, ConnectionPathLength},
    floor_builder::filled::Filled,
    point_index::PointIndex,
    DungeonTile, FloorBuilder, Point,
};

use super::FloorBuilderState;

/// A [`FloorBuilder`] that has data to be written to it's internal map.
#[derive(Debug)]
pub(in crate::floor_builder) struct Drawable {
    /// The connection paths to be drawn.
    pub(super) to_draw: Vec<ConnectionPath>,
}
impl FloorBuilderState for Drawable {
    const TYPE_NAME: &'static str = "Drawable";

}

impl FloorBuilder<Drawable> {
    /// Draws the current state of the [`FloorBuilder`] with the provided
    /// function.
    ///
    /// `draw_with` takes 3 parameters:
    /// The first is true if the point is the first in the path, the second is
    /// true if the point is the last in the path, and the third argument is
    /// the point itself. The function is expected to return a
    /// [`DungeonTile`] that will be placed at the provided point.
    ///
    /// Note that the first and second arguments are not mutually exclisive.
    pub(in crate::floor_builder) fn draw(
        mut self,
        // TODO: Make a more explicit type for the first two arguments (`enum PositionInPath`
        // perhaps?)
        draw_with: fn(
            // is_first
            bool,
            // is_last
            bool,
            Point,
        ) -> DungeonTile,
    ) -> FloorBuilder<Filled> {
        for path in self.extra.to_draw.clone() {
            match &path.path {
                ConnectionPathLength::Length1 { point } => {
                    *self.map.at_mut(*point, self.width) = draw_with(true, true, *point)
                }
                ConnectionPathLength::Length2 { start, end } => {
                    *self.map.at_mut(*start, self.width) = draw_with(true, false, *start);
                    *self.map.at_mut(*end, self.width) = draw_with(false, true, *end);
                }
                ConnectionPathLength::Length3Plus { points, start, end } => {
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
