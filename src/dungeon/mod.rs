pub mod dungeon_tile;
mod floor_builder;
use ansi_term::ANSIStrings;
use core::fmt;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::{num::NonZeroUsize, usize};

pub use crate::dungeon::dungeon_tile::DungeonTile;

pub use self::floor_builder::floor_builder_state::Blank;
pub use self::floor_builder::FloorBuilder;
pub use point::*;
mod point;

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

impl Dungeon {
    pub fn to_gif(&self) -> Vec<u8> {
        use gif::{Encoder, Frame, Repeat};
        use std::borrow::Cow;

        let color_map = &[0xFF, 0xFF, 0xFF, 0, 0, 0, 0xFF, 0, 0];

        let mut image = vec![];
        {
            let mut encoder = Encoder::new(
                &mut image,
                self.floors[0].width as u16,
                self.floors[0].height as u16,
                color_map,
            )
            .unwrap();
            encoder.set_repeat(Repeat::Infinite).unwrap();
            for floor in &self.floors {
                let frame = Frame {
                    width: floor.width as u16,
                    height: floor.height as u16,
                    buffer: Cow::Owned(
                        floor
                            .data
                            .iter()
                            .map(DungeonTile::as_u8)
                            .collect::<Vec<_>>(),
                    ),
                    delay: 100,
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
    width: usize,
    height: usize,
    data: Vec<DungeonTile>,
}

impl Floor {
    pub fn new(width: usize, height: usize) -> Self {
        FloorBuilder::create(width, height)
    }

    pub(crate) fn pretty(&self, extra_points: Vec<Point>, extra_points2: Vec<Point>) -> String {
        self.data
            .chunks(self.width)
            .zip(0i32..)
            .map(|i| {
                ANSIStrings(
                    &i.0.iter()
                        .zip(0i32..)
                        .map(|j| {
                            j.0.print(
                                extra_points2.contains(&Point {
                                    row: Row::new(i.1),
                                    column: Column::new(j.1),
                                }),
                                extra_points.contains(&Point {
                                    row: Row::new(i.1),
                                    column: Column::new(j.1),
                                }),
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

impl Dungeon {
    pub fn new(
        height: usize,
        width: usize,
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
    (((from.row.get() as i64 - to.row.get() as i64).pow(2)
        + (from.column.get() as i64 - to.column.get() as i64).pow(2)) as f64)
        .sqrt()
}

#[cfg(test)]
mod test_dungeon {
    use std::{fs, mem};

    use super::*;

    #[test]
    fn test_serialize() {
        let dungeon = Dungeon::new(50, 50, NonZeroUsize::new(10).unwrap(), DungeonType::Cave);

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
        let blank_floor = FloorBuilder::<Blank>::blank(10, 10);

        assert!(blank_floor.height == 10);
        assert!(blank_floor.width == 10);
    }

    #[test]
    pub(crate) fn test_random_fill_generation() {
        let random_filled_floor = FloorBuilder::<Blank>::blank(50, 100);
        let formatted = random_filled_floor.pretty(vec![], vec![]);

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
