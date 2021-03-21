#![allow(clippy::dead_code)]
// #![allow(dead_code)]

use noise::{Billow, MultiFractal, NoiseFn, Seedable};
use pathfinding::prelude::astar;
use std::{
    collections::{HashMap, VecDeque},
    num::NonZeroUsize,
    u128,
};

use rand::{thread_rng, Rng};

/// Represents a dungeon.
pub struct Dungeon {
    /// What type of dungeon this is.
    /// This tells what the background of the dungeon should look like.
    pub dungeon_type: DungeonType,
    pub floors: Vec<FloorBuilder>,
}

/// Represents a floor of a dungeon.
/// The outer `Vec` is the columns, and the inner
/// `Vec` is the rows. `0` is at the top-right.
pub struct FloorBuilder {
    pub(crate) height: NonZeroUsize,
    pub(crate) width: NonZeroUsize,
    pub(crate) map: Vec<Vec<WallOrEmpty>>,
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

impl Dungeon {
    pub fn new(_height: NonZeroUsize, _width: NonZeroUsize, _floors: NonZeroUsize) -> Self {
        todo!()
    }
}

/// See: http://roguebasin.roguelikedevelopment.org/index.php?title=Cellular_Automata_Method_for_Generating_Random_Cave-Like_Levels
impl FloorBuilder {
    pub fn new(height: NonZeroUsize, width: NonZeroUsize) -> FloorBuilder {
        let mut floor = FloorBuilder::blank(height, width);
        floor.random_fill();
        floor.smoothen();
        floor
    }

    fn blank(height: NonZeroUsize, width: NonZeroUsize) -> FloorBuilder {
        FloorBuilder {
            height,
            width,
            map: vec![vec![Empty; height.get()]; width.get()],
        }
    }

    fn random_fill(&mut self) {
        let mut rng = thread_rng();

        let mut noise_map = vec![vec![0_u128; self.height.get()]; self.width.get()];
        let noise = Billow::new().set_seed(rng.gen()).set_persistence(128.0);

        // build initial maps (walls and noise)
        for column in 0..self.height.get() {
            for row in 0..self.width.get() {
                let float = noise.get([
                    (row as f64 / self.width.get() as f64) + 0.1,
                    (column as f64 / self.height.get() as f64) + 0.1,
                ]) + 0.001;
                noise_map[row][column] = (float.abs() * 10000.0).powi(2).floor() as u128;

                // create a border around the map
                if row == 0 {
                    self.map[row][column] = Wall;
                } else if column == 0 {
                    self.map[row][column] = Wall;
                } else if row == self.width.get() - 1 {
                    self.map[row][column] = Wall;
                } else if column == self.height.get() - 1 {
                    self.map[row][column] = Wall;
                }
                // otherwise, make a wall some percent of the time
                else {
                    self.map[row][column] = if 52 >= rng.gen_range(0..101) {
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
        let goal = (self.width.get() - 4, self.height.get() - 4);
        let astar_result = astar(
            &(4, 4),
            |&(x, y)| {
                // println!("checking point {:?}", (x, y));
                self.get_legal_neighbors_with_noise_value(x, y, &noise_map)
            },
            |&(_x, _y)| 0, /* u128::MAX >> 1 */
            /* {
                if self.is_out_of_bounds(x, y) {
                    f64::INFINITY
                } else {
                    noise_map[x as usize][y as usize]
                }
            } */
            |&(x, y)| !self.is_out_of_bounds_usize(x, y) && (x as usize, y as usize) == goal,
        )
        .expect("no path found");

        for &(x, y) in &astar_result.0 {
            self.map[x][y] = Empty;
            for (nx, ny) in self.get_legal_neighbors_down_and_right(x, y) {
                self.map[nx][ny] = Empty;
            }
        }

        // dbg!(&astar_result);
    }

    // build bridges between the disjointed caves and the closest empty space
    // flood fill to find the caves, then draw a path between the closest two empty spaces for every cave
    // somehow make sure all the caves are connected
    // use a hashmap of (x, y) => (x, y) for every cave connection, then do something with that?
    // need to get the borders of the caves somehow
    // whenever a wall is encountered, push it to a vec?
    fn get_cave_borders(&self) -> Vec<Vec<(usize, usize)>> {
        let mut connections = HashMap::new();
        connections.insert((0usize, 0usize), (0usize, 0usize));

        // REVIEW: use a vec of hashsets maybe? to ensure no point is encountered twice on the border
        let mut caves = Vec::<Vec<(usize, usize)>>::new();

        let mut already_visited = vec![vec![false; self.height.get()]; self.width.get()];
        // self.closest_empty_point_to_center();

        // loop through the entire map
        for x in 0..self.height.get() {
            'y: for y in 0..self.width.get() {
                // if the point has already been visited (by either the main loop or the cave searching) then continue looping through the map
                if already_visited[x][y] {
                    continue 'y;
                }
                // otherwise, mark the point as visited
                already_visited[x][y] = true;

                // if there's an empty space at the point, BFS to find the border of the cave (no diagonals)
                if self.map[x][y] == Empty {
                    let mut border = Vec::new();

                    let mut queue = self.get_legal_neighbors(x, y).collect::<VecDeque<_>>();

                    loop {
                        if let Some((px, py)) = queue.pop_front() {
                            // if point is empty, mark it as visited and then add all of it's
                            // legal neighbors to the queue
                            if self.map[px][py] == Empty {
                                if already_visited[px][py] {
                                    continue;
                                }
                                already_visited[px][py] = true;
                                self.get_legal_neighbors(px, py)
                                    .for_each(|p| queue.push_back(p));
                            } else {
                                // println!("found a wall");
                                border.push((px, py));
                                // dbg!(&border);
                            }
                        } else {
                            // add the found cave to the collection of all caves
                            // do some other fancy stuff maybe
                            caves.push(border);
                            continue 'y;
                        }
                    }
                }
            }
        }

        caves
    }

    fn closest_empty_point_to_center(&self) {
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

    fn smoothen(&mut self) {
        for r in 0..7 {
            // dbg!(r);
            for row in 0..self.height.get() {
                for column in 0..self.width.get() {
                    self.map[column][row] = self.place_wall_logic(column, row, r < 4);
                }
            }
        }
    }

    fn place_wall_logic(&self, x: usize, y: usize, create_new_walls: bool) -> WallOrEmpty {
        let num_walls_1_away = self.get_adjacent_walls(x, y, 1, 1);
        let num_walls_2_away = if create_new_walls {
            self.get_adjacent_walls(x, y, 2, 2)
        } else {
            0
        };

        if self.map[x][y] == Wall {
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

    fn get_adjacent_walls(&self, x: usize, y: usize, distance_x: i64, distance_y: i64) -> u8 {
        let start_x = (x as i64) - distance_x;
        let start_y = (y as i64) - distance_y;
        let end_x = (x as i64) + distance_x;
        let end_y = (y as i64) + distance_y;

        let mut wall_counter = 0;

        for i_y in start_y..=end_y {
            for i_x in start_x..=end_x {
                if !(i_x == x as i64 && i_y == y as i64) {
                    if self.is_wall(i_x, i_y) {
                        wall_counter += 1;
                    }
                }
            }
        }
        return wall_counter;
    }

    fn is_wall(&self, x: i64, y: i64) -> bool {
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

    fn is_out_of_bounds_or_border(&self, x: i64, y: i64) -> bool {
        if x < 1 || y < 1 {
            return true;
        } else if x >= self.width.get() as i64 - 1 || y >= self.height.get() as i64 - 1 {
            return true;
        }
        return false;
    }

    fn is_out_of_bounds(&self, x: i64, y: i64) -> bool {
        if x < 0 || y < 0 {
            return true;
        } else if x > self.width.get() as i64 - 1 || y > self.height.get() as i64 - 1 {
            return true;
        }
        return false;
    }

    fn is_out_of_bounds_usize(&self, x: usize, y: usize) -> bool {
        self.is_out_of_bounds(x as i64, y as i64)
    }

    fn get_legal_neighbors_with_noise_value<'a>(
        &'a self,
        x: usize,
        y: usize,
        noise_map: &'a Vec<Vec<u128>>,
    ) -> impl Iterator<Item = ((usize, usize), u128)> + 'a {
        self.get_legal_neighbors(x, y)
            .map(move |(x, y)| ((x as usize, y as usize), noise_map[x as usize][y as usize]))
    }

    fn get_legal_neighbors(&self, x: usize, y: usize) -> impl Iterator<Item = (usize, usize)> + '_ {
        let x64 = x as i64;
        let y64 = y as i64;

        vec![
            (x64 + 1, y64),
            (x64, y64 + 1),
            (x64 - 1, y64),
            (x64, y64 - 1),
        ]
        .into_iter()
        .filter(move |&(x, y)| !self.is_out_of_bounds_or_border(x, y))
        .map(|(x, y)| (x as usize, y as usize))
    }

    fn get_legal_neighbors_down_and_right(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let x64 = x as i64;
        let y64 = y as i64;
        let possible_neighbors = [(x64 + 1, y64), (x64, y64 - 1)];

        possible_neighbors
            .iter()
            .copied()
            .filter(|&(x, y)| !self.is_out_of_bounds_or_border(x, y))
            .map(|(x, y)| (x as usize, y as usize))
            .collect()
    }

    fn print(&self, extra_points: Vec<(usize, usize)>) -> String {
        self.map
            .iter()
            .enumerate()
            .map(|i| {
                i.1.iter()
                    .enumerate()
                    .map(|j| match j.1 {
                        Empty => "  ",
                        Wall => {
                            if extra_points.contains(&(i.0, j.0)) {
                                "▒▒"
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

#[cfg(test)]
mod test_dungeon {
    use super::*;

    #[test]
    fn test_blank_floor_generation() {
        let blank_floor = FloorBuilder::blank(
            NonZeroUsize::new(10).unwrap(),
            NonZeroUsize::new(10).unwrap(),
        );

        assert!(blank_floor.height.get() == 10);
        assert!(blank_floor.width.get() == 10);
    }

    #[test]
    fn test_random_fill_generation() {
        let random_filled_floor = FloorBuilder::new(
            NonZeroUsize::new(50).unwrap(),
            NonZeroUsize::new(100).unwrap(),
        );
        let formatted = random_filled_floor.print(vec![]);

        // println!("{}", &formatted)
    }

    #[test]
    fn test_border_finding() {
        let floor_builder = FloorBuilder::new(
            NonZeroUsize::new(100).unwrap(),
            NonZeroUsize::new(100).unwrap(),
        );
        let caves = floor_builder.get_cave_borders();
        let all_border_points = caves.iter().cloned().flatten().collect::<Vec<_>>();

        println!("{}", floor_builder.print(all_border_points));
        let caves_pretty = caves
            .iter()
            .map(|v| {
                v.iter()
                    .map(|(x, y)| format!("({}, {})", x, y))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        println!("caves = {:#?}", caves_pretty);
    }
}
