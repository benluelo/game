use crate::dungeon::{
    distance, dungeon_tile::DungeonTile, floor_builder::floor_builder_state::*, Border, BorderId,
    Column, Connection, ConnectionPath, ConnectionPathLength, Floor, Point, Row,
};
use ansi_term::ANSIStrings;
use itertools::Itertools;
use noise::{Billow, MultiFractal, NoiseFn, Seedable};
use pathfinding::prelude::dijkstra;
use petgraph::{
    algo::{kosaraju_scc, min_spanning_tree},
    data::FromElements,
    graphmap::UnGraphMap,
};
use rand::{
    prelude::{SliceRandom, StdRng},
    thread_rng, Rng, SeedableRng,
};

use std::{
    collections::{HashMap, HashSet, VecDeque},
    convert::TryInto,
    fmt::Debug,
    iter,
};

pub mod floor_builder_state;

pub const MIN_FLOOR_SIZE: usize = 10;

/// Represents a floor of a dungeon
/// See http://roguebasin.roguelikedevelopment.org/index.php?title=Cellular_Automata_Method_for_Generating_Random_Cave-Like_Levels
#[derive(Debug)]
pub struct FloorBuilder<S: FloorBuilderState> {
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) map: Vec<DungeonTile>,
    pub(crate) noise_map: Vec<u128>,
    extra: S,
}

impl FloorBuilder<New> {
    pub(crate) fn create(width: usize, height: usize) -> Floor {
        assert!(
            width >= MIN_FLOOR_SIZE,
            "floor width too small: {}; minimum is {}",
            width,
            MIN_FLOOR_SIZE
        );
        assert!(
            height >= MIN_FLOOR_SIZE,
            "floor height too small: {}; minimum is {}",
            height,
            MIN_FLOOR_SIZE
        );
        let fb =
            FloorBuilder::<Blank>::blank(width.try_into().unwrap(), height.try_into().unwrap())
                .random_fill()
                .smoothen(7, |r| r < 4)
                .get_cave_borders()
                .build_connections(BuildConnectionIterations::Finite(20))
                .trace_connection_paths(true, true)
                .draw(|_, _, _| DungeonTile::Empty)
                .smoothen(7, |_| false)
                .check_for_secret_passages()
                .draw(|is_first, is_last, _| {
                    if is_first || is_last {
                        dbg!(is_first);
                        dbg!(is_last);
                        println!();
                        DungeonTile::SecretDoor { requires_key: true }
                    } else {
                        DungeonTile::SecretPassage
                    }
                });
        fb.finish()
    }
}

impl FloorBuilder<Filled> {
    fn finish(self) -> Floor {
        Floor {
            height: self.height.try_into().unwrap(),
            width: self.width.try_into().unwrap(),
            data: self.map,
        }
    }
}

impl FloorBuilder<Blank> {
    pub fn random_fill(mut self) -> FloorBuilder<Filled> {
        let mut rng = thread_rng();

        let noise = Billow::new().set_seed(rng.gen()).set_persistence(128.0);

        // build initial maps (walls and noise)
        // TODO: extract magic numbers to contants
        for column in 0i32..self.width {
            for row in 0i32..self.height {
                let point = Point {
                    column: Column::new(column),
                    row: Row::new(row),
                };
                let float = noise.get([
                    (column as f64 / self.width as f64) + 0.1,
                    (row as f64 / self.height as f64) + 0.1,
                ]) + 0.001;
                *self.noise_map.at_mut(point, self.width) =
                    (float.abs() * 10000.0).powi(2).floor() as u128;

                // make a wall some percent of the time
                *self.map.at_mut(point, self.width) = if 52 >= rng.gen_range(0..101) {
                    DungeonTile::Wall
                } else {
                    DungeonTile::Empty
                }
            }
        }

        // println!("{}\n", self.pretty(vec![], vec![]));

        // ANCHOR: dijkstra
        // find path through noise map and apply path to walls map
        let goal = Point {
            row: Row::new(self.height - 4),
            column: Column::new(self.width - 4),
        };
        let (found_path, _) = dijkstra(
            &Point {
                row: Row::new(4),
                column: Column::new(4),
            },
            |&point| {
                self.get_legal_neighbors(point)
                    .map(|p| (p, *self.noise_map.at(p, self.width)))
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
        }

        FloorBuilder {
            extra: Filled {},
            height: self.height,
            width: self.width,
            map: self.map,
            noise_map: self.noise_map,
        }
    }
}

impl FloorBuilder<Drawable> {
    fn draw(
        mut self,
        draw_with: fn(
            // is_first
            bool,
            // is_last
            bool,
            Point,
        ) -> DungeonTile,
    ) -> FloorBuilder<Filled> {
        for path in self.extra.to_draw.into_iter() {
            use ConnectionPathLength::*;
            match &path.path {
                Length1 { point } => {
                    *self.map.at_mut(*point, self.width) = draw_with(true, true, *point)
                }
                Length2 { start, end } => {
                    *self.map.at_mut(*start, self.width) = draw_with(true, false, *start);
                    *self.map.at_mut(*end, self.width) = draw_with(false, true, *end);
                }
                Length3Plus { points, start, end } => {
                    assert!(!points.contains(start));
                    assert!(!points.contains(end));
                    *self.map.at_mut(*start, self.width) = draw_with(true, false, *start);
                    *self.map.at_mut(*end, self.width) = draw_with(false, true, *end);

                    for point in points {
                        *self.map.at_mut(*point, self.width) = draw_with(false, false, *point);
                    }
                }
            };
        }

        FloorBuilder {
            width: self.width,
            height: self.height,
            map: self.map,
            noise_map: self.noise_map,
            extra: Filled {},
        }
    }
}

impl FloorBuilder<HasConnections> {
    /// takes the connections from [`FloorBuilder::build_connections`] and traces paths between them, leaving the paths in the `to_draw` state of `FloorBuilder<Drawable>`.
    fn trace_connection_paths(self, wide: bool, use_noise_map: bool) -> FloorBuilder<Drawable> {
        let to_draw = self
            .extra
            .connections
            .iter()
            .map(|(&(from, from_id), &(to, to_id))| {
                let mut rng = StdRng::from_entropy();

                let (path, _) = dijkstra(
                    &from,
                    |&point| {
                        self.get_legal_neighbors(point).map(|p| {
                            (
                                p,
                                if use_noise_map {
                                    *self.noise_map.at(p, self.width)
                                } else {
                                    1
                                },
                            )
                        })
                    },
                    |&point| !self.is_out_of_bounds_usize(point) && (point == to),
                )
                .expect("no path found");

                // let mut all_points = vec![];

                // TODO: figure out what this was for lol
                // match path.len() {
                //     // 1 or two long, all of the passages will become doors
                //     1 | 2 => {}
                //     // 3 or more long, make sure
                //     _ => {}
                // }

                // REVIEW: use a `try_*` iterator?

                let all_points = path
                    .into_iter()
                    .enumerate()
                    .filter_map(|(_index, point)| {
                        // if an empty point is found anywhere beside the path, the path is finished
                        // make sure to not check for the first point (hence the enumerate)
                        // FIXME: somehow make sure the points aren't from the starting cave
                        if self.map.at(point, self.width).is_empty() {
                            None
                        } else {
                            Some(point)
                        }
                    })
                    .collect::<Vec<_>>();

                ConnectionPath {
                    start_border_id: from_id,
                    end_border_id: to_id,
                    path: match all_points.as_slice() {
                        [point] => ConnectionPathLength::Length1 { point: *point },
                        [start, end] => ConnectionPathLength::Length2 {
                            start: *start,
                            end: *end,
                        },
                        _ => ConnectionPathLength::Length3Plus {
                            points: all_points
                                .iter()
                                .copied()
                                .chain(if wide {
                                    all_points
                                        .iter()
                                        .flat_map(|&point| match rng.gen_bool(0.5) {
                                            true => self
                                                .get_legal_neighbors_down_and_right(point)
                                                .collect::<Vec<_>>(),
                                            false => {
                                                self.get_legal_neighbors(point).collect::<Vec<_>>()
                                            }
                                        })
                                        .collect_vec()
                                } else {
                                    vec![]
                                })
                                .filter(move |v| *v != from && *v != to)
                                .collect::<HashSet<_>>(),
                            start: from,
                            end: to,
                        },
                    },
                }
            })
            .collect();

        FloorBuilder {
            width: self.width,
            height: self.height,
            map: self.map,
            noise_map: self.noise_map,
            extra: Drawable { to_draw },
        }
    }
}

pub enum BuildConnectionIterations {
    FullyConnect,
    Finite(u8),
    // until there are `x` amount of sccs
    Until(u8),
}

impl BuildConnectionIterations {
    pub fn as_range(&self) -> Box<dyn Iterator<Item = u8>> {
        match self {
            BuildConnectionIterations::FullyConnect => {
                println!("fully connect bb");
                Box::new(iter::repeat(u8::MAX))
            }
            BuildConnectionIterations::Finite(amount) => {
                println!("finite: {}", &amount);
                Box::new(0..*amount)
            }
            BuildConnectionIterations::Until(_) => todo!(),
        }
    }
}

#[cfg(test)]
mod test_build_connection_iterations {
    use super::*;

    #[test]
    fn test_finite() {
        let to_zip = vec![1, 2, 3, 4, 5];
        for i in BuildConnectionIterations::Finite(10)
            .as_range()
            .zip(&to_zip)
        {
            println!("{:?}", i);
        }
    }
    #[test]
    fn test_infinite() {
        let to_zip = vec![1, 2, 3, 4, 5];
        for i in BuildConnectionIterations::FullyConnect
            .as_range()
            .zip(to_zip)
        {
            println!("{:?}", i);
        }
    }
}

impl FloorBuilder<HasBorders> {
    /// build bridges between the disjointed caves and the closest cave border point *not* in the border of said disjointed cave
    fn build_connections(
        self,
        iterations: BuildConnectionIterations,
    ) -> FloorBuilder<HasConnections> {
        if self.extra.borders.len() == 1 {
            return FloorBuilder {
                width: self.width,
                height: self.height,
                map: self.map,
                noise_map: self.noise_map,
                extra: HasConnections {
                    ..Default::default()
                },
            };
        }

        let all_border_points = self
            .extra
            .borders
            .iter()
            .cloned()
            .map(|Border { id, points }| points.into_iter().zip(iter::repeat(id)))
            .flatten()
            .collect::<HashMap<Point, BorderId>>();

        let mut connections_with_points = HashMap::<(Point, BorderId), (Point, BorderId)>::new();
        // a graph is required to check for the strongly connected components and the
        // minimum spanning tree (because i don't want to implement that myself lol)
        let mut connected_borders_graph = UnGraphMap::<BorderId, ()>::new();

        for id in self.extra.borders.iter().map(|b| b.id) {
            connected_borders_graph.add_node(id);
        }

        // loop through all the borders
        // build one connection per loop
        dbg!(self.extra.borders.len());
        dbg!(iterations
            .as_range()
            .zip(&self.extra.borders)
            .collect::<Vec<_>>());
        for (_, current_border) in iterations.as_range().zip(&self.extra.borders) {
            dbg!(current_border.id);
            let already_connected_ids = connected_borders_graph
                .neighbors(current_border.id)
                .collect::<Vec<_>>();

            let maybe_new_connection: Option<Connection> = all_border_points
                .iter()
                // filter out border points that are either:
                // - in the current border, or
                .filter(|(_, &id)| id != current_border.id)
                // - in a border the current border is already connected to
                .filter(|(_, &id)| !already_connected_ids.contains(&id))
                .flat_map(|(&point, &id)| {
                    // create a `Connection` between every point in this border and the borders it isn't already connected to
                    // LINK src/dungeon/mod.rs#connection
                    current_border
                        .points
                        .iter()
                        .map(move |&current_border_point| Connection {
                            distance: distance(point, current_border_point),
                            from: (current_border_point, current_border.id),
                            to: (point, id),
                        })
                })
                // find the point that's closest to the current border
                .reduce(|prev, curr| {
                    if prev.distance < curr.distance {
                        prev
                    } else {
                        curr
                    }
                });

            if let Some(Connection { from, to, .. }) = maybe_new_connection {
                connections_with_points.insert(from, to);
                connected_borders_graph.add_edge(from.1, to.1, ());
            }

            // strongly connected components
            let sccs = kosaraju_scc(&connected_borders_graph);

            // if there is only one scc, we're done here
            dbg!(&sccs);
            if sccs.len() == 1 {
                let msf = UnGraphMap::from_elements(min_spanning_tree(
                    &connected_borders_graph.into_graph::<usize>(),
                ));
                return FloorBuilder {
                    width: self.width,
                    height: self.height,
                    map: self.map,
                    noise_map: self.noise_map,
                    extra: HasConnections {
                        // remove extra connections fro mthe connections_with_points hashmap (make it into the MSF)
                        connections: connections_with_points
                            .into_iter()
                            .filter(|&((_, k), (_, v))| msf.contains_edge(k, v))
                            .collect(),
                        borders: self.extra.borders.into_iter().map(|b| (b.id, b)).collect(),
                    },
                };
            }
        }
        FloorBuilder {
            width: self.width,
            height: self.height,
            map: self.map,
            noise_map: self.noise_map,
            extra: HasConnections {
                connections: connections_with_points,
                borders: self.extra.borders.into_iter().map(|b| (b.id, b)).collect(),
            },
        }
    }
}

impl FloorBuilder<Smoothed> {
    fn get_cave_borders(self) -> FloorBuilder<HasBorders> {
        let mut already_visited = vec![false; (self.width * self.height).try_into().unwrap()];

        let mut borders = vec![];

        // loop through the entire map
        for column in 0..self.width {
            'rows: for row in 0..self.height {
                let point = Point {
                    column: Column::new(column),
                    row: Row::new(row),
                };
                // if the point has already been visited (by either the main loop or the cave searching) then continue looping through the map
                if *already_visited.at(point, self.width) {
                    continue 'rows;
                }
                // otherwise, mark the point as visited
                *already_visited.at_mut(point, self.width) = true;

                // if there's an empty space at the point, BFS to find the border of the cave (no diagonals)
                if self.map.at(point, self.width).is_empty() {
                    let mut border = HashSet::new();

                    let mut queue = self.get_legal_neighbors(point).collect::<VecDeque<_>>();

                    loop {
                        if let Some(point) = queue.pop_front() {
                            // if point is empty, mark it as visited and then add all of it's
                            // legal neighbors to the queue
                            if self.map.at(point, self.width).is_empty() {
                                if *already_visited.at(point, self.width) {
                                    continue;
                                }
                                *already_visited.at_mut(point, self.width) = true;
                                self.get_legal_neighbors(point)
                                    .for_each(|p| queue.push_back(p));
                            } else {
                                border.insert(point);
                            }
                        } else {
                            if !border.is_empty() {
                                println!("found border");
                                // add the found cave to the collection of all caves
                                borders.push(border);
                            }
                            continue 'rows;
                        }
                    }
                }
            }
        }
        let mut vec_of_borders = borders
            .iter()
            .enumerate()
            .map(|(id, hashset)| Border {
                id: BorderId(id),
                points: hashset.clone(),
            })
            .collect::<Vec<_>>();
        vec_of_borders.shuffle(&mut rand::thread_rng());

        FloorBuilder {
            extra: HasBorders {
                borders: vec_of_borders,
            },
            height: self.height,
            width: self.width,
            map: self.map,
            noise_map: self.noise_map,
        }
    }

    fn check_for_secret_passages(self) -> FloorBuilder<Drawable> {
        let self_with_borders = self.get_cave_borders();

        // if there is more than 1 cave (border), find secret passages
        if self_with_borders.extra.borders.len() > 1 {
            println!("building connections");
            self_with_borders
                .build_connections(BuildConnectionIterations::FullyConnect)
                .trace_connection_paths(false, false)
        } else {
            let new_self = self_with_borders;
            FloorBuilder {
                height: new_self.height,
                width: new_self.width,
                map: new_self.map,
                noise_map: new_self.noise_map,
                extra: Drawable { to_draw: vec![] },
            }
        }
    }
}

impl<S: Smoothable> FloorBuilder<S> {
    fn smoothen(
        mut self,
        repeat: usize,
        create_new_walls: fn(usize) -> bool,
    ) -> FloorBuilder<Smoothed> {
        for r in 0..repeat {
            for column in 0..self.width {
                for row in 0..self.height {
                    let point = Point {
                        column: Column::new(column),
                        row: Row::new(row),
                    };
                    *self.map.at_mut(point, self.width) =
                        self.place_wall_logic(point, create_new_walls(r));
                }
            }
        }
        FloorBuilder {
            width: self.width,
            height: self.height,
            map: self.map,
            noise_map: self.noise_map,
            extra: Smoothed {},
        }
    }
}

impl<S: FloorBuilderState> FloorBuilder<S> {
    pub fn blank(width: i32, height: i32) -> FloorBuilder<Blank> {
        FloorBuilder {
            width,
            height,
            map: vec![Default::default(); (width * height).try_into().unwrap()],
            noise_map: vec![Default::default(); (width * height).try_into().unwrap()],
            extra: Blank {},
        }
    }
    /// will only return wall or empty
    fn place_wall_logic(&self, point: Point, create_new_walls: bool) -> DungeonTile {
        let num_walls_1_away = self.get_adjacent_walls(point, 1, 1);

        if self.map.at(point, self.width).is_wall() {
            if num_walls_1_away >= 4 {
                return DungeonTile::Wall;
            }
            if create_new_walls && self.get_adjacent_walls(point, 2, 2) < 2 {
                return DungeonTile::Wall;
            }
            if num_walls_1_away < 2 {
                return DungeonTile::Empty;
            }
        } else if num_walls_1_away >= 5 {
            return DungeonTile::Wall;
        }

        DungeonTile::Empty
    }

    pub fn get_adjacent_walls(&self, point: Point, distance_x: i32, distance_y: i32) -> usize {
        let start_x = point.row.get().saturating_sub(distance_x);
        let start_y = point.column.get().saturating_sub(distance_y);
        let end_x = point.row.get().saturating_add(distance_x);
        let end_y = point.column.get().saturating_add(distance_y);

        let mut counter = 0;

        for i_y in start_y..=end_y {
            for i_x in start_x..=end_x {
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

    fn is_wall(&self, point: Point) -> bool {
        // Consider out-of-bounds a wall
        if self.is_out_of_bounds(point) {
            return true;
        }

        if self.map.at(point, self.width).is_wall() {
            return true;
        }
        false
    }

    fn is_out_of_bounds(&self, point: Point) -> bool {
        (point.column.get() < 0 || point.row.get() < 0)
            || (point.column.get() > self.width.saturating_sub(1)
                || point.row.get() > self.height.saturating_sub(1))
    }

    fn is_out_of_bounds_usize(&self, point: Point) -> bool {
        self.is_out_of_bounds(point)
    }

    // TODO: remove use of i64
    fn get_legal_neighbors(&self, point: Point) -> impl Iterator<Item = Point> + '_ {
        #[rustfmt::skip]
        let v = vec![
            point + Point { row: Row::new( 1), column: Column::new( 0) },
            point + Point { row: Row::new( 0), column: Column::new( 1) },
            point + Point { row: Row::new(-1), column: Column::new( 0) },
            point + Point { row: Row::new( 0), column: Column::new(-1) },
        ];

        v.into_iter()
            .filter(move |&point| !self.is_out_of_bounds(point))
    }

    fn get_legal_neighbors_down_and_right(&self, point: Point) -> impl Iterator<Item = Point> + '_ {
        let down = point
            + Point {
                row: Row::new(1),
                column: Column::new(0),
            };
        let right = point
            + Point {
                row: Row::new(0),
                column: Column::new(1),
            };
        vec![down, right]
            .into_iter()
            .filter(move |&point| !self.is_out_of_bounds(point))
    }

    pub(crate) fn _pretty(&self, extra_points: Vec<Point>, extra_points2: Vec<Point>) -> String {
        self.map
            // .par_iter()
            .chunks(self.width as usize)
            .enumerate()
            .map(|i| {
                ANSIStrings(
                    &i.1.iter() /* par_ */
                        .enumerate()
                        .map(|j| {
                            j.1._print(
                                extra_points2.contains(&Point {
                                    row: Row::new(i.0.try_into().unwrap()),
                                    column: Column::new(j.0.try_into().unwrap()),
                                }),
                                extra_points.contains(&Point {
                                    row: Row::new(i.0.try_into().unwrap()),
                                    column: Column::new(j.0.try_into().unwrap()),
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

trait PointIndex<T> {
    type Output;
    fn at(&self, point: Point, width: i32) -> &Self::Output;
    fn at_mut(&mut self, point: Point, width: i32) -> &mut Self::Output;
}

impl<T> PointIndex<T> for Vec<T> {
    type Output = T;

    fn at(&self, point: Point, width: i32) -> &Self::Output {
        &self[((point.row.get() * width) + point.column.get()) as usize]
    }

    fn at_mut(&mut self, point: Point, width: i32) -> &mut Self::Output {
        &mut self[((point.row.get() * width) + point.column.get()) as usize]
    }
}

#[cfg(test)]
mod test_point_index {
    use super::*;

    #[test]
    fn test_point_index() {
        const WIDTH: usize = 10;
        const HEIGHT: usize = 20;
        #[rustfmt::skip]
        const MAP: [i32; WIDTH * HEIGHT] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let mut map = Vec::from(MAP);

        assert_eq!(
            map.at(
                Point {
                    row: Row::new(3),
                    column: Column::new(4)
                },
                WIDTH as i32
            ),
            &1
        );

        *map.at_mut(
            Point {
                row: Row::new(7),
                column: Column::new(2),
            },
            WIDTH as i32,
        ) = 1;

        assert_eq!(
            map.at(
                Point {
                    row: Row::new(7),
                    column: Column::new(2)
                },
                WIDTH as i32
            ),
            &1
        );
    }
}
