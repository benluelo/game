use std::{convert::TryInto, num::NonZeroU16};

use dungeon::{Dungeon, DungeonType};

fn main() {
    let _d = Dungeon::new(
        100.try_into().unwrap(),
        150.try_into().unwrap(),
        NonZeroU16::new(10).unwrap(),
        DungeonType::Cave,
        true,
    );
}
