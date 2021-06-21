use std::{convert::TryInto, num::NonZeroUsize};

use dungeon::DungeonType;

pub use crate::dungeon::Dungeon;

pub mod bounded_int;
mod command;
pub mod dungeon;
mod example;

// bench
pub fn create_dungeon(width: i32, height: i32) {
    let _ = Dungeon::new(
        height.try_into().unwrap(),
        width.try_into().unwrap(),
        NonZeroUsize::new(10).unwrap(),
        DungeonType::Cave,
    );
}
