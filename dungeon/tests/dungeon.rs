use std::{convert::TryInto, num::NonZeroU16};

use game::{Dungeon, DungeonType};

#[test]
fn test_dungeon_creation() {
    let d = Dungeon::new(
        100.try_into().unwrap(),
        150.try_into().unwrap(),
        NonZeroU16::new(10).unwrap(),
        DungeonType::Cave,
        true,
    );

    std::fs::write("dungeon_test.gif", d.to_gif()).unwrap();
}
