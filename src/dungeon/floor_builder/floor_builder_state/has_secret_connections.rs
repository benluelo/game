use std::convert::TryInto;

use itertools::Itertools;
use rand::{prelude::SliceRandom, thread_rng};

use crate::dungeon::{point_index::PointIndex, Column, FloorBuilder, Point, Row};

use super::{filled::Filled, FloorBuilderState};

pub(in crate::dungeon::floor_builder) struct HasSecretPassages;

impl FloorBuilderState for HasSecretPassages {}

impl FloorBuilder<HasSecretPassages> {
    pub(in crate::dungeon::floor_builder) fn place_treasure_chests(self) -> FloorBuilder<Filled> {
        let mut rng = thread_rng();
        let empty_points_sorted_by_noise = self
            .width
            .expand_lower()
            .range_from(&0.try_into().unwrap())
            .flat_map(|column| {
                self.height
                    .expand_lower()
                    .range_from(&0.try_into().unwrap())
                    .map(move |row| Point {
                        column: Column::new(column),
                        row: Row::new(row),
                    })
            })
            .filter(|&point| {
                matches!(
                    self.map.at(point, self.width),
                    crate::dungeon::DungeonTile::Empty
                )
            })
            .sorted_by(|&a, &b| {
                self.noise_map
                    .at(a, self.width)
                    .cmp(self.noise_map.at(b, self.width))
            });

        for point in empty_points_sorted_by_noise {
            println!("{}", self.noise_map.at(point, self.width));
        }

        todo!()
    }
}
