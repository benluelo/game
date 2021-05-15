use std::num::NonZeroUsize;

use dungeon::DungeonType;

pub use crate::dungeon::Dungeon;

pub mod dungeon;

// bench
pub fn create_dungeon(height: usize, width: usize) {
    let _ = Dungeon::new(
        NonZeroUsize::new(height).unwrap(),
        NonZeroUsize::new(width).unwrap(),
        NonZeroUsize::new(10).unwrap(),
        DungeonType::Cave,
    );
}
