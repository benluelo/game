#[allow(clippy::needless_pass_by_value)]
pub mod constants;
pub mod key_press_handling;
pub mod player;

use bevy::{ecs::schedule::ReportExecutionOrderAmbiguities, prelude::*, render::camera::Camera};
use dungeon::{Dungeon, DungeonTile, DungeonType, Point};
use std::{convert::TryInto, num::NonZeroU16, ops::Index};

use crate::{
    constants::{PLAYER_MOVEMENT_DELAY_SECONDS, PLAYER_MOVING_TIME_SECONDS, TILE_Z_INDEX},
    key_press_handling::KeyPressTime,
    player::{Player, PlayerDirection, PlayerState},
    utils::{player_sprite_bundle, point_to_transform},
};

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "game".to_string(),
            width: 500.0,
            height: 500.0,
            ..Default::default()
        })
        .insert_resource(Dungeon::new(
            80_i32.try_into().unwrap(),
            50_i32.try_into().unwrap(),
            NonZeroU16::new(1).unwrap(),
            DungeonType::Cave,
            false,
        ))
        // TODO: Make this a component of `Player`, not a resource.
        .insert_resource(PlayerState::Still)
        .add_plugins(DefaultPlugins)
        // .insert_resource(Msaa { samples: 4 })
        .insert_resource(KeyPressTime(Default::default()))
        .add_startup_system(setup.system().label("setup"))
        .add_startup_stage_after(
            StartupStage::Startup,
            "spawn_player_and_board",
            SystemStage::single(spawn_player_and_board.system()),
        )
        // .add_startup_system(spawn_player_and_board.system().after("setup"))
        .add_system_to_stage(CoreStage::PreUpdate, key_press_handling::system())
        .add_system_set(
            SystemSet::new()
                .with_system(
                    player_movement_input_handling
                        .system()
                        .label("player_movement_input_handling"),
                )
                .with_system(
                    smooth_player_movement
                        .system()
                        .label("smooth_player_movement")
                        .after("player_movement_input_handling"),
                )
                .with_system(
                    camera_player_tracking
                        .exclusive_system()
                        .at_end()
                        .label("camera_player_tracking")
                        .after("smooth_player_movement"),
                ),
        )
        .insert_resource(ReportExecutionOrderAmbiguities)
        // .add_system_set_to_stage(
        //     CoreStage::PostUpdate,
        //     SystemSet::new()
        //         .with_system(position_translation.system())
        //         .with_system(size_scaling.system()),
        // )
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

/// The internal position of something in the world; where in the world it can
/// be considered to be.
pub struct Position(Point);

fn spawn_player_and_board(
    mut commands: Commands,
    dungeon: Res<Dungeon>,
    materials: Res<Materials>,
) {
    let floor = dungeon.floors.first().unwrap();

    for (point, tile) in floor.iter_points_and_tiles() {
        if matches!(tile, DungeonTile::Entrance) {
            commands
                .spawn_bundle(player_sprite_bundle(&materials, point, floor))
                .insert(Player)
                .insert(PlayerDirection::Up)
                .insert(Position(point));
        }

        commands
            .spawn_bundle(utils::tile_sprite_bundle(&materials, tile, point, floor))
            .insert(Tile { tile_type: *tile })
            .insert(Position(point));
    }
}
pub mod utils;

/// Moves the player between squares smoothly.
fn smooth_player_movement(
    time: Res<Time>,
    dungeon: Res<Dungeon>,
    player_direction: Query<&PlayerDirection, With<Player>>,
    mut player_state: ResMut<PlayerState>,
    mut player_position: Query<&mut Position, With<Player>>,
    mut player_transform: Query<&mut Transform, With<Player>>,
) {
    let floor = dungeon.floors.first().unwrap();
    let mut position = player_position.single_mut().unwrap();

    let mut transform = player_transform.single_mut().unwrap();

    transform.rotation = player_direction.single().unwrap().to_rotation();

    let mut done_moving = false;
    // dbg!(&*player_state);
    match *player_state {
        PlayerState::Moving {
            destination,
            ref mut timer,
        } => {
            timer.tick(time.delta());

            transform.translation = point_to_transform(position.0, floor, TILE_Z_INDEX)
                .translation
                .lerp(
                    point_to_transform(destination, floor, TILE_Z_INDEX).translation,
                    timer.percent(),
                );

            if timer.finished() {
                done_moving = true;
                position.0 = destination;
            }
        }
        PlayerState::Still => return,
    }

    if done_moving {
        *player_state = PlayerState::Still;
    }
}

/// makes the camera follow the player.
/// TODO: when the player is in corners, make the camera stay in the same spot.
/// NOTE: this will first require fixing how the tiles are drawn
#[allow(clippy::type_complexity)]
fn camera_player_tracking(
    mut set: QuerySet<(
        Query<&Transform, With<Player>>,
        Query<&mut Transform, With<Camera>>,
    )>,
) {
    set.q1_mut().single_mut().unwrap().translation = set.q0().single().unwrap().translation;
}

/// Moves the player by changing it's internal [`dungeon::Point`] according to
/// the input.
fn player_movement_input_handling(
    key_press_time: ResMut<KeyPressTime>,
    dungeon: Res<Dungeon>,
    mut player_state: ResMut<PlayerState>,
    player_position: Query<&Position, With<Player>>,
    mut player_direction: Query<&mut PlayerDirection, With<Player>>,
) {
    let floor = &dungeon.floors[0];

    if let Ok(player_position) = player_position.single() {
        if matches!(*player_state, PlayerState::Moving { .. }) {
            return;
        };

        let mut player_direction = player_direction.single_mut().unwrap();

        if let Some((new_direction, &time_pressed)) = key_press_time
            .0
            .iter()
            .filter_map(|(k, v)| match k {
                KeyCode::Up => Some((PlayerDirection::Up, v)),
                KeyCode::Down => Some((PlayerDirection::Down, v)),
                KeyCode::Left => Some((PlayerDirection::Left, v)),
                KeyCode::Right => Some((PlayerDirection::Right, v)),
                _ => None,
            })
            .reduce(|a, b| if a.1 >= b.1 { a } else { b })
        {
            // tap to change direction
            if time_pressed < PLAYER_MOVEMENT_DELAY_SECONDS && new_direction != *player_direction {
                {
                    *player_direction = new_direction;
                    return;
                }
            }
            // if tap is in the direction the player is already facing, move in that direction,
            // or, if the key has been pressed for long enough, move in that direction
            else if new_direction == *player_direction
                || time_pressed >= PLAYER_MOVEMENT_DELAY_SECONDS
            {
                *player_direction = new_direction;
                *player_state = PlayerState::Moving {
                    destination: match new_direction.try_move_to_point(&player_position.0, floor) {
                        Some(p) => p,
                        None => return,
                    },
                    timer: Timer::from_seconds(PLAYER_MOVING_TIME_SECONDS, false),
                }
            }
        } else {
            return;
        }
    }
}
