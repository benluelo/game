//! All the valid states a floor builder may be in. each module contains it's
//! own state,  and the `impl`s found in each module represent the valid
//! transitions for each state.

use std::fmt::Debug;

/// Marker trait for possible states for a floor builder to be in, along with
/// their associated data.
pub trait FloorBuilderState: Debug {
    const TYPE_NAME: &'static str;
}

/// Marker trait for floor builder states that can be [`FloorBuilder::smoothen`]ed.
///
/// [`FloorBuilder::smoothen`]: method@crate::floor_builder::FloorBuilder::smoothen
pub trait Smoothable: FloorBuilderState {}

/// A blank [`FloorBuilder`], with everything set to their default.
///
/// [`FloorBuilder`]: crate::floor_builder::FloorBuilder
pub(in crate::floor_builder) mod blank;

/// A [`FloorBuilder`] that has state to write to it's internal map.
///
/// [`FloorBuilder`]: crate::floor_builder::FloorBuilder
pub(in crate::floor_builder) mod drawable;

/// A resting state for the [`FloorBuilder`]. It is technically 'completable' in
/// this state.
///
/// [`FloorBuilder`]: crate::floor_builder::FloorBuilder
pub(in crate::floor_builder) mod filled;

/// A [`FloorBuilder`] that has the borders around all of the caves in it's map.
///
/// [`FloorBuilder`]: crate::floor_builder::FloorBuilder
pub(in crate::floor_builder) mod has_borders;

/// [`FloorBuilder`] state containing connections between the caves.
///
/// [`FloorBuilder`]: crate::floor_builder::FloorBuilder
pub(in crate::floor_builder) mod has_connections;

/// State that marks the [`FloorBuilder`] as having the secret paths drawn
/// between the remaining caves that weren't connected.
///
/// [`FloorBuilder`]: crate::floor_builder::FloorBuilder
pub(in crate::floor_builder) mod has_secret_connections;

/// Original state of the [`FloorBuilder`]. Entry point to the state machine.
///
/// [`FloorBuilder`]: crate::floor_builder::FloorBuilder
pub(in crate) mod new;

pub(in crate::floor_builder) mod random_filled;

/// State that represents a [`FloorBuilder`] that has has the cellular automata
/// run on it, smoothing out the edges of the caves.
///
/// [`FloorBuilder`]: crate::floor_builder::FloorBuilder
pub(in crate::floor_builder) mod smoothed;
