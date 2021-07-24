use ansi_term::{ANSIString, Colour::Green, Style};
use serde::{Deserialize, Serialize};

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
    /// A secret door to a secret passageway. May or may not require a key to open.
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
    /// Entrance to the floor. Both entrances and exits are to be one-way paths; once
    /// you leave a floor you cannot go back to it.
    Entrance,
    /// Exit to the next floor, or the end of the dungeon. Both entrances and exits
    /// are to be one-way paths; once you leave a floor you cannot go back to it.
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
    pub fn as_u8(&self) -> u8 {
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

    /// Returns `true` if the dungeon_tile is [`Empty`].
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Returns `true` if the dungeon_tile is [`Wall`].
    pub fn is_wall(&self) -> bool {
        matches!(self, Self::Wall)
    }

    /// Returns `true` if the dungeon_tile is [`SecretDoor`].
    pub fn is_secret_door(&self) -> bool {
        matches!(self, Self::SecretDoor { .. })
    }

    /// Returns `true` if the dungeon_tile is [`SecretPassage`].
    pub fn is_secret_passage(&self) -> bool {
        matches!(self, Self::SecretPassage)
    }

    /// Returns `true` if the dungeon_tile is [`TreasureChest`].
    pub fn is_treasure_chest(&self) -> bool {
        matches!(self, Self::TreasureChest { .. })
    }

    /// Returns `true` if the dungeon_tile is [`Entrance`].
    pub fn is_entrance(&self) -> bool {
        matches!(self, Self::Entrance)
    }

    /// Returns `true` if the dungeon_tile is [`Exit`].
    pub fn is_exit(&self) -> bool {
        matches!(self, Self::Exit)
    }
}

impl Default for DungeonTile {
    fn default() -> Self {
        Self::Empty
    }
}

impl DungeonTile {
    /// Returns whether or not the tile can be traversed by the player.
    pub fn is_solid(&self) -> bool {
        matches!(self, DungeonTile::Wall | DungeonTile::TreasureChest { .. })
    }

    // REVIEW: Move this to a [`ToBlockCharachter`] implementation?
    pub(crate) fn _print(&self, var1: bool, var2: bool) -> ANSIString {
        let style = Style::new();

        if var1 {
            style.fg(Green);
        }

        if var2 {
            style.bold();
        }

        style.paint(match self {
            DungeonTile::Empty => "  ",
            DungeonTile::Wall => "██",
            DungeonTile::SecretDoor { .. } => "SD",
            DungeonTile::SecretPassage => "<>",
            DungeonTile::TreasureChest { .. } => "TC",
            DungeonTile::Entrance => "EN",
            DungeonTile::Exit => "EX",
        })
    }
}
