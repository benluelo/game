use std::num::NonZeroUsize;

use game::{dungeon::DungeonType, Dungeon};

#[test]
fn test_dungeon_creation() {
    let d = Dungeon::new(100, 150, NonZeroUsize::new(10).unwrap(), DungeonType::Cave);

    std::fs::write("dungeon_test.gif", d.to_gif()).unwrap();
}
