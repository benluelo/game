use std::convert::TryInto;

use noise::{Billow, MultiFractal, NoiseFn, Seedable};
use pathfinding::prelude::dijkstra;
use rand::{thread_rng, Rng};

use crate::dungeon::{
    floor_builder::RANDOM_FILL_WALL_CHANCE, point_index::PointIndex, Column, DungeonTile,
    FloorBuilder, Point, Row,
};

use super::{filled::Filled, FloorBuilderState};

/// A blank floor builder, with all values in the floor map and the noise map set to their default.
#[derive(Debug)]
pub(in crate::dungeon::floor_builder) struct Blank {}
impl FloorBuilderState for Blank {}

impl FloorBuilder<Blank> {
    pub fn random_fill(mut self) -> FloorBuilder<Filled> {
        let mut rng = thread_rng();

        let noise = Billow::new().set_seed(rng.gen()).set_persistence(128.0);

        // build initial maps (walls and noise)
        for column in self
            .width
            .expand_lower::<0>()
            .range_from(&0.try_into().unwrap())
        {
            for row in self
                .height
                .expand_lower::<0>()
                .range_from(&0.try_into().unwrap())
            {
                let point = Point {
                    column: Column::new(column),
                    row: Row::new(row),
                };

                *self.noise_map.at_mut(point, self.width) = ((noise.get([
                    (column.as_unbounded() as f64 / self.width.as_unbounded() as f64) + 0.1,
                    (row.as_unbounded() as f64 / self.height.as_unbounded() as f64) + 0.1,
                ]) + 0.001)
                    .abs()
                    * 10000.0)
                    .powi(2)
                    .floor() as u128;

                // make a wall some percent of the time
                *self.map.at_mut(point, self.width) =
                    if rng.gen_range(0..=100) <= RANDOM_FILL_WALL_CHANCE {
                        DungeonTile::Wall
                    } else {
                        DungeonTile::Empty
                    }
            }
        }

        // println!("{}\n", self.pretty(vec![], vec![]));

        // ANCHOR: dijkstra
        // find path through noise map and apply path to walls map
        let goal = Point {
            row: Row::new(self.height.expand_lower()).saturating_sub(4),
            column: Column::new(self.width.expand_lower::<0>()).saturating_sub(4),
        };

        let (found_path, _) = dijkstra(
            &Point {
                row: Row::new(4.try_into().unwrap()),
                column: Column::new(4.try_into().unwrap()),
            },
            |&point| {
                self.get_legal_neighbors(point)
                    .map(|p| (p, *self.noise_map.at(p, self.width)))
            },
            |&point| !self.is_out_of_bounds(point) && point == goal,
        )
        .expect("no path found");

        for &point in &found_path {
            *self.map.at_mut(point, self.width) = DungeonTile::Empty;
            for neighbor in self
                .get_legal_neighbors_down_and_right(point)
                .collect::<Vec<_>>()
            {
                *self.map.at_mut(neighbor, self.width) = DungeonTile::Empty;
            }
        }

        FloorBuilder {
            extra: Filled {},
            height: self.height,
            width: self.width,
            map: self.map,
            noise_map: self.noise_map,
        }
    }
}
