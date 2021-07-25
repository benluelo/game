#![warn(missing_docs)]
#![allow(clippy::needless_continue)]
#![warn(missing_docs, clippy::missing_docs_in_private_items)]

//! Dungeon creation. Creates 2d cave-like dungeons using cellular automata (among other techniques).
//!
//! # Examples
//! ```rust
//! let _ = Dungeon::new(
//!     height.try_into().unwrap(),
//!     width.try_into().unwrap(),
//!     NonZeroU16::new(10).unwrap(),
//!     DungeonType::Cave,
//!     false,
//! );
//! ```

pub use crate::dungeon_tile::DungeonTile;
use border::BorderId;
pub use floor_builder::FloorBuilder;
pub use point::*;
use serde::{Deserialize, Serialize};
use std::{convert::TryInto, fmt, num::NonZeroU16, usize};

/// The various things a tile can be in a dungeon floor.
///
/// See the type-level documentation for more information.
pub mod dungeon_tile;
/// A 1-dimensional type representing a 2-dimensional grid, indexable by a [`Point`].
///
/// See the type-level documentation for more information.
pub mod point_index;

mod border;
mod connection_path;
mod floor_builder;
mod point;

pub use point::Point;
pub use point_index::PointIndex;

use crate::floor_builder::{MAX_FLOOR_SIZE, MIN_FLOOR_SIZE};
use bounded_int::BoundedInt;

// ANCHOR[id=connection]
#[derive(Debug, Clone, Copy, PartialEq)]
struct Connection {
    distance: f64,
    from: (Point, BorderId),
    to: (Point, BorderId),
}

/// A 2D dungeon containing multiple floors of various sizes.
///
/// # Examples
/// ```rust
/// let _ = Dungeon::new(
///     height.try_into().unwrap(),
///     width.try_into().unwrap(),
///     NonZeroU16::new(10).unwrap(),
///     DungeonType::Cave,
///     false,
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dungeon {
    /// The type of the dungeon. This only affects the way the dungeon
    /// is presented aesthetically.
    pub dungeon_type: DungeonType,
    /// The floors of the dungeon. Will never be empty.
    // TODO: Maybe use https://docs.rs/vec1/1.8.0/vec1/? It seems to be fairly well maintained.
    pub floors: Vec<Floor>,
}

impl Dungeon {
    /// Encodes the dungeon to a gif, with each floor being a frame, and
    /// returns the image as bytes.
    #[must_use]
    pub fn to_gif(&self) -> Vec<u8> {
        use gif::{Encoder, Frame, Repeat};
        use std::borrow::Cow;

        let mut image = vec![];
        {
            let mut encoder = Encoder::new(
                &mut image,
                self.floors
                    .iter()
                    .reduce(|a, b| if a.width >= b.width { a } else { b })
                    .unwrap()
                    .width
                    .as_unbounded()
                    .try_into()
                    .unwrap(),
                self.floors
                    .iter()
                    .reduce(|a, b| if a.height >= b.height { a } else { b })
                    .unwrap()
                    .height
                    .as_unbounded()
                    .try_into()
                    .unwrap(),
                &DungeonTile::COLOR_MAP,
            )
            .unwrap();
            encoder.set_repeat(Repeat::Infinite).unwrap();
            for floor in &self.floors {
                let frame = Frame {
                    width: floor.width.as_unbounded().try_into().unwrap(),
                    height: floor.height.as_unbounded().try_into().unwrap(),
                    buffer: Cow::Owned(
                        floor
                            .data
                            .iter()
                            .map(DungeonTile::as_u8)
                            .collect::<Vec<_>>(),
                    ),
                    delay: 300,
                    ..Frame::default()
                };
                encoder.write_frame(&frame).unwrap();
            }
        }
        image
    }
}

/// A floor of a [`Dungeon`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Floor {
    /// width
    pub width: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
    /// height
    pub height: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
    /// data
    pub data: Vec<DungeonTile>,
}

impl Floor {
    /// Creates a new floor with the given parameters.
    #[must_use]
    pub fn new(
        id: FloorId,
        width: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
        height: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
        gif_output: bool,
    ) -> Self {
        FloorBuilder::create(id, width, height, gif_output)
    }

    /// Returns an iterator over the tiles in the floor and their respective [`Point`].
    pub fn iter_points_and_tiles(&self) -> impl Iterator<Item = (Point, &DungeonTile)> + '_ {
        let height = self.height.expand_lower();

        self.width
            .expand_lower()
            .range_from(0.try_into().unwrap())
            .flat_map(move |column| {
                height.range_from(0.try_into().unwrap()).map(move |row| {
                    let point = Point {
                        column: Column::new(column),
                        row: Row::new(row),
                    };
                    (point, self.data.at(point, self.width))
                })
            })
    }

    /// Returns a refrence to the tile at the specified point.
    #[must_use]
    pub fn at(&self, point: Point) -> &DungeonTile {
        self.data.at(point, self.width)
    }

    /// Returns a mutable reference to the tile at the specified point.
    #[must_use]
    pub fn at_mut(&mut self, point: Point) -> &mut DungeonTile {
        self.data.at_mut(point, self.width)
    }
}

/// A unique, opaque ID assigned to each floor upon creation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct FloorId(u16);

impl fmt::Display for FloorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

impl Dungeon {
    /// Creates a new dungeon with the specified paramaters.
    #[must_use]
    pub fn new(
        height: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
        width: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
        floor_count: NonZeroU16,
        dungeon_type: DungeonType,
        gif_output: bool,
    ) -> Self {
        Self {
            dungeon_type,
            floors: (0_u16..floor_count.get())
                // .into_par_iter()
                .map(|id| FloorBuilder::create(FloorId(id), width, height, gif_output))
                .collect(),
        }
    }

    /// Returns the dungeon as JSON.
    ///
    /// # Errors
    /// This function should never error given the implementation of [`Serialize`] for [`Dungeon`],
    /// however the [`Result`] is still returned from the call to [`serde_json::to_string`].
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// The different types of dungeon a [`Dungeon`] can be.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DungeonType {
    /// A cave dungeon.
    ///
    /// Should be rocky and have a 'gloomy' atmosphere to it.
    Cave,
    /// A forest dungeon.
    ///
    /// Should be lucious, overgrown, and *very* green.
    Forest,
}

// TODO: Make this an instance method on [`Point`].
fn distance(from: Point, to: Point) -> f64 {
    (((from.row.get().as_unbounded() - to.row.get().as_unbounded()).pow(2)
        + (from.column.get().as_unbounded() - to.column.get().as_unbounded()).pow(2)) as f64)
        .sqrt()
}

#[cfg(test)]
mod test_dungeon {
    use std::{convert::TryInto, fs};

    use super::*;

    #[test]
    fn test_serialize() {
        let dungeon = Dungeon::new(
            50.try_into().unwrap(),
            50.try_into().unwrap(),
            NonZeroU16::new(10).unwrap(),
            DungeonType::Cave,
            false,
        );

        let contents = rmp_serde::to_vec(&dungeon).unwrap();

        // dbg!(contents.len() * mem::size_of::<u8>());

        fs::write("./test.mp", &contents).unwrap();

        let contents_json = serde_json::to_string(&dungeon).unwrap();

        // dbg!(contents_json.len() * mem::size_of::<u8>());

        fs::write("./test.json", contents_json).unwrap();

        rmp_serde::from_read_ref::<_, Dungeon>(&contents).unwrap();
    }

    // #[test]
    // pub(crate) fn test_border_finding() {
    //     let floor_builder = FloorBuilder::<Blank>::blank(50, 100);
    //     let caves = floor_builder.get_cave_borders();
    //     let all_border_points = caves.iter().cloned().flatten().collect::<Vec<_>>();

    //     println!("{}", floor_builder.pretty(all_border_points, vec![]));
    //     let caves_pretty = caves
    //         .iter()
    //         .map(|v| {
    //             v.iter()
    //                 .map(|point| format!("({}, {})", point.x, point.y))
    //                 .collect::<Vec<_>>()
    //         })
    //         .collect::<Vec<_>>();
    //     println!("caves = {:#?}", caves_pretty);
    // }

    // #[test]
    // pub(crate) fn test_cave_connections() {
    //     let mut floor_builder = FloorBuilder::create(
    //         NonZeroUsize::new(50).unwrap(),
    //         NonZeroUsize::new(100).unwrap(),
    //     );
    //     let caves = floor_builder.get_cave_borders();
    //     let connections = floor_builder.build_connections(caves);
    //     floor_builder.draw_connections(connections.clone());
    //     floor_builder.smoothen(7, |_| false);
    //     println!(
    //         "{}",
    //         floor_builder.pretty(
    //             connections
    //                 .into_iter()
    //                 .map(|i| vec![i.0 .1, i.1 .1])
    //                 .flatten()
    //                 .collect(),
    //             vec![]
    //         )
    //     );
    // }

    //     // #[test]
    //     // fn test_border_connections() {
    //     //     let mut floor_builder = FloorBuilder::new(
    //     //         NonZeroUsize::new(50).unwrap(),
    //     //         NonZeroUsize::new(100).unwrap(),
    //     //     );
    //     //     let caves = floor_builder.get_cave_borders();
    //     //     let connections = floor_builder.build_connections(caves);
    //     //     floor_builder.draw_connections(connections);
    //     //     floor_builder.smoothen(7, |_| false);

    //     //     assert!(floor_builder.get_cave_borders().len() == 1);
    //     // }
}
