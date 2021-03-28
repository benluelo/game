#![allow(clippy::dead_code)]
// #![allow(dead_code)]

use core::fmt;
use noise::{Billow, MultiFractal, NoiseFn, Seedable};
use pathfinding::prelude::astar;
use petgraph::{
    algo::kosaraju_scc,
    dot::{Config, Dot},
    graphmap::UnGraphMap,
    Graph,
};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs::File,
    io::Write,
    iter,
    num::NonZeroUsize,
    u128,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct Point {
    x: usize,
    y: usize,
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

use rand::{thread_rng, Rng};

/// Represents a dungeon.
pub struct Dungeon {
    /// What type of dungeon this is.
    /// This tells what the background of the dungeon should look like.
    pub dungeon_type: DungeonType,
    pub floors: Vec<FloorBuilder>,
}

impl Dungeon {
    pub fn new(_height: NonZeroUsize, _width: NonZeroUsize, _floors: NonZeroUsize) -> Self {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WallOrEmpty {
    Empty = 0,
    Wall = 1,
}

use WallOrEmpty::*;

#[non_exhaustive]
pub enum DungeonType {
    Cave,
    Forest,
}

/// Represents a floor of a dungeon.
/// The outer `Vec` is the columns, and the inner
/// `Vec` is the rows. `0` is at the top-right.
pub struct FloorBuilder {
    pub(crate) height: NonZeroUsize,
    pub(crate) width: NonZeroUsize,
    pub(crate) map: Vec<Vec<WallOrEmpty>>,
    pub(crate) noise_map: Vec<Vec<u128>>,
}

/// See http://roguebasin.roguelikedevelopment.org/index.php?title=Cellular_Automata_Method_for_Generating_Random_Cave-Like_Levels
impl FloorBuilder {
    pub fn new(height: NonZeroUsize, width: NonZeroUsize) -> FloorBuilder {
        let mut floor = FloorBuilder::blank(height, width);
        floor.random_fill();
        floor.smoothen(7, |r| r < 4);
        floor
    }

    pub(crate) fn blank(height: NonZeroUsize, width: NonZeroUsize) -> FloorBuilder {
        FloorBuilder {
            height,
            width,
            map: vec![vec![Empty; height.get()]; width.get()],
            noise_map: vec![vec![0_u128; height.get()]; width.get()],
        }
    }

    pub(crate) fn random_fill(&mut self) {
        let mut rng = thread_rng();

        let noise = Billow::new().set_seed(rng.gen()).set_persistence(128.0);

        // build initial maps (walls and noise)
        for x in 0..self.width.get() {
            for y in 0..self.height.get() {
                let float = noise.get([
                    (y as f64 / self.width.get() as f64) + 0.1,
                    (x as f64 / self.height.get() as f64) + 0.1,
                ]) + 0.001;
                self.noise_map[x][y] = (float.abs() * 10000.0).powi(2).floor() as u128;

                // create a border around the map
                if y == 0 {
                    self.map[x][y] = Wall;
                } else if x == 0 {
                    self.map[x][y] = Wall;
                } else if y == self.width.get() - 1 {
                    self.map[x][y] = Wall;
                } else if x == self.height.get() - 1 {
                    self.map[x][y] = Wall;
                }
                // otherwise, make a wall some percent of the time
                else {
                    self.map[x][y] = if 52 >= rng.gen_range(0..101) {
                        Wall
                    } else {
                        Empty
                    }
                }
            }
        }

        // dbg!(&noise_map
        //     .iter()
        //     .flatten()
        //     .fold(&0_u128, |curr, prev| if curr > prev { curr } else { prev }));

        // ANCHOR: astar
        // find path through noise map and apply path to walls map
        let goal = Point {
            x: self.width.get() - 4,
            y: self.height.get() - 4,
        };
        let astar_result = astar(
            &Point { x: 4, y: 4 },
            |&point| {
                self.get_legal_neighbors(point)
                    .map(|p| (p, self.noise_map[p.x][p.y]))
            },
            |_| 0, /* u128::MAX >> 1 */
            /* {
                if self.is_out_of_bounds(x, y) {
                    f64::INFINITY
                } else {
                    noise_map[x as usize][y as usize]
                }
            } */
            |&point| !self.is_out_of_bounds_usize(point.x, point.y) && point == goal,
        )
        .expect("no path found");

        for point in &astar_result.0 {
            self.map[point.x][point.y] = Empty;
            for neighbor in self
                .get_legal_neighbors_down_and_right(*point)
                .collect::<Vec<_>>()
            {
                self.map[neighbor.x][neighbor.y] = Empty;
            }
        }

        // dbg!(&astar_result);
    }

    pub(crate) fn draw_connections(
        &mut self,
        connections: HashMap<(BorderId, Point), (BorderId, Point)>,
    ) {
        let mut rng = thread_rng();

        for ((_, from), (_, to)) in connections.into_iter() {
            let astar_result = astar(
                &from,
                |&point| {
                    self.get_legal_neighbors(point)
                        .map(|p| (p, self.noise_map[p.x][p.y]))
                },
                |_| 1,
                |&point| !self.is_out_of_bounds_usize(point.x, point.y) && (point == to),
            )
            .expect("no path found");

            let mut all_points = vec![];

            for point in astar_result.0 {
                if self.map[point.x][point.y] == Empty {
                    break;
                }
                all_points.push(point);
                self.map[point.x][point.y] = Empty;
            }

            for point in all_points {
                for neighbor in match rng.gen_bool(0.5) {
                    true => self
                        .get_legal_neighbors_down_and_right(point)
                        .collect::<Vec<_>>(),
                    false => self.get_legal_neighbors(point).collect::<Vec<_>>(),
                } {
                    self.map[neighbor.x][neighbor.y] = Empty;
                }
            }
        }
    }

    // build bridges between the disjointed caves and the closest empty space
    // flood fill to find the caves (with self.get_cave_borders), then draw a path (using a*) between the closest two empty spaces for every cave
    // somehow make sure all the caves are connected
    // use a hashmap of (x, y) => (x, y) for every cave connection, then do something with that?
    pub(crate) fn build_connections(
        &mut self,
        borders: Vec<HashSet<Point>>,
    ) -> HashMap<(BorderId, Point), (BorderId, Point)> {
        let borders_with_ids = borders
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
        // for (id, _) in borders_with_ids.clone() {
        //     connections.add_node(id);
        // }

        if borders.len() == 1 {
            return connections_with_points;
        }

        loop {
            // loop through all the borders
            for (current_id, border) in borders_with_ids.clone() {
                let already_connected_ids =
                    connections.neighbors(current_id).collect::<HashSet<_>>();
                let connection = all_border_points
                    .iter()
                    // remove the points from the collection of all the border points the ones that are in the current border
                    .filter(|(_, id)| *id != current_id)
                    // remove all the points from the borders the current border is already connected to
                    .filter(|(_, id)| !already_connected_ids.contains(id))
                    .map(|&(point, id)| {
                        // find the point that's closest to the current border
                        border
                            .clone()
                            .into_iter()
                            .map(move |border_point| Connection {
                                distance: distance(point, border_point),
                                from: (current_id, border_point),
                                to: (id, point),
                            })
                    })
                    .flatten()
                    .reduce(|prev, curr| {
                        if prev.distance < curr.distance {
                            prev
                        } else {
                            curr
                        }
                    });
                match connection {
                    Some(conn) => {
                        connections_with_points.insert(conn.to, conn.from);
                        connections.add_edge(conn.to.0, conn.from.0, ());
                    }
                    None => {
                        if connections.neighbors(current_id).count() == 0 {
                            panic!(
                                "disjointed cave\n{}",
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
                        }
                    }
                }
            }
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
            if sccs.len() > 1 {
                continue;
            } else {
                break connections_with_points;
            }
        }
    }

    pub(crate) fn get_cave_borders(&self) -> Vec<HashSet<Point>> {
        let mut caves = Vec::<HashSet<Point>>::new();

        let mut already_visited = vec![vec![false; self.height.get()]; self.width.get()];
        // self.closest_empty_point_to_center();

        // loop through the entire map
        for x in 0..self.width.get() {
            'y: for y in 0..self.height.get() {
                // if the point has already been visited (by either the main loop or the cave searching) then continue looping through the map
                if already_visited[x][y] {
                    continue 'y;
                }
                // otherwise, mark the point as visited
                already_visited[x][y] = true;

                // if there's an empty space at the point, BFS to find the border of the cave (no diagonals)
                if self.map[x][y] == Empty {
                    let mut border = HashSet::new();

                    let mut queue = self
                        .get_legal_neighbors(Point { x, y })
                        .collect::<VecDeque<_>>();

                    loop {
                        if let Some(point) = queue.pop_front() {
                            // if point is empty, mark it as visited and then add all of it's
                            // legal neighbors to the queue
                            if self.map[point.x][point.y] == Empty {
                                if already_visited[point.x][point.y] {
                                    continue;
                                }
                                already_visited[point.x][point.y] = true;
                                self.get_legal_neighbors(point)
                                    .for_each(|p| queue.push_back(p));
                            } else {
                                // println!("found a wall");
                                border.insert(point);
                                // dbg!(&border);
                            }
                        } else {
                            if border.len() > 0 {
                                // add the found cave to the collection of all caves
                                // do some other fancy stuff maybe
                                caves.push(border);
                            }
                            continue 'y;
                        }
                    }
                }
            }
        }

        caves
    }

    pub(crate) fn closest_empty_point_to_center(&self) {
        let map_center_x = self.height.get() / 2;
        let map_center_y = self.width.get() / 2;

        if self.map[map_center_x][map_center_y] == Wall {
            (map_center_x, map_center_y)
        } else {
            let mut distance: i64 = 1;
            'outer: loop {
                for i in -distance..=distance {
                    for j in -distance..=distance {
                        if i.abs() != distance && j.abs() != distance {
                            continue;
                        }

                        if !self.is_wall(map_center_x as i64 + i, map_center_y as i64 + j) {
                            break 'outer (
                                (map_center_x as i64 + i) as usize,
                                (map_center_y as i64 + j) as usize,
                            );
                        }
                    }
                }
                distance += 1;
            }
        };
    }

    pub(crate) fn smoothen(&mut self, repeat: usize, create_new_walls: fn(usize) -> bool) {
        for r in 0..repeat {
            for x in 0..self.width.get() {
                for y in 0..self.height.get() {
                    self.map[x][y] = self.place_wall_logic(Point { x, y }, create_new_walls(r));
                }
            }
        }
    }

    pub(crate) fn place_wall_logic(&self, point: Point, create_new_walls: bool) -> WallOrEmpty {
        let num_walls_1_away = self.get_adjacent_walls(point, 1, 1);
        let num_walls_2_away = if create_new_walls {
            self.get_adjacent_walls(point, 2, 2)
        } else {
            0
        };

        if self.map[point.x][point.y] == Wall {
            if num_walls_1_away >= 4 {
                return Wall;
            }
            if create_new_walls && num_walls_2_away < 2 {
                return Wall;
            }
            if num_walls_1_away < 2 {
                return Empty;
            }
        } else {
            if num_walls_1_away >= 5 {
                return Wall;
            }
        }
        return Empty;
    }

    pub(crate) fn get_adjacent_walls(&self, point: Point, distance_x: i64, distance_y: i64) -> u8 {
        let start_x = (point.x as i64) - distance_x;
        let start_y = (point.y as i64) - distance_y;
        let end_x = (point.x as i64) + distance_x;
        let end_y = (point.y as i64) + distance_y;

        let mut wall_counter = 0;

        for i_y in start_y..=end_y {
            for i_x in start_x..=end_x {
                if !(i_x == point.x as i64 && i_y == point.y as i64) {
                    if self.is_wall(i_x, i_y) {
                        wall_counter += 1;
                    }
                }
            }
        }
        return wall_counter;
    }

    pub(crate) fn is_wall(&self, x: i64, y: i64) -> bool {
        // Consider out-of-bound a wall
        if self.is_out_of_bounds(x, y) {
            return true;
        }

        if self.map[x as usize][y as usize] == Wall {
            return true;
        }

        if self.map[x as usize][y as usize] == Empty {
            return false;
        }
        return false;
    }

    pub(crate) fn is_out_of_bounds_or_border(&self, x: i64, y: i64) -> bool {
        if x < 1 || y < 1 {
            return true;
        } else if x >= self.width.get() as i64 - 1 || y >= self.height.get() as i64 - 1 {
            return true;
        }
        return false;
    }

    pub(crate) fn is_out_of_bounds(&self, x: i64, y: i64) -> bool {
        if x < 0 || y < 0 {
            return true;
        } else if x > self.width.get() as i64 - 1 || y > self.height.get() as i64 - 1 {
            return true;
        }
        return false;
    }

    pub(crate) fn is_out_of_bounds_usize(&self, x: usize, y: usize) -> bool {
        self.is_out_of_bounds(x as i64, y as i64)
    }

    pub(crate) fn get_legal_neighbors(&self, point: Point) -> impl Iterator<Item = Point> + '_ {
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

    pub(crate) fn get_legal_neighbors_down_and_right(
        &self,
        point: Point,
    ) -> impl Iterator<Item = Point> + '_ {
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
            .iter()
            .enumerate()
            .map(|i| {
                i.1.iter()
                    .enumerate()
                    .map(|j| match j.1 {
                        Empty => {
                            if extra_points2.contains(&Point { x: i.0, y: j.0 }) {
                                "[]"
                            } else if extra_points.contains(&Point { x: i.0, y: j.0 }) {
                                "EE"
                            } else {
                                "  "
                            }
                        }
                        Wall => {
                            if extra_points2.contains(&Point { x: i.0, y: j.0 }) {
                                "░░"
                            } else if extra_points.contains(&Point { x: i.0, y: j.0 }) {
                                "▓▓"
                            } else {
                                "██"
                            }
                        }
                    })
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn distance(from: Point, to: Point) -> f64 {
    (((from.x as i64 - to.x as i64).pow(2) + (from.y as i64 - to.y as i64).pow(2)) as f64).sqrt()
}

#[cfg(test)]
mod test_dungeon {
    use petgraph::algo::min_spanning_tree;

    use super::*;

    #[test]
    pub(crate) fn test_blank_floor_generation() {
        let blank_floor = FloorBuilder::blank(
            NonZeroUsize::new(10).unwrap(),
            NonZeroUsize::new(10).unwrap(),
        );

        assert!(blank_floor.height.get() == 10);
        assert!(blank_floor.width.get() == 10);
    }

    #[test]
    pub(crate) fn test_random_fill_generation() {
        let random_filled_floor = FloorBuilder::new(
            NonZeroUsize::new(50).unwrap(),
            NonZeroUsize::new(100).unwrap(),
        );
        let formatted = random_filled_floor.pretty(vec![], vec![]);

        // println!("{}", &formatted)
    }

    #[test]
    pub(crate) fn test_border_finding() {
        let floor_builder = FloorBuilder::new(
            NonZeroUsize::new(100).unwrap(),
            NonZeroUsize::new(100).unwrap(),
        );
        let caves = floor_builder.get_cave_borders();
        let all_border_points = caves.iter().cloned().flatten().collect::<Vec<_>>();

        println!("{}", floor_builder.pretty(all_border_points, vec![]));
        let caves_pretty = caves
            .iter()
            .map(|v| {
                v.iter()
                    .map(|point| format!("({}, {})", point.x, point.y))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        // println!("caves = {:#?}", caves_pretty);
    }

    #[test]
    pub(crate) fn test_cave_connections() {
        let mut floor_builder = FloorBuilder::new(
            NonZeroUsize::new(50).unwrap(),
            NonZeroUsize::new(100).unwrap(),
        );
        let caves = floor_builder.get_cave_borders();
        let connections = floor_builder.build_connections(caves);
        floor_builder.draw_connections(connections.clone());
        floor_builder.smoothen(7, |_| false);
        // let _ = min_spanning_tree(&connections)
        //     .inspect(|mst| {
        //         dbg!(mst);
        //     })
        //     .collect::<Vec<_>>();
        println!(
            "{}",
            floor_builder.pretty(
                connections
                    .into_iter()
                    .map(|i| vec![i.0 .1, i.1 .1])
                    .flatten()
                    .collect(),
                vec![]
            )
        );
    }
}
