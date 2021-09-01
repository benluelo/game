use std::{
    convert::TryInto,
    num::NonZeroU16,
    ops::{Add, Sub},
};

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
enum DungeonPluginStageLabel {
    SpawnPlayerAndBoard,
    PlayerDirectionHandling,
    PlayerMovementInputHandling,
    SmoothPlayerMovement,
    CameraPlayerTracking,
}

#[derive(Debug, Clone)]
pub struct CurrentFloor(u16);

impl CurrentFloor {
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

pub struct DungeonPlugin;

#[derive(Debug, Clone)]
pub enum FloorChangedEvent {
    Up { previous_position: Position },
    Down { previous_position: Position },
}

impl FloorChangedEvent {
    fn _previous_position(&self) -> Position {
        match self {
            FloorChangedEvent::Up { previous_position } => *previous_position,
            FloorChangedEvent::Down { previous_position } => *previous_position,
        }
    }
}

impl Plugin for DungeonPlugin {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        use DungeonPluginStageLabel::*;

        app.insert_resource(Dungeon::new(
            80_i32.try_into().unwrap(),
            50_i32.try_into().unwrap(),
            NonZeroU16::new(10).unwrap(),
            DungeonType::Cave,
            false,
        ))
        .insert_resource(CurrentFloor(0))
        // .insert_resource(Msaa { samples: 4 })
        .add_startup_stage_after(
            StartupStage::Startup,
            DungeonPluginStageLabel::SpawnPlayerAndBoard,
            SystemStage::single(spawn_player_and_board.system()),
        )
        .add_event::<FloorChangedEvent>()
        // .add_startup_system(spawn_player_and_board.system().after("setup"))
        .add_system_set(
            SystemSet::new()
                .with_system(
                    player_movement_input_handling
                        .system()
                        .label(PlayerMovementInputHandling),
                )
                .with_system(
                    player_direction_handling
                        .system()
                        .label(PlayerDirectionHandling)
                        .after(PlayerMovementInputHandling),
                )
                .with_system(
                    smooth_player_movement
                        .system()
                        .label(SmoothPlayerMovement)
                        .after(PlayerDirectionHandling),
                )
                .with_system(
                    camera_player_tracking
                        .exclusive_system()
                        .at_end()
                        .label(CameraPlayerTracking)
                        .after(SmoothPlayerMovement),
                ),
        )
        .add_system(floor_changed_event_listener.system())
        .add_system(sync_dungeon_data.system().label("sync_dungeon_data"));
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

#[derive(Debug)]
pub struct Tile {
    #[allow(dead_code)]
    tile_type: DungeonTile,
}

/// The internal position of something in the world.
#[derive(Debug, Clone, Copy)]
pub struct Position(Point);

fn sync_dungeon_data(
    current_floor: Res<CurrentFloor>,
    mut dungeon: ResMut<Dungeon>,
    changes: Query<(&Tile, &Position), Changed<Tile>>,
) {
    for (tile, position) in changes.iter() {
        debug!(?tile.tile_type, "changed tile");
        *dungeon.floors[current_floor.as_usize()].at_mut(position.0) = tile.tile_type
    }
}

fn spawn_player_and_board(
    mut commands: Commands,
    dungeon: Res<Dungeon>,
    materials: Res<Materials>,
    current_floor: Res<CurrentFloor>,
) {
    let floor = dungeon.floors.get(current_floor.as_usize()).unwrap();

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

fn player_direction_handling(
    mut player_transform: Query<&mut Transform, With<Player>>,
    player_direction: Query<&PlayerDirection, With<Player>>,
) {
    let mut player_transform = player_transform.single_mut().unwrap();
    player_transform.rotation = player_direction.single().unwrap().to_rotation();
}

/// Moves the player between squares smoothly.
///
/// Will also a `FloorChangedEvent` upon reaching the entrance or exit.
#[allow(clippy::too_many_arguments)]
fn smooth_player_movement(
    time: Res<Time>,
    dungeon: Res<Dungeon>,
    mut floor_change_event_writer: EventWriter<FloorChangedEvent>,
    current_floor: Res<CurrentFloor>,
    mut player_state: Query<&mut PlayerState, With<Player>>,
    mut player_position: Query<&mut Position, With<Player>>,
    mut player_transform: Query<&mut Transform, With<Player>>,
) {
    let mut mut_player_state = player_state.single_mut().unwrap();

    // println!("{:?}", mut_player_state.clone());

    match *mut_player_state {
        PlayerState::Moving {
            destination,
            ref mut timer,
        } => {
            timer.tick(time.delta());

            let floor = dungeon.floors.get(current_floor.as_usize()).unwrap();

            let mut player_position = player_position.single_mut().unwrap();
            let mut player_transform = player_transform.single_mut().unwrap();

            player_transform.translation =
                point_to_transform(player_position.0, floor, TILE_Z_INDEX)
                    .translation
                    .lerp(
                        point_to_transform(destination, floor, TILE_Z_INDEX).translation,
                        timer.percent(),
                    );

            if timer.finished() {
                let previous_position = *player_position;
                player_position.0 = destination;
                *mut_player_state = PlayerState::Still;
                match floor.at(player_position.0) {
                    DungeonTile::Entrance => {
                        println!("going up a floor, {:?}", *current_floor);
                        floor_change_event_writer.send(FloorChangedEvent::Up { previous_position })
                    }
                    DungeonTile::Exit => {
                        println!("going down a floor, {:?}", *current_floor);
                        floor_change_event_writer
                            .send(FloorChangedEvent::Down { previous_position })
                    }
                    _ => {}
                }
            }
        }
        PlayerState::Still => {}
    }
}

fn floor_changed_event_listener(
    dungeon: Res<Dungeon>,
    mut current_floor: ResMut<CurrentFloor>,
    mut event_listener: EventReader<FloorChangedEvent>,
) {
    // let floor = dungeon.floors.get(current_floor.as_usize()).unwrap();

    for event in event_listener.iter() {
        match event {
            FloorChangedEvent::Up {
                previous_position: _,
            } => {
                if dungeon
                    .floors
                    .get(current_floor.as_usize().sub(1))
                    .is_some()
                {
                    current_floor.0 = current_floor.0.sub(1)
                }
            }
            FloorChangedEvent::Down {
                previous_position: _,
            } => {
                if dungeon
                    .floors
                    .get(current_floor.as_usize().add(1))
                    .is_some()
                {
                    current_floor.0 = current_floor.0.add(1)
                } else {
                    // dungeon over
                }
            }
        }
    }
}

/// makes the camera follow the player.
/// TODO: when the player is in corners, make the camera stay in the same spot(?).
/// NOTE: this will first require fixing how the tiles are drawn
#[allow(clippy::type_complexity)]
fn camera_player_tracking(
    mut set: QuerySet<(
        Query<&Transform, With<Player>>,
        Query<&mut Transform, With<Camera>>,
    )>,
) {
    let player = set.q0().single().unwrap().translation;
    for mut camera in set.q1_mut().iter_mut() {
        camera.translation = player;
    }
}

/// Moves the player by changing it's internal [`dungeon::Point`] according to
/// the input.
fn player_movement_input_handling(
    key_press_time: Res<KeyPressTime>,
    dungeon: Res<Dungeon>,
    mut player_state: Query<&mut PlayerState, With<Player>>,
    player_position: Query<&Position, With<Player>>,
    mut player_direction: Query<&mut PlayerDirection, With<Player>>,
    current_floor: Res<CurrentFloor>,
) {
    let floor = dungeon.floors.get(current_floor.as_usize()).unwrap();

    let mut player_state = player_state.single_mut().unwrap();
    let player_position = player_position.single().unwrap();
    let mut player_direction = player_direction.single_mut().unwrap();

    if matches!(*player_state, PlayerState::Moving { .. }) {
        return;
    };

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
        if new_direction != *player_direction {
            {
                *player_direction = new_direction;
            }
        }
        // if tap is in the direction the player is already facing, move in that direction,
        else if new_direction == *player_direction
            // or, if the key has been pressed for long enough, move in that direction
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
    }
}
