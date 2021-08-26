use std::ops::{Add, Mul};

use bounded_int::BoundedInt;
use noise::{Billow, MultiFractal, NoiseFn, Seedable};
use rand::{thread_rng, Rng};

use crate::{
    floor_builder::{
        floor_builder_state::random_filled::RandomFilled, MAX_FLOOR_SIZE, MIN_FLOOR_SIZE,
        RANDOM_FILL_WALL_PERCENT_CHANCE,
    },
    point_index::{iter_points, PointIndex},
    DungeonTile, FloorBuilder,
};

use super::FloorBuilderState;

/// A blank floor builder, with all values in the floor map and the noise map
/// set to their default.
#[derive(Debug)]
pub(in crate::floor_builder) struct Blank {}
impl FloorBuilderState for Blank {
    const TYPE_NAME: &'static str = "Blank";
}

impl FloorBuilder<Blank> {
    /// TODO: Split this function into two parts, `random_fill` and
    /// `trace_entrance_exit` (or something along those lines)
    pub(in crate::floor_builder) fn random_fill(mut self) -> FloorBuilder<RandomFilled> {
        let mut rng = thread_rng();

        let mut noise = create_billow(&mut rng);

        // build initial maps (walls and noise)
        for point in iter_points(self.width, self.height) {
            *self.noise_map.at_mut(point, self.width) = get_noise_value(
                &mut noise,
                point.column.get(),
                point.row.get(),
                self.height,
                self.width,
            );

            if self.is_out_of_bounds(point) {
                *self.map.at_mut(point, self.width) = DungeonTile::Wall;
                continue;
            }

            // make a wall some percent of the time
            *self.map.at_mut(point, self.width) =
                if rng.gen_range(0..=100) <= RANDOM_FILL_WALL_PERCENT_CHANCE {
                    DungeonTile::Wall
                } else {
                    DungeonTile::Empty
                }
        }

        // original noisy walls map
        self.frame_from_current_state(100);

        FloorBuilder {
            extra: RandomFilled {},
            height: self.height,
            width: self.width,
            map: self.map,
            noise_map: self.noise_map,
            frames: self.frames,
            id: self.id,
        }
    }
}

/// Gets the noise value for the provided billow at the row and column
/// specified.
fn get_noise_value(
    noise: &mut Billow,
    column: BoundedInt<0, MAX_FLOOR_SIZE>,
    row: BoundedInt<0, MAX_FLOOR_SIZE>,
    height: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
    width: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
) -> u16 {
    /// Makes it easier for me to think about powers of two
    mod u4 {
        /// 2^4
        pub const MAX: u8 = 16;
    }

    #[allow(clippy::cast_possible_truncation)]
    let n = noise
        .get([
            (column.as_unbounded() as f64 / width.as_unbounded() as f64),
            (row.as_unbounded() as f64 / height.as_unbounded() as f64),
        ])
        // change the range from [-1, 1] to [-8, 8]
        .mul((u4::MAX / 2) as f64)
        // change the range from [-8, 8] to [0, 16]
        .add((u4::MAX) as f64)
        // change the range from [0, 16] to [0, 2^16]
        .powi(4)
        // round up
        .ceil() as u16;

    // these ⭐ magic numbers ⭐ have been hand crafted to perfection
    // don't touch them pls
    // TODO: Figure out what these magic numbers do lol
    #[allow(clippy::cast_possible_truncation)] // it doesnt truncate (even if it did that'd be ok)
    if n <= (u16::MAX as f64 / 2.5) as u16 {
        n / 2
    } else {
        u16::MAX
    }
}

/// Creates a [`Billow`] using some magic numbers that have been fine tuned to
/// work well.
///
/// Don't touch 'em
fn create_billow(rng: &mut impl rand::Rng) -> Billow {
    Billow::new()
        .set_octaves(1)
        .set_frequency(5.0)
        .set_lacunarity(0.001)
        .set_persistence(0.001)
        .set_seed(rng.gen())
}

#[cfg(test)]
mod test_noise {
    use std::{convert::TryInto, path::Path};

    use itertools::Itertools;

    use super::*;

    #[test]
    fn test_noise_map_prettyness_lol_idk() {
        const WIDTH: i32 = 200;
        const HEIGHT: i32 = 200;
        let mut noise_map = vec![0; (WIDTH * HEIGHT) as usize];

        let mut rng = thread_rng();

        let mut noise = create_billow(&mut rng);

        for point in iter_points(
            BoundedInt::<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>::new(WIDTH).unwrap(),
            BoundedInt::<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>::new(HEIGHT).unwrap(),
        ) {
            let n = get_noise_value(
                &mut noise,
                point.column.get(),
                point.row.get(),
                HEIGHT.try_into().unwrap(),
                WIDTH.try_into().unwrap(),
            );
            *noise_map.at_mut(point, BoundedInt::<0, MAX_FLOOR_SIZE>::new(WIDTH).unwrap()) = n;
        }

        // let pixels = noise_map
        //     .into_iter()
        //     .flat_map(|x| {
        //         let red = (x >> 16) & 0xFF;
        //         let green = (x >> 8) & 0xFF;
        //         let blue = x & 0xFF;
        //         [red as u8, green as u8, blue as u8]
        //     })
        //     .collect_vec();

        let _ = image::save_buffer(
            &Path::new(&"noise.png"),
            &*noise_map
                .into_iter()
                .flat_map(|integer| [integer as u8, (integer >> 8) as u8])
                .collect_vec(),
            WIDTH as u32,
            HEIGHT as u32,
            image::ColorType::L16,
        )
        .unwrap();
    }
}
