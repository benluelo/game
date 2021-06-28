use ansi_term::{ANSIString, Colour::Green, Style};
use serde::{Deserialize, Serialize};

// #[serde(untagged)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DungeonTile {
    // #[serde(with = "custom_serde::empty")]
    // #[serde(rename = "0")]
    Empty,
    // #[serde(with = "custom_serde::wall")]
    // #[serde(rename = "1")]
    Wall,
    SecretDoor { requires_key: bool },
    // #[serde(with = "custom_serde::secret_passage")]
    // #[serde(rename = "2")]
    SecretPassage,
    // #[serde(serialize_with = "custom_serde::serialize")]
    TreasureChest { contents: () },
    Entrance,
    Exit,
}

impl DungeonTile {
    pub const COLOR_MAP: [u8; 21] = [
        0xFF, 0xFF, 0xFF, // black
        0x00, 0x00, 0x00, // white
        0xFF, 0x00, 0x00, // red
        0x00, 0xFF, 0x00, // green
        0x00, 0x00, 0xFF, // blue
        0xFF, 0x00, 0xFF, // purple
        0xAA, 0x40, 0x00, // yellow
    ];

    pub fn as_u8(&self) -> u8 {
        match self {
            DungeonTile::Empty => 0,
            DungeonTile::Wall => 1,
            DungeonTile::SecretDoor { requires_key: _ } => 2,
            DungeonTile::SecretPassage => 3,
            DungeonTile::TreasureChest { contents: _ } => 4,
            DungeonTile::Entrance => 5,
            DungeonTile::Exit => 6,
        }
    }
}

impl Default for DungeonTile {
    fn default() -> Self {
        Self::Empty
    }
}

impl DungeonTile {
    pub(crate) fn is_wall(&self) -> bool {
        matches!(self, DungeonTile::Wall)
    }

    pub(crate) fn is_empty(&self) -> bool {
        !self.is_wall()
        // matches!(self, DungeonTile::Empty)
    }

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
