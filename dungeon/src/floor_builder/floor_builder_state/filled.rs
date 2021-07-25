use std::convert::TryInto;

use crate::{DungeonTile, Floor, FloorBuilder};

use super::{FloorBuilderState, Smoothable};

// The final state of the floor builder.
#[derive(Debug)]
pub(in crate::floor_builder) struct Filled {}
impl FloorBuilderState for Filled {}
impl Smoothable for Filled {}

// impl FloorBuilder<Filled> {
//     pub(in crate::floor_builder) fn finish(self) -> Floor {
//         Floor {
//             height: self.height,
//             width: self.width,
//             data: self.map,
//         }
//     }
// }

impl<S: FloorBuilderState> FloorBuilder<S> {
    pub(in crate::floor_builder) fn finish(self) -> Floor {
        if let Some(frames) = self.frames {
            use gif::{Encoder, Repeat};

            let mut image = vec![];
            {
                let mut encoder = Encoder::new(
                    &mut image,
                    self.width.as_unbounded().try_into().unwrap(),
                    self.height.as_unbounded().try_into().unwrap(),
                    &DungeonTile::COLOR_MAP,
                )
                .unwrap();
                encoder.set_repeat(Repeat::Finite(0)).unwrap();
                for frame in frames {
                    encoder.write_frame(&frame).unwrap();
                }
            }

            let out_path = format!("out/floor_{}.gif", self.id);
            match std::fs::write(&out_path, image) {
                Ok(_) => {
                    eprintln!("Successfully wrote to {}", out_path)
                }
                Err(err) => {
                    eprintln!("Error writing to {}: {}", out_path, err)
                }
            }
        }
        Floor {
            height: self.height,
            width: self.width,
            data: self.map,
        }
    }
}
