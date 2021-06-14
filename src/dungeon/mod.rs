pub mod dungeon_tile;
mod floor_builder;
// use ansi_term::ANSIStrings;
use core::fmt;
use std::collections::HashSet;
use std::iter;
// use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::{num::NonZeroUsize, usize};

pub use crate::dungeon::dungeon_tile::DungeonTile;
use crate::dungeon::floor_builder::bounded_int::BoundedInt;
use crate::dungeon::floor_builder::MAX_FLOOR_SIZE;
use crate::dungeon::floor_builder::MIN_FLOOR_SIZE;

pub use self::floor_builder::floor_builder_state::Blank;
pub use self::floor_builder::FloorBuilder;
pub use point::*;
mod point;

#[derive(Debug, Clone)]
pub(crate) struct ConnectionPath {
    start_border_id: BorderId,
    end_border_id: BorderId,
    path: ConnectionPathLength,
}

#[allow(dead_code)]
impl ConnectionPath {
    pub fn length(&self) -> usize {
        use ConnectionPathLength::*;
        match &self.path {
            Length1 { .. } => 1,
            Length2 { .. } => 2,
            // add 2 to include the start and end points
            Length3Plus { points, .. } => points.len() + 2,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Point> {
        use ConnectionPathLength::*;
        match &self.path {
            Length1 { point } => vec![*point],
            Length2 { start, end } => vec![*start, *end],
            Length3Plus { points, start, end } => iter::once(*start)
                .chain(points.clone())
                .chain(iter::once(*end))
                .collect(),
        }
        .into_iter()
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ConnectionPathLength {
    Length1 {
        point: Point,
    },
    Length2 {
        start: Point,
        end: Point,
    },
    Length3Plus {
        start: Point,
        end: Point,
        /// the points of the path between the start and the end, escluding start and end
        points: HashSet<Point>,
    },
}

#[derive(Clone)]
pub(crate) struct Border {
    pub id: BorderId,
    pub points: HashSet<Point>,
}

impl fmt::Debug for Border {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Border").field("id", &self.id).finish()
    }
}

// ANCHOR[id=connection]
#[derive(Debug, Clone, Copy, PartialEq)]
struct Connection {
    distance: f64,
    from: (Point, BorderId),
    to: (Point, BorderId),
}

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub(crate) struct BorderId(usize);

impl fmt::Debug for BorderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&*format!("{}", self.0))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dungeon {
    pub dungeon_type: DungeonType,
    pub floors: Vec<Floor>,
}

impl Dungeon {
    pub fn to_gif(&self) -> Vec<u8> {
        use gif::{Encoder, Frame, Repeat};
        use std::borrow::Cow;

        let color_map = &[
            0xFF, 0xFF, 0xFF, // black
            0x00, 0x00, 0x00, // white
            0xFF, 0x00, 0x00, // red
            0x00, 0xFF, 0x00, // green
            0x00, 0x00, 0xFF, // blue
        ];

        let mut image = vec![];
        {
            let mut encoder = Encoder::new(
                &mut image,
                self.floors[0].width.as_unbounded() as u16,
                self.floors[0].height.as_unbounded() as u16,
                color_map,
            )
            .unwrap();
            encoder.set_repeat(Repeat::Infinite).unwrap();
            for floor in &self.floors {
                let frame = Frame {
                    width: floor.width.as_unbounded() as u16,
                    height: floor.height.as_unbounded() as u16,
                    buffer: Cow::Owned(
                        floor
                            .data
                            .iter()
                            .map(DungeonTile::as_u8)
                            .collect::<Vec<_>>(),
                    ),
                    delay: 300,
                    ..Default::default()
                };
                encoder.write_frame(&frame).unwrap();
            }
        }
        image
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Floor {
    width: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
    height: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
    data: Vec<DungeonTile>,
}

impl Floor {
    pub fn new(
        width: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
        height: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
    ) -> Self {
        FloorBuilder::create(width, height)
    }

    // pub(crate) fn pretty(&self, extra_points: Vec<Point>, extra_points2: Vec<Point>) -> String {
    //     self.data
    //         .chunks(self.width)
    //         .zip(0i32..)
    //         .map(|i| {
    //             ANSIStrings(
    //                 &i.0.iter()
    //                     .zip(0i32..)
    //                     .map(|j| {
    //                         j.0.print(
    //                             extra_points2.contains(&Point {
    //                                 row: Row::new(i.1),
    //                                 column: Column::new(j.1),
    //                             }),
    //                             extra_points.contains(&Point {
    //                                 row: Row::new(i.1),
    //                                 column: Column::new(j.1),
    //                             }),
    //                         )
    //                     })
    //                     .collect::<Vec<_>>(),
    //             )
    //             .to_string()
    //         })
    //         .collect::<Vec<_>>()
    //         .join("\n")
    // }
}

impl Dungeon {
    pub fn new(
        height: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
        width: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
        floor_count: NonZeroUsize,
        dungeon_type: DungeonType,
    ) -> Self {
        Self {
            dungeon_type,
            floors: (0..floor_count.get())
                // .into_par_iter()
                .map(|_| FloorBuilder::create(width, height))
                .collect(),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DungeonType {
    Cave,
    Forest,
}

fn distance(from: Point, to: Point) -> f64 {
    (((*from.row.get() - *to.row.get()).pow(2) + (*from.column.get() - *to.column.get()).pow(2))
        as f64)
        .sqrt()
}

#[cfg(test)]
mod test_dungeon {
    use std::{convert::TryInto, fs, mem};

    use super::*;

    #[test]
    fn test_serialize() {
        let dungeon = Dungeon::new(
            50.try_into().unwrap(),
            50.try_into().unwrap(),
            NonZeroUsize::new(10).unwrap(),
            DungeonType::Cave,
        );

        let contents = rmp_serde::to_vec(&dungeon).unwrap();

        dbg!(contents.len() * mem::size_of::<u8>());

        fs::write("./test.mp", &contents).unwrap();

        let contents_json = serde_json::to_string(&dungeon).unwrap();

        dbg!(contents_json.len() * mem::size_of::<u8>());

        fs::write("./test.json", contents_json).unwrap();

        rmp_serde::from_read_ref::<_, Dungeon>(&contents).unwrap();
    }

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
