use serde::{Deserialize, Serialize};

use crate::floor_builder::to_block_character::ToBlockDrawingCharacter;

/// The various things a tile can be in a dungeon floor.
///
/// Note that these are just base map features. Players and mobs will interact
/// with these tiles, but are not tiles themselves.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DungeonTile {
    /// Empty space. Traversable.
    Empty,
    /// Solid wall. Not traversable.
    Wall,
    /// A secret door to a secret passageway. May or may not require a key to
    /// open.
    SecretDoor {
        /// Whether or not the door requires a key to open.
        requires_key: bool,
        /// Whether or not the door is open. All doors start off closed.
        is_open: bool,
    },
    /// A secret passageway between two secret doors.
    SecretPassage,
    /// A treasure chest that will at some point contain something.
    // TODO: Add treasure (lol)
    TreasureChest {
        /// Placeholder for now to make refactoring easier as fields are added.
        contents: (),
    },
    /// Entrance to the floor. Both entrances and exits are to be one-way paths;
    /// once you leave a floor you cannot go back to it.
    Entrance,
    /// Exit to the next floor, or the end of the dungeon. Both entrances and
    /// exits are to be one-way paths; once you leave a floor you cannot go
    /// back to it.
    Exit,
}

impl DungeonTile {
    /// Color map for use in exporting the floor to a gif.
    ///
    /// See [`DungeonTile::as_u8`].
    pub const COLOR_MAP: [u8; 21] = [
        0xFF, 0xFF, 0xFF, // black
        0x00, 0x00, 0x00, // white
        0xFF, 0x00, 0x00, // red
        0x00, 0xFF, 0x00, // green
        0x00, 0x00, 0xFF, // blue
        0xFF, 0x00, 0xFF, // purple
        0xAA, 0x40, 0x00, // yellow
    ];

    /// Returns the u8 value of the tile for use in exporting to gif.
    ///
    /// This operation is lossy; any variant with attached information
    /// does not have it's information encoded in it's u8 value.
    #[must_use]
    #[allow(clippy::trivially_copy_pass_by_ref)] // so it can be passed directly to Iterator::map
    pub const fn as_u8(&self) -> u8 {
        match self {
            DungeonTile::Empty => 0,
            DungeonTile::Wall => 1,
            DungeonTile::SecretDoor { .. } => 2,
            DungeonTile::SecretPassage => 3,
            DungeonTile::TreasureChest { .. } => 4,
            DungeonTile::Entrance => 5,
            DungeonTile::Exit => 6,
        }
    }

    /// Returns `true` if `self` is [`DungeonTile::Empty`].
    #[must_use]
    pub const fn is_empty(self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Returns `true` if `self` is [`DungeonTile::Wall`].
    #[must_use]
    pub const fn is_wall(self) -> bool {
        matches!(self, Self::Wall)
    }

    /// Returns `true` if `self` is [`DungeonTile::SecretDoor`].
    #[must_use]
    pub const fn is_secret_door(self) -> bool {
        matches!(self, Self::SecretDoor { .. })
    }

    /// Returns `true` if `self` is [`DungeonTile::SecretPassage`].
    #[must_use]
    pub const fn is_secret_passage(self) -> bool {
        matches!(self, Self::SecretPassage)
    }

    /// Returns `true` if `self` is [`DungeonTile::TreasureChest`].
    #[must_use]
    pub const fn is_treasure_chest(self) -> bool {
        matches!(self, Self::TreasureChest { .. })
    }

    /// Returns `true` if `self` is [`DungeonTile::Entrance`].
    #[must_use]
    pub const fn is_entrance(self) -> bool {
        matches!(self, Self::Entrance)
    }

    /// Returns `true` if `self` is [`DungeonTile::Exit`].
    #[must_use]
    pub const fn is_exit(self) -> bool {
        matches!(self, Self::Exit)
    }

    /// Returns whether or not the tile can be traversed by the player.
    #[must_use]
    pub const fn is_solid(self) -> bool {
        matches!(self, DungeonTile::Wall | DungeonTile::TreasureChest { .. })
    }
}

impl Default for DungeonTile {
    fn default() -> Self {
        Self::Empty
    }
}

impl ToBlockDrawingCharacter for DungeonTile {
    fn to_block(&self) -> &'static str {
        #[allow(clippy::non_ascii_literal)]
        match self {
            DungeonTile::Empty => "  ",
            DungeonTile::Wall => "██",
            DungeonTile::SecretDoor { .. } => "SD",
            DungeonTile::SecretPassage => "<>",
            DungeonTile::TreasureChest { .. } => "TC",
            DungeonTile::Entrance => "EN",
            DungeonTile::Exit => "EX",
        }
    }
}
