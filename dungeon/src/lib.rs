pub use crate::dungeon_tile::DungeonTile;
use border::BorderId;
pub use floor_builder::FloorBuilder;
pub use point::*;
use serde::{Deserialize, Serialize};
use std::{convert::TryInto, fmt, num::NonZeroU16, usize};

use bounded_int;
// mod command;
// mod example;
mod border;
mod connection_path;
pub mod dungeon_tile;
mod floor_builder;
mod r#macro;
mod point;
pub use point::Point;
pub use point_index::PointIndex;
pub mod point_index;

use crate::{
    bounded_int::BoundedInt,
    floor_builder::{MAX_FLOOR_SIZE, MIN_FLOOR_SIZE},
};

// bench
pub fn create_dungeon(width: i32, height: i32) {
    let _ = Dungeon::new(
        height.try_into().unwrap(),
        width.try_into().unwrap(),
        NonZeroU16::new(10).unwrap(),
        DungeonType::Cave,
        false,
    );
}

// use ansi_term::ANSIStrings;
// use rayon::iter::{IntoParallelIterator, ParallelIterator};

// ANCHOR[id=connection]
#[derive(Debug, Clone, Copy, PartialEq)]
struct Connection {
    distance: f64,
    from: (Point, BorderId),
    to: (Point, BorderId),
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

        let mut image = vec![];
        {
            let mut encoder = Encoder::new(
                &mut image,
                self.floors[0].width.as_unbounded() as u16,
                self.floors[0].height.as_unbounded() as u16,
                &DungeonTile::COLOR_MAP,
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
    pub width: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
    pub height: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
    pub data: Vec<DungeonTile>,
}

impl Floor {
    pub fn new(
        id: FloorId,
        width: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
        height: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
        gif_output: bool,
    ) -> Self {
        FloorBuilder::create(id, width, height, gif_output)
    }

    pub fn iter_points_and_tiles(&self) -> impl Iterator<Item = (Point, &DungeonTile)> + '_ {
        let height = self.height.expand_lower();

        self.width
            .expand_lower()
            .range_from(&0.try_into().unwrap())
            .map(move |column| {
                height.range_from(&0.try_into().unwrap()).map(move |row| {
                    let point = Point {
                        column: Column::new(column),
                        row: Row::new(row),
                    };
                    (point, self.data.at(point, self.width))
                })
            })
            .flatten()
    }

    pub fn at(&self, point: Point) -> &DungeonTile {
        self.data.at(point, self.width)
    }

    pub fn at_mut(&mut self, point: Point) -> &mut DungeonTile {
        self.data.at_mut(point, self.width)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct FloorId(u16);

impl fmt::Display for FloorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

impl Dungeon {
    pub fn new(
        height: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
        width: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
        floor_count: NonZeroU16,
        dungeon_type: DungeonType,
        gif_output: bool,
    ) -> Self {
        Self {
            dungeon_type,
            floors: (0u16..floor_count.get())
                // .into_par_iter()
                .map(|id| FloorBuilder::create(FloorId(id), width, height, gif_output))
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
    (((from.row.get().as_unbounded() - to.row.get().as_unbounded()).pow(2)
        + (from.column.get().as_unbounded() - to.column.get().as_unbounded()).pow(2)) as f64)
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
