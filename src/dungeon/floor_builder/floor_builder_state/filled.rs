use crate::dungeon::{Floor, FloorBuilder};

use super::{FloorBuilderState, Smoothable};

// The final state of the floor builder.
#[derive(Debug)]
pub(in crate::dungeon::floor_builder) struct Filled {}
impl FloorBuilderState for Filled {}
impl Smoothable for Filled {}

// impl FloorBuilder<Filled> {
//     pub(in crate::dungeon::floor_builder) fn finish(self) -> Floor {
//         Floor {
//             height: self.height,
//             width: self.width,
//             data: self.map,
//         }
//     }
// }

impl<S: FloorBuilderState> FloorBuilder<S> {
    pub(in crate::dungeon::floor_builder) fn finish(self) -> Floor {
        if let Some(frames) = self.frames {
            use gif::{Encoder, Repeat};

            let color_map = &[
                0xFF, 0xFF, 0xFF, // black
                0x00, 0x00, 0x00, // white
                0xFF, 0x00, 0x00, // red
                0x00, 0xFF, 0x00, // green
                0x00, 0x00, 0xFF, // blue
            ];

            let mut image = vec![];
            {
                let mut encoder = Encoder::new(
                    &mut image,
                    self.width.as_unbounded() as u16,
                    self.height.as_unbounded() as u16,
                    color_map,
                )
                .unwrap();
                encoder.set_repeat(Repeat::Finite(0)).unwrap();
                for frame in frames {
                    dbg!("writing frame");
                    encoder.write_frame(&frame).unwrap();
                }
            }

            std::fs::write("floor.gif", image).unwrap();
        }
        Floor {
            height: self.height,
            width: self.width,
            data: self.map,
        }
    }
}
