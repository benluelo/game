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
}

impl DungeonTile {
    pub fn as_u8(&self) -> u8 {
        match self {
            DungeonTile::Empty => 0,
            DungeonTile::Wall => 1,
            DungeonTile::SecretDoor { requires_key: _ } => 2,
            DungeonTile::SecretPassage => 3,
            DungeonTile::TreasureChest { contents: _ } => 4,
        }
    }
}

// struct Test {
//     a: String,
//     b: std::num::NonZeroUsize,
// }

// #[derive(Debug)]
// struct ZeroError;

// impl Test {
//     pub fn new(a: String, b: usize) -> Result<Test, ZeroError> {
//         Ok(Self {
//             a,
//             b: NonZeroUsize::new(b).ok_or(ZeroError)?,
//         })
//     }
// }

// fn main() {
//     let safe_to_unwrap = Test::new("hi".into(), 100).unwrap();
//     let will_panic = Test::new("oh no!".into(), 0).unwrap();
// }

// mod custom_serde {
//     use std::collections::HashMap;

//     use serde::{Deserialize, Deserializer, Serialize, Serializer, de::{self, Unexpected}, ser::{SerializeMap, SerializeStruct}};

//     macro_rules! create_serde_mod {
//         ($($name:ident -> $value:literal)+) => {
//             $(pub(super) mod $name {
//                 use serde::{de::{self, Unexpected}, Deserializer, Deserialize, Serializer};

//                 pub(in super::super) fn serialize<S>(s: S) -> Result<S::Ok, S::Error>
//                 where
//                     S: Serializer,
//                 {
//                     s.serialize_i32($value)
//                 }

//                 pub(in super::super) fn deserialize<'de, D>(d: D) -> Result<(), D::Error>
//                 where
//                     D: Deserializer<'de>,
//                 {
//                     match i32::deserialize(d)? {
//                         $value => Ok(()),
//                         n => Err(de::Error::custom(Unexpected::Signed(n.into()))),
//                     }
//                 }
//             })+
//         };
//     }

//     pub(in super::super) fn serialize<S>(contents: &(), s: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         #[derive(Serialize, Deserialize)]
//         struct TreasureChest {
//             #[serde(rename = "TreasureChest")]
//             inner: Inner,
//         }
//         #[derive(Serialize, Deserialize)]
//         struct Inner {
//             contents: (),
//         }

//         TreasureChest {
//             inner: Inner {
//                 contents: *contents,
//             },
//         };

//         let mut map = s.serialize_struct(
//             "TreasureChest", 1)?;
//         map.serialize_field(
//             &TreasureChest {
//                 inner: Inner {
//                     contents: *contents,
//                 },
//             },
//         )?;
//         map.end()
//     }

//     pub(in super::super) fn deserialize<'de, D>(d: D) -> Result<(), D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         match i32::deserialize(d)? {
//             0 => Ok(()),
//             n => Err(de::Error::custom(Unexpected::Signed(n.into()))),
//         }
//     }

//     create_serde_mod! {
//         empty -> 0
//         wall -> 1
//         secret_passage -> 2
//     }
// }

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
        })
    }
}

// #[cfg(test)]
// mod test_dungeon_tile_serialize_deserialize {
//     use super::*;
//     use serde_test::*;

//     #[test]
//     fn test_empty() {
//         assert_tokens(&DungeonTile::Empty, &[Token::I32(0)])
//     }

//     #[test]
//     fn test_wall() {
//         assert_tokens(&DungeonTile::Wall, &[Token::I32(1)])
//     }

//     #[test]
//     fn test_chest() {
//         assert_tokens(
//             &DungeonTile::TreasureChest { contents: () },
//             &[
//                 Token::Map { len: Some(1) },
//                 Token::Str("TreasureChest"),
//                 Token::Struct { name: "T", len: 1 },
//                 Token::Str("contents"),
//                 Token::Unit,
//                 Token::StructEnd,
//                 Token::MapEnd,
//             ],
//         )
//     }

//     #[test]
//     fn test_secret_door() {
//         assert_tokens(
//             &DungeonTile::SecretDoor { requires_key: true },
//             &[
//                 Token::StructVariant {
//                     name: "DungeonTile",
//                     variant: "SecretDoor",
//                     len: 1,
//                 },
//                 Token::Str("requires_key"),
//                 Token::Bool(true),
//                 Token::StructVariantEnd,
//             ],
//         )
//     }
// }
