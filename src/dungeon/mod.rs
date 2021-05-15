mod floor_builder;
pub mod dungeon_tile;
use core::fmt;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;

pub use crate::dungeon::dungeon_tile::DungeonTile;

pub use self::floor_builder::floor_builder_state::Blank;
pub use self::floor_builder::FloorBuilder;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Connection {
    distance: f64,
    from: (BorderId, Point),
    to: (BorderId, Point),
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

// impl Dungeon<'_> {
//     pub fn into_binary(self) -> Vec<u8> {
//         // first byte is the dungeon type
//         let mut bytes = vec![self.dungeon_type as u8];

//         // now add the floors, separated by the FloorEnd control byte
//         let iter = self
//             .floors
//             .iter()
//             .map(|floor| {
//                 floor
//                     .iter()
//                     .map(|columns| {
//                         columns
//                             .iter()
//                             .map(|tile| *tile as u8)
//                             .chain(std::iter::once(BinaryDungeonControlByte::FloorRowEnd as u8))
//                     })
//                     .flatten()
//                     .chain(std::iter::once(BinaryDungeonControlByte::FloorEnd as u8))
//             })
//             .flatten();
//         bytes.extend(iter);

//         bytes
//     }

//     pub fn from_binary(bytes: Vec<u8>) -> Result<Self, DungeonDecodeError> {
//         let mut bytes = bytes.into_iter();
//         let dungeon_type = bytes
//             .next()
//             .map(TryInto::<DungeonType>::try_into)
//             .ok_or(DungeonDecodeError::UnexpectedEndOfBytes)??;

//         let mut floors: Vec<Floor> = vec![];

//         loop {
//             let mut floor_bytes = bytes
//                 .by_ref()
//                 .take_while(|b| *b != BinaryDungeonControlByte::FloorEnd as u8);
//             let mut rows = vec![];

//             loop {
//                 let row = floor_bytes
//                     .by_ref()
//                     .take_while(|b| *b != BinaryDungeonControlByte::FloorRowEnd as u8)
//                     .map(TryInto::<DungeonTile>::try_into)
//                     .collect::<Result<Vec<_>, _>>()?;
//                 if row.is_empty() {
//                     break;
//                 }
//                 rows.push(row.to_owned());
//             }
//             if rows.is_empty() {
//                 break;
//             }
//             floors.push(Floor(rows))
//         }

//         Ok(Dungeon {
//             dungeon_type,
//             floors,
//         })
//     }
// }

// #[test]
// fn test_dungeon_decoding() {
//     let dungeon1 = Dungeon::new(
//         NonZeroUsize::new(10).unwrap(),
//         NonZeroUsize::new(10).unwrap(),
//         NonZeroUsize::new(3).unwrap(),
//         DungeonType::Cave,
//     );

//     assert_eq!(
//         dungeon1,
//         Dungeon::from_binary(dungeon1.clone().into_binary()).unwrap()
//     )
// }

// #[test]
// fn test_dungeon_encoding() {
//     let dungeon = Dungeon::new(
//         NonZeroUsize::new(10).unwrap(),
//         NonZeroUsize::new(10).unwrap(),
//         NonZeroUsize::new(6).unwrap(),
//         DungeonType::Cave,
//     );

//     println!("{:X?}", dungeon.into_binary());
//     // println!("{}", dungeon.floors);
// }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Floor(Vec<Vec<DungeonTile>>);

impl Dungeon {
    pub fn new(
        height: NonZeroUsize,
        width: NonZeroUsize,
        floor_count: NonZeroUsize,
        dungeon_type: DungeonType,
    ) -> Self {
        Self {
            dungeon_type,
            floors: (0..floor_count.get())
                .into_par_iter()
                .map(|_| FloorBuilder::create(height, width))
                .collect(),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DungeonType {
    Cave,
    Forest,
}

fn distance(from: Point, to: Point) -> f64 {
    (((from.x as i64 - to.x as i64).pow(2) + (from.y as i64 - to.y as i64).pow(2)) as f64).sqrt()
}

#[cfg(test)]
mod test_dungeon {
    use super::*;

    #[test]
    fn test_serialize() {
        let dungeon = Dungeon::new(
            NonZeroUsize::new(100).unwrap(),
            NonZeroUsize::new(100).unwrap(),
            NonZeroUsize::new(10).unwrap(),
            DungeonType::Cave,
        );

        std::fs::write(
            "./test",
            rmp_serde::to_vec(&dungeon).unwrap()
            // serde_json::to_string_pretty(&dungeon).unwrap(),
        )
        .unwrap();
    }

    //     #[test]
    //     pub(crate) fn test_blank_floor_generation() {
    //         let blank_floor = FloorBuilder::blank(
    //             NonZeroUsize::new(10).unwrap(),
    //             NonZeroUsize::new(10).unwrap(),
    //         );

    //         assert!(blank_floor.height.get() == 10);
    //         assert!(blank_floor.width.get() == 10);
    //     }

    //     #[test]
    //     pub(crate) fn test_random_fill_generation() {
    //         let random_filled_floor = FloorBuilder::new(
    //             NonZeroUsize::new(50).unwrap(),
    //             NonZeroUsize::new(100).unwrap(),
    //         );
    //         let formatted = random_filled_floor.pretty(vec![], vec![]);

    //         println!("{}", &formatted)
    //     }

    //     #[test]
    //     pub(crate) fn test_border_finding() {
    //         let floor_builder = FloorBuilder::new(
    //             NonZeroUsize::new(100).unwrap(),
    //             NonZeroUsize::new(100).unwrap(),
    //         );
    //         let caves = floor_builder.get_cave_borders();
    //         let all_border_points = caves.iter().cloned().flatten().collect::<Vec<_>>();

    //         println!("{}", floor_builder.pretty(all_border_points, vec![]));
    //         let caves_pretty = caves
    //             .iter()
    //             .map(|v| {
    //                 v.iter()
    //                     .map(|point| format!("({}, {})", point.x, point.y))
    //                     .collect::<Vec<_>>()
    //             })
    //             .collect::<Vec<_>>();
    //         println!("caves = {:#?}", caves_pretty);
    //     }

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
