use crate::{
    bounded_int::BoundedInt,
    {
        dungeon_tile::DungeonTile, floor_builder::floor_builder_state::*, point_index::PointIndex,
        Column, Point, Row,
    },
};
use ansi_term::ANSIStrings;
use itertools::Itertools;
use pathfinding::prelude::dijkstra;

use std::{borrow::Cow, convert::TryInto, fmt::Debug, vec};

use self::floor_builder_state::{blank::Blank, smoothed::Smoothed};

mod floor_builder_state;
mod to_block_character;

pub const MIN_FLOOR_SIZE: i32 = 10;
pub const MAX_FLOOR_SIZE: i32 = 200;
const RANDOM_FILL_WALL_CHANCE: u8 = 52;

/// Represents a floor of a dungeon
/// See http://roguebasin.roguelikedevelopment.org/index.php?title=Cellular_Automata_Method_for_Generating_Random_Cave-Like_Levels
#[derive(Debug)]
pub struct FloorBuilder<S: FloorBuilderState> {
    pub(crate) width: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
    pub(crate) height: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
    pub(crate) map: Vec<DungeonTile>,
    pub(crate) noise_map: Vec<u16>,
    extra: S,
    frames: Option<Vec<gif::Frame<'static>>>,
}

// #[cfg(test)]
// mod test_build_connection_iterations {
//     use super::*;

//     #[test]
//     fn test_finite() {
//         let to_zip = vec![1, 2, 3, 4, 5];
//         for i in BuildConnectionIterations::Finite(10)
//             .as_range()
//             .zip(&to_zip)
//         {
//             println!("{:?}", i);
//         }
//     }
//     #[test]
//     fn test_infinite() {
//         let to_zip = vec![1, 2, 3, 4, 5];
//         for i in BuildConnectionIterations::FullyConnect
//             .as_range()
//             .zip(to_zip)
//         {
//             println!("{:?}", i);
//         }
//     }
// }

impl<S: Smoothable> FloorBuilder<S> {
    fn smoothen(
        mut self,
        repeat: usize,
        create_new_walls: fn(usize) -> bool,
    ) -> FloorBuilder<Smoothed> {
        for r in 0..repeat {
            for column in self.width.expand_lower().range_from(&0.try_into().unwrap()) {
                for row in self
                    .height
                    .expand_lower()
                    .range_from(&0.try_into().unwrap())
                {
                    let point = Point {
                        column: Column::new(column),
                        row: Row::new(row),
                    };
                    *self.map.at_mut(point, self.width) =
                        self.place_wall_logic(point, create_new_walls(r));
                }
            }

            self.frame_from_current_state(100);
        }
        FloorBuilder {
            width: self.width,
            height: self.height,
            map: self.map,
            noise_map: self.noise_map,
            extra: Smoothed {},
            frames: self.frames,
        }
    }
}

impl<S: FloorBuilderState> FloorBuilder<S> {
    fn frame_from_current_state(&mut self, delay: u16) {
        if let Some(ref mut frames) = self.frames {
            println!("adding frame");
            frames.push(gif::Frame {
                width: self.width.as_unbounded() as u16,
                height: self.height.as_unbounded() as u16,
                buffer: Cow::Owned(self.map.iter().map(DungeonTile::as_u8).collect::<Vec<_>>()),
                delay,
                ..Default::default()
            });
        }
    }

    // ANCHOR state machine entry point
    pub(in crate::floor_builder) fn blank(
        width: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
        height: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
    ) -> FloorBuilder<Blank> {
        FloorBuilder {
            width,
            height,
            map: vec![
                Default::default();
                (width.as_unbounded() * height.as_unbounded())
                    .try_into()
                    .unwrap()
            ],
            noise_map: vec![
                Default::default();
                (width.as_unbounded() * height.as_unbounded())
                    .try_into()
                    .unwrap()
            ],
            extra: Blank {},
            // frames: Some(vec![]),
            frames: None,
        }
    }

    /// will only return wall or empty
    fn place_wall_logic(&self, point: Point, create_new_walls: bool) -> DungeonTile {
        use DungeonTile::{Empty, Wall};

        if self.is_out_of_bounds(point) {
            return Wall;
        }

        let num_walls_1_away = self.get_adjacent_walls(point, 1, 1);

        if self.map.at(point, self.width).is_wall() {
            if num_walls_1_away >= 4 {
                return Wall;
            }
            if create_new_walls && self.get_adjacent_walls(point, 2, 2) < 2 {
                return Wall;
            }
            if num_walls_1_away < 2 {
                return Empty;
            }
        } else if num_walls_1_away >= 5 {
            return Wall;
        }

        Empty
    }

    pub fn get_adjacent_walls(&self, point: Point, distance_x: i32, distance_y: i32) -> usize {
        let start_x = point.row.get().saturating_sub(distance_x);
        let start_y = point.column.get().saturating_sub(distance_y);
        let end_x = point.row.get().saturating_add(distance_x);
        let end_y = point.column.get().saturating_add(distance_y);

        let mut counter = 0;

        for i_y in start_y.range_to_inclusive(&end_y) {
            for i_x in start_x.range_to_inclusive(&end_x) {
                if !(i_x == point.row.get() && i_y == point.column.get())
                    && self.is_wall(Point {
                        row: Row::new(i_x),
                        column: Column::new(i_y),
                    })
                {
                    counter += 1;
                }
            }
        }
        counter
    }

    /// Considers out-of-bounds a wall
    fn is_wall(&self, point: Point) -> bool {
        if self.is_out_of_bounds(point) {
            return true;
        }

        if self.map.at(point, self.width).is_wall() {
            return true;
        }
        false
    }

    fn is_out_of_bounds(&self, point: Point) -> bool {
        // REVIEW: points can't be 0
        point.column.get() == 0.try_into().unwrap()
            || point.row.get() == 0.try_into().unwrap()
            || point.column.get() >= (self.width.as_unbounded() - 1).try_into().unwrap()
            || point.row.get() >= (self.height.as_unbounded() - 1).try_into().unwrap()
    }

    fn get_legal_neighbors(&self, point: Point) -> impl Iterator<Item = Point> + '_ {
        #[rustfmt::skip]
        let v = vec![
            point.saturating_add_row(1),
            point.saturating_add_column(1),
            point.saturating_sub_row(1),
            point.saturating_sub_column(1),
        ];

        v.into_iter()
            .unique()
            .filter(move |&p| !self.is_out_of_bounds(p) && p != point)
    }

    fn get_legal_neighbors_down_and_right(&self, point: Point) -> impl Iterator<Item = Point> + '_ {
        let down = point.saturating_add_row(1);
        let right = point.saturating_add_column(1);
        vec![down, right]
            .into_iter()
            .filter(move |&p| !self.is_out_of_bounds(p) && p != point)
    }

    pub(crate) fn _pretty(&self, extra_points: Vec<Point>, extra_points2: Vec<Point>) -> String {
        self.map
            // .par_iter()
            .chunks(self.width.as_unbounded() as usize)
            .zip(0..)
            .map(|i| {
                ANSIStrings(
                    &i.0.iter()
                        .zip(0..)
                        .map(|j| {
                            j.0._print(
                                extra_points2.contains(&Point {
                                    row: Row::new(i.1.try_into().unwrap()),
                                    column: Column::new(j.1.try_into().unwrap()),
                                }),
                                extra_points.contains(&Point {
                                    row: Row::new(i.1.try_into().unwrap()),
                                    column: Column::new(j.1.try_into().unwrap()),
                                }),
                                // true, true,
                            )
                        })
                        .collect::<Vec<_>>(),
                )
                .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod test_super {
    use crate::floor_builder::to_block_character::print_vec_2d;

    use super::*;

    #[test]
    pub(crate) fn test_blank_floor_generation() {
        let blank_floor =
            FloorBuilder::<Blank>::blank(10.try_into().unwrap(), 10.try_into().unwrap());

        assert!(blank_floor.height.as_unbounded() == 10);
        assert!(blank_floor.width.as_unbounded() == 10);
    }

    #[test]
    pub(crate) fn test_random_fill_generation() {
        let random_filled_floor =
            FloorBuilder::<Blank>::blank(50.try_into().unwrap(), 100.try_into().unwrap());
        let formatted = random_filled_floor._pretty(vec![], vec![]);

        println!("{}", &formatted)
    }

    #[test]
    fn test_is_out_of_bounds() {
        let width = 10.try_into().unwrap();
        let height = 15.try_into().unwrap();

        let blank_floor = FloorBuilder::<Blank>::blank(width, height);

        let mut new_vec = vec![false; (width.as_unbounded() * height.as_unbounded()) as usize];

        for column in width.expand_lower().range_from(&0.try_into().unwrap()) {
            for row in height.expand_lower().range_from(&0.try_into().unwrap()) {
                let point = Point {
                    column: Column::new(column),
                    row: Row::new(row),
                };
                *new_vec.at_mut(point, width) = blank_floor.is_out_of_bounds(point);
            }
        }

        println!("{}", print_vec_2d(new_vec, width));
    }
    #[test]
    fn test_get_legal_neighbors() {
        let width = 10.try_into().unwrap();
        let height = 15.try_into().unwrap();

        let blank_floor = FloorBuilder::<Blank>::blank(width, height);

        let mut new_vec = vec![false; (width.as_unbounded() * height.as_unbounded()) as usize];

        for column in width.expand_lower().range_from(&0.try_into().unwrap()) {
            for row in height.expand_lower().range_from(&0.try_into().unwrap()) {
                let point = Point {
                    column: Column::new(column),
                    row: Row::new(row),
                };
                *new_vec.at_mut(point, width) = blank_floor.is_out_of_bounds(point);
            }
        }

        println!("{}", print_vec_2d(new_vec, width));
    }
}
