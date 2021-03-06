#[allow(clippy::wildcard_imports)]
use crate::{
    dungeon_tile::DungeonTile,
    floor_builder::{floor_builder_state::*, to_block_character::ToAsciiCharacter},
    point_index::PointIndex,
    Column, FloorId, Point, Row,
};
use bounded_int::BoundedInt;
use gif::Frame;
use itertools::Itertools;
use pathfinding::prelude::dijkstra;

use std::{borrow::Cow, convert::TryInto, fmt::Debug, vec};

use self::floor_builder_state::{blank::Blank, smoothed::Smoothed};

mod floor_builder_state;

/// Represents a type that can be 'pretty-printed' using ascii characters.
///
/// See the type-level documentation for more information.
pub(crate) mod to_block_character;

/// The minimum dimensions a [`Floor`](crate::Floor) can have.
pub const MIN_FLOOR_SIZE: i32 = 10;

/// The maximum dimensions a [`Floor`](crate::Floor) can have.
pub const MAX_FLOOR_SIZE: i32 = 200;

/// The percent chance of a wall being placed during the initial noise
/// generation.
const RANDOM_FILL_WALL_PERCENT_CHANCE: u8 = 52;

/// Builder struct for a [`Floor`](crate::Floor).
///
/// See <http://roguebasin.roguelikedevelopment.org/index.php?title=Cellular_Automata_Method_for_Generating_Random_Cave-Like_Levels>
#[derive(Debug)]
pub struct FloorBuilder<S: FloorBuilderState> {
    /// The width of the floor.
    pub(crate) width: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
    /// The height of the floor.
    pub(crate) height: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
    /// The map of tiles.
    pub(crate) map: Vec<DungeonTile>,
    /// The map containing the noise for the floor builder, used in various
    /// places during the build process.
    pub(crate) noise_map: Vec<u16>,
    /// The current state of the floor builder. May or may not contain extra
    /// data to be used at that stage of generation.
    extra: S,
    /// The frames of the floor builder as it is being built. Defaults to
    /// [`None`] unless specified to output to a gif.
    frames: Option<Vec<gif::Frame<'static>>>,
    /// A unique, opaque ID assigned to the floor builder upon creation.
    id: FloorId,
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
    /// Smooths out the map using cellular automata.
    fn smoothen(
        mut self,
        repeat: usize,
        create_new_walls: fn(usize) -> bool,
    ) -> FloorBuilder<Smoothed> {
        for r in 0..repeat {
            for column in self.width.expand_lower().range_from(0.try_into().unwrap()) {
                for row in self.height.expand_lower().range_from(0.try_into().unwrap()) {
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
            id: self.id,
        }
    }
}

impl<S: FloorBuilderState> FloorBuilder<S> {
    /// Dumps the current state of the builder to stdout.
    //
    /// for use in debugging only
    fn inspect(self) -> Self {
        dbg!(S::TYPE_NAME);
        self
    }

    /// Creates a [`gif::Frame`] from the current state of the floor builder,
    /// storing it in [`FloorBuilder::frames`].
    ///
    /// Note that this is essentially a noop if `self.frames` is [`None`] (i.e.,
    /// gif output has not been enabled).
    fn frame_from_current_state(&mut self, delay: u16) {
        if let Some(ref mut frames) = self.frames {
            // println!("adding frame");
            frames.push(Frame {
                width: self.width.as_unbounded().try_into().unwrap(),
                height: self.height.as_unbounded().try_into().unwrap(),
                buffer: Cow::Owned(self.map.iter().map(DungeonTile::as_u8).collect::<Vec<_>>()),
                delay,
                ..Frame::default()
            });
        }
    }

    // ANCHOR state machine entry point
    /// Creates a blank [`FloorBuilder`], with all the values set to their
    /// defaults.
    pub(in crate::floor_builder) fn blank(
        id: FloorId,
        width: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
        height: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
        gif_output: bool,
    ) -> FloorBuilder<Blank> {
        FloorBuilder {
            width,
            height,
            map: vec![
                DungeonTile::default();
                (width.as_unbounded() * height.as_unbounded())
                    .try_into()
                    .unwrap()
            ],
            noise_map: vec![
                u16::default();
                (width.as_unbounded() * height.as_unbounded())
                    .try_into()
                    .unwrap()
            ],
            extra: Blank {},
            frames: if gif_output { Some(vec![]) } else { None },
            // frames: ,
            id,
        }
    }

    /// will only return wall or empty
    fn place_wall_logic(&self, point: Point, create_new_walls: bool) -> DungeonTile {
        use DungeonTile::{Empty, Wall};

        let what_the_tile_is_currently = self.map.at(point, self.width);

        if !matches!(what_the_tile_is_currently, Empty | Wall) {
            return *what_the_tile_is_currently;
        }

        if self.is_out_of_bounds(point) {
            return Wall;
        }

        let num_walls_1_away = self.get_adjacent_walls(point, 1, 1);

        if self.map.at(point, self.width).is_solid() {
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

    /// Returns how many walls there are within the rectangle bounded by
    /// `distance_x` left and right of the point and `distance_y` above and
    /// below the point.
    pub fn get_adjacent_walls(&self, point: Point, distance_x: i32, distance_y: i32) -> usize {
        let start_x = point.row.get().saturating_sub(distance_x);
        let start_y = point.column.get().saturating_sub(distance_y);
        let end_x = point.row.get().saturating_add(distance_x);
        let end_y = point.column.get().saturating_add(distance_y);

        let mut counter = 0;

        for i_y in start_y.range_to_inclusive(end_y) {
            for i_x in start_x.range_to_inclusive(end_x) {
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
    pub fn is_wall(&self, point: Point) -> bool {
        if self.is_out_of_bounds(point) {
            return true;
        }

        if self.map.at(point, self.width).is_solid() {
            return true;
        }
        false
    }

    /// Considers the 1-wide border around the edge of the map to be out of
    /// bounds.
    fn is_out_of_bounds(&self, point: Point) -> bool {
        // REVIEW: points can't be 0
        point.column.get() == 0.try_into().unwrap()
            || point.row.get() == 0.try_into().unwrap()
            || point.column.get() >= (self.width.as_unbounded() - 1).try_into().unwrap()
            || point.row.get() >= (self.height.as_unbounded() - 1).try_into().unwrap()
    }

    /// Gets the 8 neighbours around the specified point that aren't out of
    /// bounds.
    ///
    /// ```txt
    /// x x x
    /// x p x
    /// x x x
    /// ```
    fn get_legal_neighbors_with_diagonals(&self, point: Point) -> impl Iterator<Item = Point> + '_ {
        #[rustfmt::skip]
        let v = vec![
            point.saturating_sub_row(1)
                 .saturating_sub_column(1), point.saturating_sub_row(1), point.saturating_sub_row(1)
                                                                              .saturating_add_row(1),

            point.saturating_sub_column(1), point,                       point.saturating_add_column(1),

            point.saturating_add_row(1)
                 .saturating_sub_column(1), point.saturating_add_row(1), point.saturating_add_row(1)
                                                                              .saturating_add_row(1),
        ];

        v.into_iter()
            .unique()
            .filter(move |&p| !self.is_out_of_bounds(p) && p != point)
    }

    /// Gets the 4 neighbours around the specified point that aren't out of
    /// bounds.
    ///
    /// ```txt
    /// o x o
    /// x p x
    /// o x o
    /// ```
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

    /// Gets the 2 neighbours around the specified point that aren't out of
    /// bounds, below it and to the right.
    ///
    /// ```txt
    /// o o o
    /// o p x
    /// o x 0
    /// ```
    fn get_legal_neighbors_down_and_right(&self, point: Point) -> impl Iterator<Item = Point> + '_ {
        let down = point.saturating_add_row(1);
        let right = point.saturating_add_column(1);
        vec![down, right]
            .into_iter()
            .filter(move |&p| !self.is_out_of_bounds(p) && p != point)
    }

    /// Pretty-prints the map in it's current state. Used for debugging and
    /// tests.
    #[allow(unused_variables)]
    pub(crate) fn _pretty(&self, extra_points: &[Point], extra_points2: &[Point]) -> String {
        self.map
            // .par_iter()
            .chunks(self.width.as_unbounded() as usize)
            .map(|i| {
                i.iter()
                    .flat_map(ToAsciiCharacter::to_ascii_chars)
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod test_super {
    use crate::floor_builder::to_block_character::_print_vec_2d;

    use super::*;

    #[test]
    pub(crate) fn test_blank_floor_generation() {
        let blank_floor = FloorBuilder::<Blank>::blank(
            FloorId(0),
            10.try_into().unwrap(),
            10.try_into().unwrap(),
            false,
        );

        assert!(blank_floor.height.as_unbounded() == 10);
        assert!(blank_floor.width.as_unbounded() == 10);
    }

    #[test]
    pub(crate) fn test_random_fill_generation() {
        let random_filled_floor = FloorBuilder::<Blank>::blank(
            FloorId(0),
            50.try_into().unwrap(),
            100.try_into().unwrap(),
            false,
        );
        let formatted = random_filled_floor._pretty(&[], &[]);

        println!("{}", &formatted)
    }

    #[test]
    fn test_is_out_of_bounds() {
        let width = 10.try_into().unwrap();
        let height = 15.try_into().unwrap();

        let blank_floor = FloorBuilder::<Blank>::blank(FloorId(0), width, height, false);

        let mut new_vec = vec![false; (width.as_unbounded() * height.as_unbounded()) as usize];

        for column in width.expand_lower().range_from(0.try_into().unwrap()) {
            for row in height.expand_lower().range_from(0.try_into().unwrap()) {
                let point = Point {
                    column: Column::new(column),
                    row: Row::new(row),
                };
                *new_vec.at_mut(point, width) = blank_floor.is_out_of_bounds(point);
            }
        }

        println!("{}", _print_vec_2d(&new_vec, width));
    }
    #[test]
    fn test_get_legal_neighbors() {
        let width = 10.try_into().unwrap();
        let height = 15.try_into().unwrap();

        let blank_floor = FloorBuilder::<Blank>::blank(FloorId(0), width, height, false);

        let mut new_vec = vec![false; (width.as_unbounded() * height.as_unbounded()) as usize];

        for column in width.expand_lower().range_from(0.try_into().unwrap()) {
            for row in height.expand_lower().range_from(0.try_into().unwrap()) {
                let point = Point {
                    column: Column::new(column),
                    row: Row::new(row),
                };
                *new_vec.at_mut(point, width) = blank_floor.is_out_of_bounds(point);
            }
        }

        println!("{}", _print_vec_2d(&new_vec, width));
    }
}
