use std::num::NonZeroUsize;

use game::{dungeon::DungeonType, Dungeon};

#[test]
fn test_dungeon_creation() {
    let d = Dungeon::new(
        NonZeroUsize::new(50).unwrap(),
        NonZeroUsize::new(100).unwrap(),
        NonZeroUsize::new(10).unwrap(),
        DungeonType::Cave,
    );

    std::fs::write("dungeon_test.json", d.to_json().unwrap().as_bytes()).unwrap();
}
