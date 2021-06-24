use crate::dungeon::{floor_builder::dijkstra, point_index::PointIndex};
use itertools::Itertools;

use std::collections::{HashMap, HashSet};

use crate::dungeon::{
    Border, BorderId, ConnectionPath, ConnectionPathLength, DungeonTile, FloorBuilder, Point,
};

use super::{drawable::Drawable, FloorBuilderState};

#[derive(Debug, Default)]
pub(in crate::dungeon::floor_builder) struct HasConnections {
    pub(in crate::dungeon::floor_builder) connections:
        HashMap<(Point, BorderId), (Point, BorderId)>,
    pub(in crate::dungeon::floor_builder) borders: HashMap<BorderId, Border>,
}
impl FloorBuilderState for HasConnections {}

impl FloorBuilder<HasConnections> {
    /// takes the connections from [`FloorBuilder::build_connections`] and traces paths
    /// between them, leaving the paths in the `to_draw` state of `FloorBuilder<Drawable>`.
    pub(in crate::dungeon::floor_builder) fn trace_connection_paths(
        self,
        wide: bool,
        use_noise_map: bool,
    ) -> FloorBuilder<Drawable> {
        let all_border_points = self
            .extra
            .borders
            .iter()
            .flat_map(|(_, Border { points, .. })| points.clone())
            .collect::<HashSet<_>>();

        let to_draw = self
            .extra
            .connections
            .iter()
            .filter_map(|(&(from, from_id), &(to, to_id))| {
                dijkstra(
                    &from,
                    |&point| {
                        self.get_legal_neighbors(point)
                            // keep if it *is* the final point or it *isn't* a border point
                            .filter(|p| *p == to || !all_border_points.contains(p))
                            .map(|p| {
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
                    |&point| {
                        (!self.is_out_of_bounds(point) && (point == to))
                            || matches!(
                                self.map.at(point, self.width),
                                &DungeonTile::SecretDoor { .. } | &DungeonTile::SecretPassage
                            )
                    },
                )
                .map(|(path, _)| {
                    ConnectionPath {
                        start_border_id: from_id,
                        end_border_id: to_id,
                        path: match path.as_slice() {
                            [point] => ConnectionPathLength::Length1 { point: *point },
                            [start, end] => ConnectionPathLength::Length2 {
                                start: *start,
                                end: *end,
                            },
                            _ => ConnectionPathLength::Length3Plus {
                                points: path
                                    .iter()
                                    .copied()
                                    .chain(if wide {
                                        path.iter()
                                            .flat_map(|&point| {
                                                // match rng.gen_bool(0.5) {
                                                // true => self
                                                // .get_legal_neighbors_down_and_right(point)
                                                // .collect::<Vec<_>>(),
                                                // false => {
                                                self.get_legal_neighbors(point)
                                                /* .collect::<Vec<_>>() */
                                                // }
                                                // }
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
