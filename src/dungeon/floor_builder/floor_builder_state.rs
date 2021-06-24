///! All the valid states a floor builder may be in. each module contains it's own state,
///! and the `impl`s found in each module represent the valid transitions for each state.

/// Marker trait for possible states for a floor builder to be in, along with their associated data.
pub trait FloorBuilderState {}

/// Marker trait for floor builder states that can be [`FloorBuilder::smoothed`]
pub trait Smoothable: FloorBuilderState {}

pub(in crate::dungeon) mod new;
pub(in crate::dungeon::floor_builder) mod blank;
pub(in crate::dungeon::floor_builder) mod buildable;
pub(in crate::dungeon::floor_builder) mod done;
pub(in crate::dungeon::floor_builder) mod drawable;
pub(in crate::dungeon::floor_builder) mod filled;
pub(in crate::dungeon::floor_builder) mod has_borders;
pub(in crate::dungeon::floor_builder) mod has_connections;
pub(in crate::dungeon::floor_builder) mod smoothed;
