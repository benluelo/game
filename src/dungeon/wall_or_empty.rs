use ansi_term::{ANSIString, Colour::Green, Style};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum DungeonTile {
    Empty = 0x00,
    Wall = 0x01,
    Chest = 0x02,
    SecretDoor = 0x03,
    SecretPassage = 0x04,
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
        matches!(self, DungeonTile::Empty)
    }

    pub(crate) fn print(&self, var1: bool, var2: bool) -> ANSIString {
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
            DungeonTile::Chest => "[]",
            DungeonTile::SecretDoor => "SD",
            DungeonTile::SecretPassage => "<>",
        })
    }
}
