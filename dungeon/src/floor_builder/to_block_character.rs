use bounded_int::BoundedInt;

use super::{MAX_FLOOR_SIZE, MIN_FLOOR_SIZE};

pub(crate) fn print_vec_2d<T: ToBlockDrawingCharacter>(
    vec: Vec<T>,
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
        match self {
            true => "██",
            false => "░░",
        }
    }
}

// impl ToBlockDrawingCharacter for u128 {
//     fn to_block(&self) -> &'static str {
//         match self {
//             true => "██",
//             false => "░░",
//         }
//     }
// }
