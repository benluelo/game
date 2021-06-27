use std::{
    convert::TryInto,
    ops::{Add, Mul},
    u64, u8,
};

use noise::{Billow, MultiFractal, NoiseFn, Seedable};
use num::Integer;
use pathfinding::prelude::dijkstra;
use rand::{thread_rng, Rng};

use crate::{
    bounded_int::BoundedInt,
    {
        floor_builder::{MAX_FLOOR_SIZE, MIN_FLOOR_SIZE, RANDOM_FILL_WALL_CHANCE},
        point_index::PointIndex,
        Column, DungeonTile, FloorBuilder, Point, Row,
    },
};

use super::{filled::Filled, FloorBuilderState};

/// A blank floor builder, with all values in the floor map and the noise map set to their default.
#[derive(Debug)]
pub(in crate::floor_builder) struct Blank {}
impl FloorBuilderState for Blank {}

impl FloorBuilder<Blank> {
    pub(in crate::floor_builder) fn random_fill(mut self) -> FloorBuilder<Filled> {
        let mut rng = thread_rng();

        let mut noise = Billow::new().set_seed(rng.gen()).set_persistence(128.0);

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

                *self.noise_map.at_mut(point, self.width) =
                    get_noise_value(&mut noise, column, row, self.height, self.width);

                if self.is_out_of_bounds(point) {
                    *self.map.at_mut(point, self.width) = DungeonTile::Wall;
                    continue;
                }

                // make a wall some percent of the time
                *self.map.at_mut(point, self.width) =
                    if rng.gen_range(0..=100) <= RANDOM_FILL_WALL_CHANCE {
                        DungeonTile::Wall
                    } else {
                        DungeonTile::Empty
                    }
            }
        }

        // original noisy walls map
        self.frame_from_current_state(100);

        // println!("{}\n", self.pretty(vec![], vec![]));

        // ANCHOR: dijkstra
        // find path through noise map and apply path to walls map
        let goal = Point {
            row: Row::new(self.height.expand_lower()).saturating_sub(4),
            column: Column::new(self.width.expand_lower()).saturating_sub(4),
        };

        let (found_path, _) = dijkstra(
            &Point {
                row: Row::new(4.try_into().unwrap()),
                column: Column::new(4.try_into().unwrap()),
            },
            |&point| {
                self.get_legal_neighbors(point)
                    .map(|p| (p, *self.noise_map.at(p, self.width) as u32))
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
            self.frame_from_current_state(1);
        }

        self.frame_from_current_state(100);

        FloorBuilder {
            extra: Filled {},
            height: self.height,
            width: self.width,
            map: self.map,
            noise_map: self.noise_map,
            frames: self.frames,
        }
    }
}

fn get_noise_value(
    noise: &mut Billow,
    column: BoundedInt<0, MAX_FLOOR_SIZE>,
    row: BoundedInt<0, MAX_FLOOR_SIZE>,
    height: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
    width: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
) -> u16 {
    // shift_range(
    u16::MAX
        - noise
            .get([
                (column.as_unbounded() as f64 / width.as_unbounded() as f64),
                (row.as_unbounded() as f64 / height.as_unbounded() as f64),
            ])
            .mul((u16::MAX / 2) as f64)
            .add(u16::MAX as f64)
            // .powi(2)
            .ceil() as u16
    // 0,
    // u8::MAX,
    // 0,
    // 16777216,
    // )
}

fn shift_range<I: Integer + Copy>(
    old_value: I,
    old_min: I,
    old_max: I,
    new_min: I,
    new_max: I,
) -> I {
    let new_value = ((old_value - old_min) / (old_max - old_min)) * (new_max - new_min) + new_min;
    new_value
}

#[cfg(test)]
mod test_noise {
    use std::path::Path;

    use itertools::Itertools;

    use super::*;

    #[test]
    fn test_noise_map_prettyness_lol_idk() {
        const WIDTH: i32 = 150;
        const HEIGHT: i32 = 100;
        let mut noise_map = vec![0; (WIDTH * HEIGHT) as usize];

        let mut rng = thread_rng();

        let mut noise = Billow::new().set_octaves(2).set_frequency(4.0).set_lacunarity(0.5) /* .set_seed(rng.gen()) */;

        for column in BoundedInt::<0, MAX_FLOOR_SIZE>::new(WIDTH)
            .unwrap()
            .range_from(&0.try_into().unwrap())
        {
            for row in BoundedInt::<0, MAX_FLOOR_SIZE>::new(HEIGHT)
                .unwrap()
                .range_from(&0.try_into().unwrap())
            {
                let point = Point {
                    column: Column::new(column),
                    row: Row::new(row),
                };

                let n = get_noise_value(
                    &mut noise,
                    column,
                    row,
                    HEIGHT.try_into().unwrap(),
                    WIDTH.try_into().unwrap(),
                );
                *noise_map.at_mut(point, BoundedInt::<0, MAX_FLOOR_SIZE>::new(WIDTH).unwrap()) =
                    dbg!(n);
            }
        }

        // let pixels = noise_map
        //     .into_iter()
        //     .flat_map(|x| {
        //         let red = (x >> 16) & 0xFF;
        //         let green = (x >> 8) & 0xFF;
        //         let blue = x & 0xFF;
        //         [red as u8, green as u8, blue as u8]
        //     })
        //     .collect_vec();

        let _ = image::save_buffer(
            &Path::new(&"noise.png"),
            &*noise_map
                .into_iter()
                .flat_map(|integer| [integer as u8, (integer >> 8) as u8])
                .collect_vec(),
            WIDTH as u32,
            HEIGHT as u32,
            image::ColorType::L16,
        )
        .unwrap();
    }
}
