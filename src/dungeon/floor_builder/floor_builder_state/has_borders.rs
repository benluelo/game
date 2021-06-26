use petgraph::algo::min_spanning_tree;
use std::{collections::HashMap, iter};

use petgraph::{algo::kosaraju_scc, data::FromElements, graphmap::UnGraphMap};

use crate::dungeon::{distance, Border, BorderId, Connection, FloorBuilder, Point};

use super::{drawable::Drawable, has_connections::HasConnections, FloorBuilderState};

#[derive(Debug)]
pub(in crate::dungeon::floor_builder) struct HasBorders {
    pub(super) borders: Vec<Border>,
}
impl FloorBuilderState for HasBorders {}

#[derive(Debug, Clone, Copy)]
pub enum BuildConnectionIterations {
    /// Until there is only 1 scc left (the caves are fully connected).
    FullyConnect,
    /// A finite amount of times, or until the caves are fully connected.
    Finite(u8),
    /// Until there are at most the specified amount of sccs; guaranteed
    /// to be `<=` the specified amount.
    Until(u8),
}

impl FloorBuilder<HasBorders> {
    /// build bridges between the disjointed caves and the closest cave border point *not* in the border of said disjointed cave
    pub(in crate::dungeon::floor_builder) fn build_connections(
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
                frames: self.frames,
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

        // match iterations {
        //     BuildConnectionIterations::FullyConnect => todo!(),
        //     BuildConnectionIterations::Finite(_) => todo!(),
        //     BuildConnectionIterations::Until(_) => todo!(),
        // }

        // loop through all the borders
        // build one connection per loop
        for (acc, current_border) in self.extra.borders.iter().enumerate() {
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

            dbg!(&sccs);
            let should_return = match iterations {
                // if there is only one scc, we're done here
                BuildConnectionIterations::FullyConnect => sccs.len() == 1,
                // if we've iterated enough times, return
                BuildConnectionIterations::Finite(amount) => acc == amount as usize,
                // if the amount of sccs is less than or equal to the amount requested, return
                BuildConnectionIterations::Until(until) => sccs.len() <= until.into(),
            };
            if should_return {
                let msf = UnGraphMap::from_elements(min_spanning_tree(
                    &connected_borders_graph.into_graph::<usize>(),
                ));
                return FloorBuilder {
                    width: self.width,
                    height: self.height,
                    map: self.map,
                    noise_map: self.noise_map,
                    extra: HasConnections {
                        // remove extra connections from the connections_with_points hashmap (make it into the MSF)
                        connections: connections_with_points
                            .into_iter()
                            .filter(|&((_, k), (_, v))| msf.contains_edge(k, v))
                            .collect(),
                        borders: self.extra.borders.into_iter().map(|b| (b.id, b)).collect(),
                    },
                    frames: self.frames,
                };
            };
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
            frames: self.frames,
        }
    }
}
