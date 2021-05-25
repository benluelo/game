use std::num::NonZeroUsize;

use dungeon::DungeonType;

pub use crate::dungeon::Dungeon;

mod command;
pub mod dungeon;
mod example;

// bench
pub fn create_dungeon(width: usize, height: usize) {
    let _ = Dungeon::new(
        height,
        width,
        NonZeroUsize::new(10).unwrap(),
        DungeonType::Cave,
    );
}
