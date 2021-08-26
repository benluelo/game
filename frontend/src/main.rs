#[allow(clippy::needless_pass_by_value)]
pub mod constants;
mod dungeon;
pub mod key_press_handling;
pub mod player;
pub mod utils;

use ::dungeon::DungeonTile;
use bevy::{ecs::schedule::ReportExecutionOrderAmbiguities, prelude::*};
use std::{num::NonZeroU16, ops::Index};

use crate::{dungeon::DungeonPlugin, key_press_handling::KeyPressHandlingPlugin};

pub struct CurrentFloor(pub NonZeroU16);

pub struct ExitFloor;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "game".to_string(),
            width: 500.0,
            height: 500.0,
            ..Default::default()
        })
        .add_startup_system(setup.system().label("setup"))
        .add_plugins(DefaultPlugins)
        .add_plugin(KeyPressHandlingPlugin)
        .add_plugin(DungeonPlugin)
        .insert_resource(ReportExecutionOrderAmbiguities)
        .run();
}

fn setup(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    server.watch_for_changes().unwrap();

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands.insert_resource(Materials {
        empty_material: materials.add(
            // Color::WHITE.into()
            server.load("empty.png").into(),
        ),
        wall_material: materials.add(Color::BLACK.into()),
        secret_door_material: materials.add(Color::RED.into()),
        secret_passage_material: materials.add(Color::LIME_GREEN.into()),
        treasure_chest_material: materials.add(Color::BLUE.into()),
        entrance_material: materials.add(Color::PINK.into()),
        exit_material: materials.add(Color::PURPLE.into()),
        player_material: materials.add(server.load("arrow.png").into()),
    });
}

pub struct Tile {
    #[allow(dead_code)]
    tile_type: DungeonTile,
}

pub struct Materials {
    empty_material: Handle<ColorMaterial>,
    wall_material: Handle<ColorMaterial>,
    secret_door_material: Handle<ColorMaterial>,
    secret_passage_material: Handle<ColorMaterial>,
    treasure_chest_material: Handle<ColorMaterial>,
    entrance_material: Handle<ColorMaterial>,
    exit_material: Handle<ColorMaterial>,
    player_material: Handle<ColorMaterial>,
}

impl Index<DungeonTile> for Materials {
    type Output = Handle<ColorMaterial>;

    fn index(&self, index: DungeonTile) -> &Self::Output {
        match index {
            DungeonTile::Empty => &self.empty_material,
            DungeonTile::Wall => &self.wall_material,
            DungeonTile::SecretDoor { .. } => &self.secret_door_material,
            DungeonTile::SecretPassage => &self.secret_passage_material,
            DungeonTile::TreasureChest { .. } => &self.treasure_chest_material,
            DungeonTile::Entrance => &self.entrance_material,
            DungeonTile::Exit => &self.exit_material,
        }
    }
}
