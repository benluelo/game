use std::{convert::TryInto, num::NonZeroUsize};

use game::{dungeon::DungeonType, Dungeon};

#[test]
fn test_dungeon_creation() {
    let d = Dungeon::new(
        100.try_into().unwrap(),
        150.try_into().unwrap(),
        NonZeroUsize::new(1).unwrap(),
        DungeonType::Cave,
    );

    std::fs::write("dungeon_test.gif", d.to_gif()).unwrap();
}
