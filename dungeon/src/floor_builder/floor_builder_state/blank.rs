use std::{
    convert::TryInto,
    ops::{Add, Mul},
};

use bounded_int::BoundedInt;
use noise::{Billow, MultiFractal, NoiseFn, Seedable};
use pathfinding::prelude::dijkstra;
use rand::{thread_rng, Rng};

use crate::{
    distance,
    floor_builder::{MAX_FLOOR_SIZE, MIN_FLOOR_SIZE, RANDOM_FILL_WALL_PERCENT_CHANCE},
    point_index::PointIndex,
    Column, DungeonTile, FloorBuilder, Point, Row,
};

use super::{filled::Filled, FloorBuilderState};

/// A blank floor builder, with all values in the floor map and the noise map
/// set to their default.
#[derive(Debug)]
pub(in crate::floor_builder) struct Blank {}
impl FloorBuilderState for Blank {}

impl FloorBuilder<Blank> {
    /// TODO: Split this function into two parts, `random_fill` and
    /// `trace_entrance_exit` (or something along those lines)
    pub(in crate::floor_builder) fn random_fill(mut self) -> FloorBuilder<Filled> {
        let mut rng = thread_rng();

        let mut noise = create_billow(&mut rng);

        // build initial maps (walls and noise)
        for column in self
            .width
            .expand_lower::<0>()
            .range_from(0.try_into().unwrap())
        {
            for row in self
                .height
                .expand_lower::<0>()
                .range_from(0.try_into().unwrap())
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
                    if rng.gen_range(0..=100) <= RANDOM_FILL_WALL_PERCENT_CHANCE {
                        DungeonTile::Wall
                    } else {
                        DungeonTile::Empty
                    }
            }
        }

        // original noisy walls map
        self.frame_from_current_state(100);

        let mut rng = thread_rng();

        // ANCHOR: dijkstra
        // find a path through the noise map and apply the found path to the walls map
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

/// Gets the noise value for the provided billow at the row and column
/// specified.
fn get_noise_value(
    noise: &mut Billow,
    column: BoundedInt<0, MAX_FLOOR_SIZE>,
    row: BoundedInt<0, MAX_FLOOR_SIZE>,
    height: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
    width: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
) -> u16 {
    /// Makes it easier for me to think about powers of two
    mod u4 {
        /// 2^4
        pub const MAX: u8 = 16;
    }

    #[allow(clippy::cast_possible_truncation)]
    let n = noise
        .get([
            (column.as_unbounded() as f64 / width.as_unbounded() as f64),
            (row.as_unbounded() as f64 / height.as_unbounded() as f64),
        ])
        // change the range from [-1, 1] to [-8, 8]
        .mul((u4::MAX / 2) as f64)
        // change the range from [-8, 8] to [0, 16]
        .add((u4::MAX) as f64)
        // change the range from [0, 16] to [0, 2^16]
        .powi(4)
        // round up
        .ceil() as u16;

    // these ⭐ magic numbers ⭐ have been hand crafted to perfection
    // don't touch them pls
    // TODO: Figure out what these magic numbers do lol
    #[allow(clippy::cast_possible_truncation)]
    if n <= (u16::MAX as f64 / 2.5) as u16 {
        n / 2
    } else {
        u16::MAX
    }
}

/// Creates a [`Billow`] using some magic numbers that have been fine tuned to
/// work well.
///
/// Don't touch 'em
fn create_billow(rng: &mut impl rand::Rng) -> Billow {
    Billow::new()
        .set_octaves(1)
        .set_frequency(5.0)
        .set_lacunarity(0.001)
        .set_persistence(0.001)
        .set_seed(rng.gen())
}

// fn _shift_range<I: Integer + Copy>(
//     old_value: I,
//     old_min: I,
//     old_max: I,
//     new_min: I,
//     new_max: I,
// ) -> I {
//     let new_value = ((old_value - old_min) / (old_max - old_min)) * (new_max
// - new_min) + new_min;     new_value
// }

#[cfg(test)]
mod test_noise {
    use std::path::Path;

    use itertools::Itertools;

    use super::*;

    #[test]
    fn test_noise_map_prettyness_lol_idk() {
        const WIDTH: i32 = 200;
        const HEIGHT: i32 = 200;
        let mut noise_map = vec![0; (WIDTH * HEIGHT) as usize];

        let mut rng = thread_rng();

        let mut noise = create_billow(&mut rng);

        for column in BoundedInt::<0, MAX_FLOOR_SIZE>::new(WIDTH)
            .unwrap()
            .range_from(0.try_into().unwrap())
        {
            for row in BoundedInt::<0, MAX_FLOOR_SIZE>::new(HEIGHT)
                .unwrap()
                .range_from(0.try_into().unwrap())
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
                *noise_map.at_mut(point, BoundedInt::<0, MAX_FLOOR_SIZE>::new(WIDTH).unwrap()) = n;
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
