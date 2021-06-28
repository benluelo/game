use std::{convert::TryInto, num::NonZeroU16};

use game::{Dungeon, DungeonType};

fn main() {
    let d = Dungeon::new(
        100.try_into().unwrap(),
        150.try_into().unwrap(),
        NonZeroU16::new(10).unwrap(),
        DungeonType::Cave,
        false,
    );
}
