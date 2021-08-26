use std::{convert::TryInto, num::NonZeroU16, ops::Add};

use bevy::{prelude::*, render::camera::Camera};
use dungeon::{Dungeon, DungeonTile, DungeonType, Point};

use crate::{
    constants::{PLAYER_MOVEMENT_DELAY_SECONDS, PLAYER_MOVING_TIME_SECONDS, TILE_Z_INDEX},
    key_press_handling::KeyPressTime,
    player::{Player, PlayerDirection, PlayerState},
    utils::{player_sprite_bundle, point_to_transform},
    Materials,
};

#[derive(StageLabel, SystemLabel, Clone, Copy, Hash, Debug, PartialEq, Eq)]
enum StageLabel {
    SpawnPlayerAndBoard,
    PlayerMovementInputHandling,
    SmoothPlayerMovement,
    CameraPlayerTracking,
}

pub struct CurrentFloor(pub NonZeroU16);

pub struct ExitFloor;

pub struct DungeonPlugin;

impl Plugin for DungeonPlugin {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        app.insert_resource(Dungeon::new(
            80_i32.try_into().unwrap(),
            50_i32.try_into().unwrap(),
            NonZeroU16::new(10).unwrap(),
            DungeonType::Cave,
            false,
        ))
        .add_event::<ExitFloor>()
        .insert_resource(CurrentFloor(NonZeroU16::new(1).unwrap()))
        // .insert_resource(Msaa { samples: 4 })
        .add_startup_stage_after(
            StartupStage::Startup,
            StageLabel::SpawnPlayerAndBoard,
            SystemStage::single(spawn_player_and_board.system()),
        )
        // .add_startup_system(spawn_player_and_board.system().after("setup"))
        .add_system_set(
            SystemSet::new()
                .with_system(
                    player_movement_input_handling
                        .system()
                        .label(StageLabel::PlayerMovementInputHandling),
                )
                .with_system(
                    smooth_player_movement
                        .system()
                        .label(StageLabel::SmoothPlayerMovement)
                        .after(StageLabel::PlayerMovementInputHandling),
                )
                .with_system(
                    camera_player_tracking
                        .exclusive_system()
                        .at_end()
                        .label(StageLabel::CameraPlayerTracking)
                        .after(StageLabel::SmoothPlayerMovement),
                ),
        );
        // .add_system_set_to_stage(
        //     CoreStage::PostUpdate,
        //     SystemSet::new()
        //         .with_system(position_translation.system())
        //         .with_system(size_scaling.system()),
        // )
    }
}

// fn receiver(mut reader: EventReader<ExitFloor>) {
//     for event in reader.iter() {
//         // handle event
//     }
// }

pub struct Tile {
    #[allow(dead_code)]
    tile_type: DungeonTile,
}

/// The internal position of something in the world.
pub struct Position(Point);

fn spawn_player_and_board(
    mut commands: Commands,
    dungeon: Res<Dungeon>,
    materials: Res<Materials>,
    current_floor: Res<CurrentFloor>,
) {
    let floor = dungeon.floors.get(current_floor.0.get() as usize).unwrap();

    for (point, tile) in floor.iter_points_and_tiles() {
        if matches!(tile, DungeonTile::Entrance) {
            commands
                .spawn_bundle(player_sprite_bundle(&materials, point, floor))
                .insert(Player)
                .insert(PlayerDirection::Up)
                .insert(PlayerState::Still)
                .insert(Position(point));
        }

        commands
            .spawn_bundle(crate::utils::tile_sprite_bundle(
                &materials, tile, point, floor,
            ))
            .insert(Tile { tile_type: *tile })
            .insert(Position(point));
    }
}

/// Moves the player between squares smoothly.
fn smooth_player_movement(
    time: Res<Time>,
    dungeon: Res<Dungeon>,
    player_direction: Query<&PlayerDirection, With<Player>>,
    mut current_floor: ResMut<CurrentFloor>,
    mut player_state: Query<&mut PlayerState, With<Player>>,
    mut player_position: Query<&mut Position, With<Player>>,
    mut player_transform: Query<&mut Transform, With<Player>>,
) {
    let floor = dungeon.floors.get(current_floor.0.get() as usize).unwrap();
    let mut position = player_position.single_mut().unwrap();

    let mut transform = player_transform.single_mut().unwrap();

    transform.rotation = player_direction.single().unwrap().to_rotation();

    let mut done_moving = false;
    // dbg!(&*player_state);
    let mut player_state = player_state.single_mut().unwrap();

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

                if floor.at(position.0).is_exit() {
                    current_floor.0 = NonZeroU16::new(current_floor.0.get().add(1)).unwrap()
                }
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
    mut player_state: Query<&mut PlayerState, With<Player>>,
    player_position: Query<&Position, With<Player>>,
    mut player_direction: Query<&mut PlayerDirection, With<Player>>,
    current_floor: Res<CurrentFloor>,
) {
    let floor = dungeon.floors.get(current_floor.0.get() as usize).unwrap();

    let mut player_state = player_state.single_mut().unwrap();

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
