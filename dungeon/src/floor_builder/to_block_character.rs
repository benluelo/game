use bounded_int::BoundedInt;

use super::{MAX_FLOOR_SIZE, MIN_FLOOR_SIZE};

#[allow(dead_code)]
pub(crate) fn print_vec_2d<T: ToBlockDrawingCharacter>(
    vec: &[T],
    width: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
) -> String {
    vec.chunks(width.as_unbounded() as usize)
        .map(|i| i.iter().map(|j| j.to_block()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

pub trait ToBlockDrawingCharacter {
    fn to_block(&self) -> &'static str;
}

impl ToBlockDrawingCharacter for bool {
    fn to_block(&self) -> &'static str {
        #[allow(clippy::non_ascii_literal)]
        match self {
            true => "██",
            false => "░░",
        }
    }
}
