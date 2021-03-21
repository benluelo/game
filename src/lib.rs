use std::num::NonZeroUsize;

use crate::dungeon::FloorBuilder;

mod dungeon;

pub fn floor_builder() {
    /* let random_filled_floor =  */
    FloorBuilder::new(
        NonZeroUsize::new(100).unwrap(),
        NonZeroUsize::new(100).unwrap(),
    );
    // let formatted = random_filled_floor
    //     .map
    //     .iter()
    //     .map(|i| {
    //         i.iter()
    //             .map(|j| match j {
    //                 WallOrEmpty::Empty => "  ",
    //                 WallOrEmpty::Wall => "██",
    //             })
    //             .collect::<String>()
    //     })
    //     .collect::<Vec<_>>()
    //     .join("\n");

    // println!("{}", &formatted)
}
