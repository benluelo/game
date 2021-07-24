use std::convert::TryInto;

use itertools::Itertools;
use rand::{prelude::SliceRandom, thread_rng, Rng};

use crate::{point_index::PointIndex, Column, DungeonTile, FloorBuilder, Point, Row};

use super::{filled::Filled, FloorBuilderState};

pub(in crate::floor_builder) struct HasSecretPassages;

impl FloorBuilderState for HasSecretPassages {}

impl FloorBuilder<HasSecretPassages> {
    pub(in crate::floor_builder) fn place_treasure_chests(mut self) -> FloorBuilder<Filled> {
        let mut rng = thread_rng();
        let mut empty_points_sorted_by_noise = self
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
            .filter(|&point| matches!(self.map.at(point, self.width), crate::DungeonTile::Empty))
            // .filter(|&point| {
            //     self.get_legal_neighbors_with_diagonals(point)
            //         .all(|point| self.map.at(point, self.width) == &DungeonTile::Empty)
            // })
            // .sorted_by(|&a, &b| {
            //     self.noise_map
            //         .at(a, self.width)
            //         .cmp(self.noise_map.at(b, self.width))
            // })
            .collect_vec();

        let mut amount = (0..rng.gen_range(5..=10)).peekable();
        empty_points_sorted_by_noise.shuffle(&mut rng);

        for point in empty_points_sorted_by_noise {
            if amount.peek().is_some() {
                if self
                    .get_legal_neighbors_with_diagonals(point)
                    .all(|point| self.map.at(point, self.width) == &DungeonTile::Empty)
                {
                    self.frame_from_current_state(10);
                    *self.map.at_mut(point, self.width) =
                        DungeonTile::TreasureChest { contents: () };
                    amount.next();
                }
                dbg!(&self.noise_map.at(point, self.width));
            } else {
                break;
            }
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
