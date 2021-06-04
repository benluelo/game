use crate::dungeon::{
    distance, dungeon_tile::DungeonTile, floor_builder::floor_builder_state::*, Border, BorderId,
    Column, Connection, Floor, Point, Row,
};
use ansi_term::ANSIStrings;
use noise::{Billow, MultiFractal, NoiseFn, Seedable};
use pathfinding::prelude::dijkstra;
use petgraph::{
    algo::{kosaraju_scc, min_spanning_tree},
    data::FromElements,
    graphmap::UnGraphMap,
};
use rand::{prelude::StdRng, thread_rng, Rng, SeedableRng};
// use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
// use num_traits::{SaturatingAdd, SaturatingSub};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    convert::TryInto,
    fmt::Debug,
    iter,
};

pub mod floor_builder_state;

pub const MIN_FLOOR_SIZE: usize = 10;

/// Represents a floor of a dungeon.
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
                .build_connections(false)
                .trace_connection_paths(true, true)
                .draw(|_, _, _| DungeonTile::Empty)
                .smoothen(7, |_| false)
                .check_for_secret_passages()
                .draw(|index, size, _| {
                    if index == 0 || index == size - 1 {
                        DungeonTile::SecretDoor { requires_key: true }
                    } else {
                        DungeonTile::SecretPassage
                    }
                });

        // loop {
        //     let fb = fb.smoothen(0, |_| false).get_cave_borders();
        //     if fb.extra.borders {}
        // }
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
    fn draw(mut self, draw_with: fn(usize, usize, Point) -> DungeonTile) -> FloorBuilder<Filled> {
        for iter in self.extra.to_draw.into_iter() {
            let size = iter.len();
            for (index, point) in iter.into_iter().enumerate() {
                *self.map.at_mut(point, self.width) = draw_with(index, size, point);
            }
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
            .map(|(&(_, from), &(_, to))| {
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

                let mut all_points = vec![];

                // match path.len() {
                //     // 1 or two long, all of the passages will become doors
                //     1 | 2 => {}
                //     // 3 or more long, make sure
                //     _ => {}
                // }
                for point in path {
                    // if an empty point is found, path is finished
                    if self.map.at(point, self.width).is_empty() {
                        break;
                    }
                    all_points.push(point);
                }

                let mut extra_points = HashSet::new();
                if wide {
                    for &point in &all_points {
                        for neighbor in match rng.gen_bool(0.5) {
                            true => self
                                .get_legal_neighbors_down_and_right(point)
                                .collect::<Vec<_>>(),
                            false => self.get_legal_neighbors(point).collect::<Vec<_>>(),
                        } {
                            extra_points.insert(neighbor);
                        }
                    }
                    iter::once(from)
                        .chain(
                            all_points
                                .into_iter()
                                .filter(move |v| *v != from && *v != to)
                                .chain(extra_points),
                        )
                        .chain(iter::once(to))
                        .collect::<Vec<_>>()
                } else {
                    iter::once(from)
                        .chain(
                            all_points
                                .into_iter()
                                .filter(move |v| *v != from && *v != to),
                        )
                        .chain(iter::once(to))
                        .collect::<Vec<_>>()
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

impl FloorBuilder<HasBorders> {
    /// build bridges between the disjointed caves and the closest cave border point *not* in the border of said disjointed cave
    fn build_connections(self, fully_connect: bool) -> FloorBuilder<HasConnections> {
        if self.extra.borders.len() == 1 {
            return FloorBuilder {
                extra: HasConnections {
                    connections: Default::default(),
                },
                height: self.height,
                width: self.width,
                map: self.map,
                noise_map: self.noise_map,
            };
        }

        let all_border_points = self
            .extra
            .borders
            .iter()
            .cloned()
            .map(|Border { id, points }| points.into_iter().zip(iter::repeat(id)))
            .flatten()
            .collect::<HashSet<(Point, BorderId)>>();

        let mut connections_with_points = HashMap::<(BorderId, Point), (BorderId, Point)>::new();
        // a graph is required to check for the strongly connected components and the
        // minimum spanning tree (because i don't want to implement that myself lol)
        let mut connections = UnGraphMap::<BorderId, ()>::new();

        // loop through all the borders
        loop {
            println!("loop");
            // the `_inner` variable is required to add to both the graph and the hashmap and keep them in sync
            // LINK src/dungeon/floor_builder.rs#why-inner-is-required
            let connections_with_points_inner: HashMap<(BorderId, Point), (BorderId, Point)> = self
                .extra
                .borders
                .iter()
                .filter_map(|current_border| {
                    let already_connected_ids = connections
                        .neighbors(current_border.id)
                        .collect::<HashSet<_>>();
                    all_border_points
                        .iter()
                        // filter out border points that are either:
                        // - in the current border, or
                        // - in a border the current border is already connected to
                        .filter(|(_, id)| {
                            *id != current_border.id || !already_connected_ids.contains(id)
                        })
                        .map(|&(point, id)| {
                            // find the point that's closest to the current border
                            current_border
                                .points
                                .iter()
                                .map(move |border_point| Connection {
                                    distance: distance(point, *border_point),
                                    from: (current_border.id, *border_point),
                                    to: (id, point),
                                })
                            // .collect::<Vec<_>>()
                        })
                        .flatten()
                        .reduce(|prev, curr| {
                            if prev.distance < curr.distance {
                                prev
                            } else {
                                curr
                            }
                        })
                        .map(|conn| (conn.to, conn.from))
                })
                .collect();

            // ANCHOR[id=why-inner-is-required] extend both the graph and the map with points with the same `_inner` variable to keep them in sync
            connections_with_points.extend(connections_with_points_inner);
            connections.extend(connections_with_points.iter().map(|(k, v)| (k.0, v.0)));

            // strongly connected components
            let sccs = kosaraju_scc(&connections);
            if fully_connect {
                if dbg!(dbg!(sccs).len()) == 1 {
                    let msf = UnGraphMap::from_elements(min_spanning_tree(
                        &connections.into_graph::<usize>(),
                    ));
                    break FloorBuilder {
                        extra: HasConnections {
                            connections: connections_with_points
                                .into_iter()
                                .filter(|&((k, _), (v, _))| msf.contains_edge(k, v))
                                .collect(),
                        },
                        height: self.height,
                        width: self.width,
                        map: self.map,
                        noise_map: self.noise_map,
                    };
                } else {
                    // if there is more than 1 scc, loop and go again
                    continue;
                }
            } else {
                break FloorBuilder {
                    extra: HasConnections {
                        connections: connections_with_points,
                    },
                    height: self.height,
                    width: self.width,
                    map: self.map,
                    noise_map: self.noise_map,
                };
            };
        }
    }
}

impl FloorBuilder<Smoothed> {
    fn get_cave_borders(self) -> FloorBuilder<HasBorders> {
        let mut already_visited = vec![false; (self.width * self.height).try_into().unwrap()];
        // self.closest_empty_point_to_center();

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
        FloorBuilder {
            extra: HasBorders {
                borders: borders
                    .iter()
                    .enumerate()
                    .map(|(id, hashset)| Border {
                        id: BorderId(id),
                        points: hashset.clone(),
                    })
                    .collect(),
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
                .build_connections(true)
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

    pub(crate) fn pretty(&self, extra_points: Vec<Point>, extra_points2: Vec<Point>) -> String {
        self.map
            // .par_iter()
            .chunks(self.width as usize)
            .enumerate()
            .map(|i| {
                ANSIStrings(
                    &i.1.iter() /* par_ */
                        .enumerate()
                        .map(|j| {
                            j.1.print(
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
