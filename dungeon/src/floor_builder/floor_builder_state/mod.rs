//! All the valid states a floor builder may be in. each module contains it's
//! own state,  and the `impl`s found in each module represent the valid
//! transitions for each state.

/// Marker trait for possible states for a floor builder to be in, along with
/// their associated data.
pub trait FloorBuilderState {}

/// Marker trait for floor builder states that can be [`FloorBuilder::smoothed`]
pub trait Smoothable: FloorBuilderState {}

/// A blank [`FloorBuilder`], with everything set to their default.
pub(in crate::floor_builder) mod blank;

/// A [`FloorBuilder`] that has state to write to it's internal map.
pub(in crate::floor_builder) mod drawable;

/// A resting state for the [`FloorBuilder`]. It is technically 'completable' in this state.
pub(in crate::floor_builder) mod filled;

/// A [`FloorBuilder`] that has the borders around all of the caves in it's map.
pub(in crate::floor_builder) mod has_borders;

/// [`FloorBuilder`] state containing connections between the caves.
pub(in crate::floor_builder) mod has_connections;

/// State that marks the [`FloorBuilder`] as having the secret paths drawn between the remaining caves that weren't connected.
pub(in crate::floor_builder) mod has_secret_connections;

/// Original state of the [`FloorBuilder`]. Entry point to the state machine.
pub(in crate) mod new;

/// State that represents a [`FloorBuilder`] that has has the cellular automata run on it, smoothing out the edges of the caves.
pub(in crate::floor_builder) mod smoothed;
