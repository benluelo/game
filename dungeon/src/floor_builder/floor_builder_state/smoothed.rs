use std::{
    collections::{HashSet, VecDeque},
    convert::TryInto,
};

use rand::prelude::SliceRandom;

use crate::{
    border::{Border, BorderId},
    floor_builder::floor_builder_state::has_borders::{BuildConnectionIterations, HasBorders},
    point_index::PointIndex,
    Column, DungeonTile, FloorBuilder, Point, Row,
};

use super::{has_secret_connections::HasSecretPassages, FloorBuilderState};

/// A [`FloorBuilder`] that has run the cellular automata on it's map; all the
/// rough edges of the caves have been smoothed out.
#[derive(Debug)]
pub(in crate::floor_builder) struct Smoothed {}

impl FloorBuilderState for Smoothed {
    const TYPE_NAME: &'static str = "Smoothed";
}

impl FloorBuilder<Smoothed> {
    /// Finds the borders around all of the caves in the [`FloorBuilder`],
    /// leaving them in the [`HasBorders`] state.
    pub(in crate::floor_builder) fn get_cave_borders(self) -> FloorBuilder<HasBorders> {
        let mut already_visited = vec![
            false;
            (self.width.as_unbounded() * self.height.as_unbounded())
                .try_into()
                .unwrap()
        ];

        let mut borders = vec![];

        // loop through the entire map
        for column in self.width.expand_lower().range_from(0.try_into().unwrap()) {
            'rows: for row in self.height.expand_lower().range_from(0.try_into().unwrap()) {
                let point = Point {
                    column: Column::new(column),
                    row: Row::new(row),
                };
                // if the point has already been visited (by either the main loop or the cave
                // searching) then continue looping through the map
                if *already_visited.at(point, self.width) {
                    continue 'rows;
                }
                // otherwise, mark the point as visited
                *already_visited.at_mut(point, self.width) = true;

                // if there's an empty space at the point, BFS to find the border of the cave
                // (no diagonals)
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
                id: BorderId::new(id),
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
            frames: self.frames,
            id: self.id,
        }
    }

    /// Adds secret passages to the map between all of the remaining disjointed
    /// caves.
    pub(in crate::floor_builder) fn check_for_secret_passages(
        self,
    ) -> FloorBuilder<HasSecretPassages> {
        println!("starting secret passages search");
        let mut self_with_borders = self.get_cave_borders();

        dbg!(self_with_borders.extra.borders.len());
        // if there is more than 1 cave (border), find secret passages
        if self_with_borders.extra.borders.len() > 1 {
            self_with_borders = self_with_borders
                .build_connections(BuildConnectionIterations::FullyConnect)
                .inspect()
                .trace_connection_paths(false, false)
                .inspect()
                .draw(|is_first, is_last, _| {
                    if is_first || is_last {
                        DungeonTile::SecretDoor {
                            requires_key: true,
                            is_open: false,
                        }
                    } else {
                        DungeonTile::SecretPassage
                    }
                })
                .smoothen(0, |_| false)
                .inspect()
                .get_cave_borders()
                .inspect()
        }

        let new_self = self_with_borders;
        FloorBuilder {
            height: new_self.height,
            width: new_self.width,
            map: new_self.map,
            noise_map: new_self.noise_map,
            extra: HasSecretPassages {},
            frames: new_self.frames,
            id: new_self.id,
        }
    }
}
