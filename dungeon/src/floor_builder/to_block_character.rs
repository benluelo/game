use bounded_int::BoundedInt;

use super::{MAX_FLOOR_SIZE, MIN_FLOOR_SIZE};

/// Returns the supplied [`Vec<T>`] as a [`String`], using the
/// [`ToAsciiCharacter`] implementation for `T` to do so.
#[allow(dead_code)]
pub(crate) fn _print_vec_2d<T: ToAsciiCharacter>(
    vec: &[T],
    width: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
) -> String {
    vec.chunks(width.as_unbounded() as usize)
        .map(|i| {
            i.iter()
                .flat_map(|j| j.to_ascii_chars())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Represents a type that can be 'pretty-printed' using ascii characters.
pub trait ToAsciiCharacter {
    /// Return the 2-character wide ascii representation of the type.
    ///
    /// Note: Return type is 2 characters because monospace fonts are typically
    /// about twice as tall as they are wide, so two characters creates
    /// (*almost*) a square.
    fn to_ascii_chars(&self) -> [char; 2];
}

impl ToAsciiCharacter for bool {
    fn to_ascii_chars(&self) -> [char; 2] {
        #[allow(clippy::non_ascii_literal)]
        match self {
            true => ['█', '█'],
            false => ['░', '░'],
        }
    }
}
