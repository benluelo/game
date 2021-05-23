use crate::dungeon::{
    distance, dungeon_tile::DungeonTile, floor_builder::floor_builder_state::*, BorderId,
    Connection, Floor, Point,
};
use ansi_term::ANSIStrings;
use noise::{Billow, MultiFractal, NoiseFn, Seedable};
use pathfinding::directed::astar::astar;
use petgraph::{algo::kosaraju_scc, graphmap::UnGraphMap};
use rand::{prelude::StdRng, thread_rng, Rng, SeedableRng};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    iter,
};

pub mod floor_builder_state;

pub const MIN_FLOOR_SIZE: usize = 10;

/// Represents a floor of a dungeon.
pub struct FloorBuilder<S: FloorBuilderState> {
    pub(crate) height: usize,
    pub(crate) width: usize,
    pub(crate) map: Vec<DungeonTile>,
    pub(crate) noise_map: Vec<u128>,
    extra: S,
}

impl FloorBuilder<New> {
    pub(crate) fn create(height: usize, width: usize) -> Floor {
        assert!(
            height >= MIN_FLOOR_SIZE,
            "floor height too small: {}; minimum is {}",
            height,
            MIN_FLOOR_SIZE
        );
        assert!(
            width >= MIN_FLOOR_SIZE,
            "floor width too small: {}; minimum is {}",
            width,
            MIN_FLOOR_SIZE
        );
        FloorBuilder::<Blank>::blank(height, width)
            .random_fill()
            .smoothen(7, |r| r < 4)
            .get_cave_borders()
            .build_connections()
            .trace_connection_paths(2)
            .draw(|_, _, _| DungeonTile::Empty)
            .smoothen(7, |_| false)
            .check_for_secret_passages()
            .draw(|index, size, _| {
                if index == 0 || index == size {
                    DungeonTile::SecretDoor { requires_key: true }
                } else {
                    DungeonTile::SecretPassage
                }
            })
            .finish()
    }
}

impl FloorBuilder<Filled> {
    fn finish(self) -> Floor {
        Floor {
            height: self.height,
            width: self.width,
            data: self.map,
        }
    }
}

/// See http://roguebasin.roguelikedevelopment.org/index.php?title=Cellular_Automata_Method_for_Generating_Random_Cave-Like_Levels
impl<S: FloorBuilderState> FloorBuilder<S> {
    pub fn blank(height: usize, width: usize) -> FloorBuilder<Blank> {
        FloorBuilder {
            height,
            width,
            map: vec![Default::default(); height * width],
            noise_map: vec![Default::default(); height * width],
            extra: Blank {},
        }
    }
}

impl FloorBuilder<Blank> {
    pub fn random_fill(mut self) -> FloorBuilder<Filled> {
        let mut rng = thread_rng();

        let noise = Billow::new().set_seed(rng.gen()).set_persistence(128.0);

        // build initial maps (walls and noise)
        for x in 0..self.width {
            for y in 0..self.height {
                let point = Point { x, y };
                let float = noise.get([
                    (y as f64 / self.width as f64) + 0.1,
                    (x as f64 / self.height as f64) + 0.1,
                ]) + 0.001;
                *self.noise_map.at_mut(point, self.height, self.width) =
                    (float.abs() * 10000.0).powi(2).floor() as u128;

                // create a border around the map
                if (y == 0) || (x == 0) || (y == self.width) || (x == self.height - 1) {
                    *self.map.at_mut(point, self.height, self.width) = DungeonTile::Wall;
                }
                // otherwise, make a wall some percent of the time
                else {
                    *self.map.at_mut(point, self.height, self.width) =
                        if 52 >= rng.gen_range(0..101) {
                            DungeonTile::Wall
                        } else {
                            DungeonTile::Empty
                        }
                }
            }
        }

        // ANCHOR: astar
        // find path through noise map and apply path to walls map
        let goal = Point {
            x: self.width - 4,
            y: self.height - 4,
        };
        let astar_result = astar(
            &Point { x: 4, y: 4 },
            |&point| {
                self.get_legal_neighbors(point)
                    .map(|p| (p, *self.noise_map.at(p, self.height, self.width)))
            },
            |_| 0,
            |&point| !self.is_out_of_bounds_usize(point.x, point.y) && point == goal,
        )
        .expect("no path found");

        for &point in &astar_result.0 {
            *self.map.at_mut(point, self.height, self.width) = DungeonTile::Empty;
            for neighbor in self
                .get_legal_neighbors_down_and_right(point)
                .collect::<Vec<_>>()
            {
                *self.map.at_mut(neighbor, self.height, self.width) = DungeonTile::Empty;
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
        let size = self.extra.to_draw.len() + 1;
        for (index, point) in self.extra.to_draw.into_iter().enumerate() {
            *self.map.at_mut(point, self.height, self.width) = draw_with(index, size, point)
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

impl FloorBuilder<HasConnections> {
    fn trace_connection_paths(self, _width: usize) -> FloorBuilder<Drawable> {
        let to_draw = self
            .extra
            .connections
            .par_iter()
            .map(|(&(_, from), &(_, to))| {
                let mut rng = StdRng::from_entropy();

                let astar_result = astar(
                    &from,
                    |&point| {
                        self.get_legal_neighbors(point)
                            .map(|p| (p, *self.noise_map.at(p, self.height, self.width)))
                    },
                    |_| 1,
                    |&point| !self.is_out_of_bounds_usize(point.x, point.y) && (point == to),
                )
                .expect("no path found");

                let mut all_points = vec![];

                for point in astar_result.0 {
                    if self.map.at(point, self.height, self.width).is_empty() {
                        break;
                    }
                    all_points.push(point);
                }

                let mut extra_points = HashSet::new();

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

                all_points.into_iter().chain(extra_points)
            })
            .flatten_iter()
            .collect();

        FloorBuilder {
            extra: Drawable { to_draw },
            height: self.height,
            width: self.width,
            map: self.map,
            noise_map: self.noise_map,
        }
    }
}

impl<'a> FloorBuilder<HasBorders> {
    // build bridges between the disjointed caves and the closest cave border point *not* in the border of said disjointed cave
    fn build_connections(self) -> FloorBuilder<HasConnections> {
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

        let borders_with_ids = self
            .extra
            .borders
            .iter()
            .enumerate()
            .map(|(id, hashset)| (BorderId(id), hashset.clone()));

        let all_border_points = borders_with_ids
            .clone()
            .map(|(id, border)| border.into_iter().zip(iter::repeat(id)))
            .flatten()
            .collect::<HashSet<(Point, BorderId)>>();

        // let mut connections = HashMap::<(BorderId, Point), HashSet<(BorderId, Point)>>::new();
        let mut connections_with_points = HashMap::<(BorderId, Point), (BorderId, Point)>::new();
        let mut connections = UnGraphMap::<BorderId, ()>::new();

        loop {
            // loop through all the borders
            let connections_with_points_inner: HashMap<(BorderId, Point), (BorderId, Point)> =
                borders_with_ids
                    .clone()
                    .filter_map(|(current_id, border)| {
                        let already_connected_ids =
                            connections.neighbors(current_id).collect::<HashSet<_>>();
                        let connection: Option<Connection> = all_border_points
                            .par_iter()
                            // remove the points from the collection of all the border points the ones that are in the current border
                            .filter(|(_, id)| *id != current_id)
                            // remove all the points from the borders the current border is already connected to
                            .filter(|(_, id)| !already_connected_ids.contains(id))
                            .map(|&(point, id)| {
                                // find the point that's closest to the current border
                                border
                                    .clone()
                                    .par_iter()
                                    .map(move |border_point| {
                                        Some(Connection {
                                            distance: distance(point, *border_point),
                                            from: (current_id, *border_point),
                                            to: (id, point),
                                        })
                                    })
                                    .collect::<Vec<_>>()
                            })
                            .flatten_iter()
                            .reduce(
                                || None,
                                |prev, curr| match (prev, curr) {
                                    (Some(prev), Some(curr)) => {
                                        if prev.distance < curr.distance {
                                            Some(prev)
                                        } else {
                                            Some(curr)
                                        }
                                    }
                                    (Some(prev), None) => Some(prev),
                                    (None, Some(curr)) => Some(curr),
                                    (None, None) => None,
                                },
                            );

                        match connection {
                            None => {
                                if connections.neighbors(current_id).count() == 0 {
                                    panic!(
                                        "\n\ndisjointed cave\n{}",
                                        self.pretty(
                                            all_border_points
                                                .iter()
                                                .filter(|(_, id)| *id == current_id)
                                                .map(|(point, _)| *point)
                                                .collect(),
                                            connections_with_points
                                                .iter()
                                                .map(|i| vec![i.0 .1, i.1 .1])
                                                .flatten()
                                                .collect()
                                        )
                                    );
                                };
                                None
                            }
                            Some(conn) => {
                                // connections_with_points.insert(conn.to, conn.from);
                                // connections.add_edge(conn.to.0, conn.from.0, ());
                                Some((conn.to, conn.from))
                            }
                        }
                    })
                    .collect();

            connections_with_points.extend(connections_with_points_inner);
            connections.extend(connections_with_points.iter().map(|(k, v)| (k.0, v.0)));
            // println!("connections = {:#?}", connections_with_points);
            // File::create("/home/ben/codeprojects/game/dot.dot")
            //     .unwrap()
            //     .write_all(
            //         format!(
            //             "{:?}",
            //             Dot::with_config(&connections.clone(), &[Config::EdgeNoLabel])
            //         )
            //         .as_bytes(),
            //     )
            //     .unwrap();
            let sccs = kosaraju_scc(&connections);
            if !sccs.len() > 1 {
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

impl<'a> FloorBuilder<Smoothed> {
    fn get_cave_borders(self) -> FloorBuilder<HasBorders> {
        let mut already_visited = vec![false; self.height * self.width];
        // self.closest_empty_point_to_center();

        let mut borders = vec![];

        // loop through the entire map
        for x in 0..self.width {
            'y: for y in 0..self.height {
                let point = Point { x, y };
                // if the point has already been visited (by either the main loop or the cave searching) then continue looping through the map
                if *already_visited.at(point, self.height, self.width) {
                    continue 'y;
                }
                // otherwise, mark the point as visited
                *already_visited.at_mut(point, self.height, self.width) = true;

                // if there's an empty space at the point, BFS to find the border of the cave (no diagonals)
                if self.map.at(point, self.height, self.width).is_empty() {
                    let mut border = HashSet::new();

                    let mut queue = self
                        .get_legal_neighbors(Point { x, y })
                        .collect::<VecDeque<_>>();

                    loop {
                        if let Some(point) = queue.pop_front() {
                            // if point is empty, mark it as visited and then add all of it's
                            // legal neighbors to the queue
                            if self.map.at(point, self.height, self.width).is_empty() {
                                if *already_visited.at(point, self.height, self.width) {
                                    continue;
                                }
                                *already_visited.at_mut(point, self.height, self.width) = true;
                                self.get_legal_neighbors(point)
                                    .for_each(|p| queue.push_back(p));
                            } else {
                                // println!("found a wall");
                                border.insert(point);
                                // dbg!(&border);
                            }
                        } else {
                            if !border.is_empty() {
                                // add the found cave to the collection of all caves
                                // do some other fancy stuff maybe
                                borders.push(border);
                            }
                            continue 'y;
                        }
                    }
                }
            }
        }
        FloorBuilder {
            extra: HasBorders { borders },
            height: self.height,
            width: self.width,
            map: self.map,
            noise_map: self.noise_map,
        }
    }
}

impl<'a> FloorBuilder<Smoothed> {
    fn check_for_secret_passages(self) -> FloorBuilder<Drawable> {
        let self_with_borders = self.get_cave_borders();

        if self_with_borders.extra.borders.len() == 1 {
            self_with_borders
                .build_connections()
                .trace_connection_paths(1)
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
            for x in 0..self.width {
                for y in 0..self.height {
                    let point = Point { x, y };
                    *self.map.at_mut(point, self.height, self.width) =
                        self.place_wall_logic(point, create_new_walls(r));
                }
            }
        }
        FloorBuilder {
            extra: Smoothed {},
            height: self.height,
            width: self.width,
            map: self.map,
            noise_map: self.noise_map,
        }
    }
}

impl<S: FloorBuilderState> FloorBuilder<S> {
    /// will only return wall or empty
    fn place_wall_logic(&self, point: Point, create_new_walls: bool) -> DungeonTile {
        let num_walls_1_away = self.get_adjacent_walls(point, 1, 1);

        if self.map.at(point, self.height, self.width).is_wall() {
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

    pub fn get_adjacent_walls(&self, point: Point, distance_x: usize, distance_y: usize) -> usize {
        let start_x = point.x.saturating_sub(distance_x);
        let start_y = point.y.saturating_sub(distance_y);
        let end_x = point.x.saturating_add(distance_x);
        let end_y = point.y.saturating_add(distance_y);

        let mut counter = 0;

        for i_y in start_y..=end_y {
            for i_x in start_x..=end_x {
                if !(i_x == point.x && i_y == point.y) && self.is_wall(i_x as i64, i_y as i64) {
                    counter += 1;
                }
            }
        }
        counter
        // (start_y..=end_y)
        //     .map(|i_y| {
        //         (start_x..=end_x).map(move |i_x| {
        //             if !(i_x == point.x as i64 && i_y == point.y as i64) && self.is_wall(i_x, i_y) {
        //                 Some(())
        //             } else {
        //                 None
        //             }
        //         })
        //     })
        //     .flatten()
        //     .filter(Option::is_some)
        //     .count()
    }

    fn is_wall(&self, x: i64, y: i64) -> bool {
        // Consider out-of-bounds a wall
        if self.is_out_of_bounds(x, y) {
            return true;
        }

        if self
            .map
            .at(
                Point {
                    x: x as usize,
                    y: y as usize,
                },
                self.height,
                self.width,
            )
            .is_wall()
        {
            return true;
        }

        if self
            .map
            .at(
                Point {
                    x: x as usize,
                    y: y as usize,
                },
                self.height,
                self.width,
            )
            .is_wall()
        {
            return false;
        }
        false
    }

    fn is_out_of_bounds_or_border(&self, x: i64, y: i64) -> bool {
        (x < 1 || y < 1) || (x >= self.width as i64 - 1 || y >= self.height as i64 - 1)
    }

    fn is_out_of_bounds(&self, x: i64, y: i64) -> bool {
        (x < 0 || y < 0) || (x > self.width as i64 - 1 || y > self.height as i64 - 1)
    }

    fn is_out_of_bounds_usize(&self, x: usize, y: usize) -> bool {
        self.is_out_of_bounds(x as i64, y as i64)
    }

    fn get_legal_neighbors(&self, point: Point) -> impl Iterator<Item = Point> + '_ {
        let x64 = point.x as i64;
        let y64 = point.y as i64;

        vec![
            (x64 + 1, y64),
            (x64, y64 + 1),
            (x64 - 1, y64),
            (x64, y64 - 1),
        ]
        .into_iter()
        .filter(move |&(x, y)| !self.is_out_of_bounds_or_border(x, y))
        .map(|(x, y)| Point {
            x: x as usize,
            y: y as usize,
        })
    }

    fn get_legal_neighbors_down_and_right(&self, point: Point) -> impl Iterator<Item = Point> + '_ {
        let x64 = point.x as i64;
        let y64 = point.y as i64;
        vec![(x64 + 1, y64), (x64, y64 - 1)]
            .into_iter()
            .filter(move |&(x, y)| !self.is_out_of_bounds_or_border(x, y))
            .map(move |(x, y)| Point {
                x: x as usize,
                y: y as usize,
            })
    }

    pub(crate) fn pretty(&self, extra_points: Vec<Point>, extra_points2: Vec<Point>) -> String {
        self.map
            .par_iter()
            .chunks(self.height)
            .enumerate()
            .map(|i| {
                ANSIStrings(
                    &i.1.par_iter()
                        .enumerate()
                        .map(|j| {
                            j.1.print(
                                extra_points2.contains(&Point { x: i.0, y: j.0 }),
                                extra_points.contains(&Point { x: i.0, y: j.0 }),
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
    fn at(&self, point: Point, height: usize, width: usize) -> &Self::Output;
    fn at_mut(&mut self, point: Point, height: usize, width: usize) -> &mut Self::Output;
}

// impl<'a, T> PointIndex<T> for &'a mut [&'a mut [T]] {
//     type Output = T;

//     fn at(&self, point: Point) -> &'a Self::Output {
//         &self[point.x][point.y]
//     }

//     fn at_mut(&mut self, point: Point) -> &'a mut Self::Output {
//         &mut self[point.x][point.y]
//     }
// }

// impl<T> PointIndex<T> for Vec<Vec<T>> {
//     type Output = T;

//     fn at(&self, point: Point) -> &Self::Output {
//         &self[point.x][point.y]
//     }

//     fn at_mut(&mut self, point: Point) -> &mut Self::Output {
//         &mut self[point.x][point.y]
//     }
// }

impl<T> PointIndex<T> for Vec<T> {
    type Output = T;

    fn at(&self, point: Point, height: usize, width: usize) -> &Self::Output {
        &self[point.x * height + point.y]
    }

    fn at_mut(&mut self, point: Point, height: usize, width: usize) -> &mut Self::Output {
        &mut self[point.x * height + point.y]
    }
}
