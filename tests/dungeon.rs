use std::num::NonZeroUsize;

use game::{dungeon::DungeonType, Dungeon};

#[test]
fn test_dungeon_creation() {
    let d = Dungeon::new(50, 100, NonZeroUsize::new(10).unwrap(), DungeonType::Cave);

    std::fs::write("dungeon_test.json", d.to_json().unwrap().as_bytes()).unwrap();
}
