use std::num::NonZeroUsize;

use crate::dungeon::FloorBuilder;

mod dungeon;

pub fn floor_builder() {
    /* let random_filled_floor =  */
    let mut fb = FloorBuilder::new(
        NonZeroUsize::new(100).unwrap(),
        NonZeroUsize::new(100).unwrap(),
    );
    let caves = fb.get_cave_borders();
    let connections = fb.build_connections(caves);
    fb.draw_connections(connections);
    // let _ = min_spanning_tree(&connections)
    //     .inspect(|mst| {
    //         dbg!(mst);
    //     })
    //     .collect::<Vec<_>>();
    // println!(
    //     "{}",
    //     floor_builder.pretty(
    //         connections
    //             .into_iter()
    //             .map(|i| vec![i.0 .1, i.1 .1])
    //             .flatten()
    //             .collect(),
    //         vec![]
    //     )
    // );
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
