use std::convert::TryInto;

use pathfinding::prelude::dijkstra;
use rand::{thread_rng, Rng};

use crate::{
    distance, point_index::PointIndex,
    Column, DungeonTile, FloorBuilder, Point, Row,
};

use super::{filled::Filled, FloorBuilderState};

/// A builder that has been filled with the original random 'seed' for the cellular automata cave creation.
#[derive(Debug)]
pub(in crate::floor_builder) struct RandomFilled {}
impl FloorBuilderState for RandomFilled {
    const TYPE_NAME: &'static str = "RandomFilled";
}

impl FloorBuilder<RandomFilled> {
    /// Traces the original path through the map from the entrance to the exit.
    pub(in crate::floor_builder) fn trace_original_path(mut self) -> FloorBuilder<Filled> {
        let mut rng = thread_rng();
        let start = Point {
            row: Row::new(
                rng.gen_range(1..(self.height.as_unbounded() - 1))
                    .try_into()
                    .unwrap(),
            ),
            column: Column::new(
                rng.gen_range(1..(self.width.as_unbounded() - 1))
                    .try_into()
                    .unwrap(),
            ),
        };

        let end = loop {
            let larger_dimension = if self.width > self.height {
                self.width
            } else {
                self.height
            };
            let maybe_end = Point {
                row: Row::new(
                    rng.gen_range(1..(self.height.as_unbounded() - 1))
                        .try_into()
                        .unwrap(),  
                ),
                column: Column::new(
                    rng.gen_range(1..(self.width.as_unbounded() - 1))
                        .try_into()
                        .unwrap(),
                ),
            };
            let dist = distance(maybe_end, start);

            #[allow(clippy::redundant_else)] // I prefer the explicitness here
            if dist > (larger_dimension.as_unbounded() as f64 / 2.0)
                && dist < (larger_dimension.as_unbounded() as f64)
            {
                break maybe_end;
            } else {
                continue;
            }
        };

        let (found_path, _) = dijkstra(
            &start,
            |&point| {
                self.get_legal_neighbors(point)
                    .map(|p| (p, *self.noise_map.at(p, self.width) as u32))
            },
            |&point| !self.is_out_of_bounds(point) && point == end,
        )
        .expect("no path found");

        *self.map.at_mut(start, self.width) = DungeonTile::Entrance;
        *self.map.at_mut(end, self.width) = DungeonTile::Exit;

        for &point in &found_path {
            if self.map.at(point, self.width).is_solid() && point != start && point != end {
                *self.map.at_mut(point, self.width) = DungeonTile::Empty;
            }
            for neighbor in self
                .get_legal_neighbors_down_and_right(point)
                .collect::<Vec<_>>()
            {
                if self.map.at(neighbor, self.width).is_solid() && point != start && point != end {
                    *self.map.at_mut(neighbor, self.width) = DungeonTile::Empty;
                }
            }
            self.frame_from_current_state(1);
        }
        assert_eq!(self.map.at(start, self.width), &DungeonTile::Entrance);
        assert_eq!(self.map.at(end, self.width), &DungeonTile::Exit);
        self.frame_from_current_state(100);
        FloorBuilder {
            extra: Filled {},
            height: self.height,
            width: self.width,
            map: self.map,
            noise_map: self.noise_map,
            frames: self.frames,
            id: self.id,
        }
    }
}
