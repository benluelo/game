pub mod constants;

use constants::{SPRITE_SIZE, TILE_Z_INDEX};
use num_traits::identities::Zero;
use std::f32::consts::PI;
use std::ops::Index;
use std::{convert::TryInto, num::NonZeroU16};

use bevy::utils::{StableHashMap, StableHashSet};
use bevy::{prelude::*, render::camera::Camera};
use dungeon::{Column, Dungeon, DungeonTile, DungeonType, Floor, Point, PointIndex, Row};

use crate::constants::{PLAYER_MOVEMENT_DELAY_SECONDS, PLAYER_Z_INDEX};

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "game".to_string(),
            width: 500.0,
            height: 500.0,
            ..Default::default()
        })
        .insert_resource(PlayerState::Still)
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(KeyPressTime(
            [
                // (KeyCode::Key1, 0.0f32),
                // (KeyCode::Key2, 0.0),
                // (KeyCode::Key3, 0.0),
                // (KeyCode::Key4, 0.0),
                // (KeyCode::Key5, 0.0),
                // (KeyCode::Key6, 0.0),
                // (KeyCode::Key7, 0.0),
                // (KeyCode::Key8, 0.0),
                // (KeyCode::Key9, 0.0),
                // (KeyCode::Key0, 0.0),
                // (KeyCode::A, 0.0),
                // (KeyCode::B, 0.0),
                // (KeyCode::C, 0.0),
                // (KeyCode::D, 0.0),
                // (KeyCode::E, 0.0),
                // (KeyCode::F, 0.0),
                // (KeyCode::G, 0.0),
                // (KeyCode::H, 0.0),
                // (KeyCode::I, 0.0),
                // (KeyCode::J, 0.0),
                // (KeyCode::K, 0.0),
                // (KeyCode::L, 0.0),
                // (KeyCode::M, 0.0),
                // (KeyCode::N, 0.0),
                // (KeyCode::O, 0.0),
                // (KeyCode::P, 0.0),
                // (KeyCode::Q, 0.0),
                // (KeyCode::R, 0.0),
                // (KeyCode::S, 0.0),
                // (KeyCode::T, 0.0),
                // (KeyCode::U, 0.0),
                // (KeyCode::V, 0.0),
                // (KeyCode::W, 0.0),
                // (KeyCode::X, 0.0),
                // (KeyCode::Y, 0.0),
                // (KeyCode::Z, 0.0),
                // (KeyCode::Escape, 0.0),
                // (KeyCode::F1, 0.0),
                // (KeyCode::F2, 0.0),
                // (KeyCode::F3, 0.0),
                // (KeyCode::F4, 0.0),
                // (KeyCode::F5, 0.0),
                // (KeyCode::F6, 0.0),
                // (KeyCode::F7, 0.0),
                // (KeyCode::F8, 0.0),
                // (KeyCode::F9, 0.0),
                // (KeyCode::F10, 0.0),
                // (KeyCode::F11, 0.0),
                // (KeyCode::F12, 0.0),
                // (KeyCode::F13, 0.0),
                // (KeyCode::F14, 0.0),
                // (KeyCode::F15, 0.0),
                // (KeyCode::F16, 0.0),
                // (KeyCode::F17, 0.0),
                // (KeyCode::F18, 0.0),
                // (KeyCode::F19, 0.0),
                // (KeyCode::F20, 0.0),
                // (KeyCode::F21, 0.0),
                // (KeyCode::F22, 0.0),
                // (KeyCode::F23, 0.0),
                // (KeyCode::F24, 0.0),
                // (KeyCode::Snapshot, 0.0),
                // (KeyCode::Scroll, 0.0),
                // (KeyCode::Pause, 0.0),
                // (KeyCode::Insert, 0.0),
                // (KeyCode::Home, 0.0),
                // (KeyCode::Delete, 0.0),
                // (KeyCode::End, 0.0),
                // (KeyCode::PageDown, 0.0),
                // (KeyCode::PageUp, 0.0),
                // (KeyCode::Left, 0.0),
                // (KeyCode::Up, 0.0),
                // (KeyCode::Right, 0.0),
                // (KeyCode::Down, 0.0),
                // (KeyCode::Back, 0.0),
                // (KeyCode::Return, 0.0),
                // (KeyCode::Space, 0.0),
                // (KeyCode::Compose, 0.0),
                // (KeyCode::Caret, 0.0),
                // (KeyCode::Numlock, 0.0),
                // (KeyCode::Numpad0, 0.0),
                // (KeyCode::Numpad1, 0.0),
                // (KeyCode::Numpad2, 0.0),
                // (KeyCode::Numpad3, 0.0),
                // (KeyCode::Numpad4, 0.0),
                // (KeyCode::Numpad5, 0.0),
                // (KeyCode::Numpad6, 0.0),
                // (KeyCode::Numpad7, 0.0),
                // (KeyCode::Numpad8, 0.0),
                // (KeyCode::Numpad9, 0.0),
                // (KeyCode::AbntC1, 0.0),
                // (KeyCode::AbntC2, 0.0),
                // (KeyCode::NumpadAdd, 0.0),
                // (KeyCode::Apostrophe, 0.0),
                // (KeyCode::Apps, 0.0),
                // (KeyCode::Asterisk, 0.0),
                // (KeyCode::Plus, 0.0),
                // (KeyCode::At, 0.0),
                // (KeyCode::Ax, 0.0),
                // (KeyCode::Backslash, 0.0),
                // (KeyCode::Calculator, 0.0),
                // (KeyCode::Capital, 0.0),
                // (KeyCode::Colon, 0.0),
                // (KeyCode::Comma, 0.0),
                // (KeyCode::Convert, 0.0),
                // (KeyCode::NumpadDecimal, 0.0),
                // (KeyCode::NumpadDivide, 0.0),
                // (KeyCode::Equals, 0.0),
                // (KeyCode::Grave, 0.0),
                // (KeyCode::Kana, 0.0),
                // (KeyCode::Kanji, 0.0),
                // (KeyCode::LAlt, 0.0),
                // (KeyCode::LBracket, 0.0),
                // (KeyCode::LControl, 0.0),
                // (KeyCode::LShift, 0.0),
                // (KeyCode::LWin, 0.0),
                // (KeyCode::Mail, 0.0),
                // (KeyCode::MediaSelect, 0.0),
                // (KeyCode::MediaStop, 0.0),
                // (KeyCode::Minus, 0.0),
                // (KeyCode::NumpadMultiply, 0.0),
                // (KeyCode::Mute, 0.0),
                // (KeyCode::MyComputer, 0.0),
                // (KeyCode::NavigateForward, 0.0),
                // (KeyCode::NavigateBackward, 0.0),
                // (KeyCode::NextTrack, 0.0),
                // (KeyCode::NoConvert, 0.0),
                // (KeyCode::NumpadComma, 0.0),
                // (KeyCode::NumpadEnter, 0.0),
                // (KeyCode::NumpadEquals, 0.0),
                // (KeyCode::Oem102, 0.0),
                // (KeyCode::Period, 0.0),
                // (KeyCode::PlayPause, 0.0),
                // (KeyCode::Power, 0.0),
                // (KeyCode::PrevTrack, 0.0),
                // (KeyCode::RAlt, 0.0),
                // (KeyCode::RBracket, 0.0),
                // (KeyCode::RControl, 0.0),
                // (KeyCode::RShift, 0.0),
                // (KeyCode::RWin, 0.0),
                // (KeyCode::Semicolon, 0.0),
                // (KeyCode::Slash, 0.0),
                // (KeyCode::Sleep, 0.0),
                // (KeyCode::Stop, 0.0),
                // (KeyCode::NumpadSubtract, 0.0),
                // (KeyCode::Sysrq, 0.0),
                // (KeyCode::Tab, 0.0),
                // (KeyCode::Underline, 0.0),
                // (KeyCode::Unlabeled, 0.0),
                // (KeyCode::VolumeDown, 0.0),
                // (KeyCode::VolumeUp, 0.0),
                // (KeyCode::Wake, 0.0),
                // (KeyCode::WebBack, 0.0),
                // (KeyCode::WebFavorites, 0.0),
                // (KeyCode::WebForward, 0.0),
                // (KeyCode::WebHome, 0.0),
                // (KeyCode::WebRefresh, 0.0),
                // (KeyCode::WebSearch, 0.0),
                // (KeyCode::WebStop, 0.0),
                // (KeyCode::Yen, 0.0),
                // (KeyCode::Copy, 0.0),
                // (KeyCode::Paste, 0.0),
                // (KeyCode::Cut, 0.0),
            ]
            .into_iter()
            .cloned()
            .collect(),
        ))
        .add_startup_system(setup.system())
        .add_startup_stage(
            "game setup",
            SystemStage::single(spawn_player_and_board.system()),
        )
        .add_plugins(DefaultPlugins)
        .add_system(player_movement_input_handling.system())
        .add_system(camera_player_tracking.system())
        .add_system(smooth_player_movement.system())
        .add_system(key_press_handling.system())
        // .add_system_set_to_stage(
        //     CoreStage::PostUpdate,
        //     SystemSet::new()
        //         .with_system(position_translation.system())
        //         .with_system(size_scaling.system()),
        // )
        .run();
}

#[derive(Debug)]
pub enum PlayerState {
    Moving { destination: Point, timer: Timer },
    Still,
}

fn setup(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    server.watch_for_changes().unwrap();
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.insert_resource(Dungeon::new(
        80_i32.try_into().unwrap(),
        50_i32.try_into().unwrap(),
        NonZeroU16::new(1).unwrap(),
        DungeonType::Cave,
        false,
    ));
    commands.insert_resource(Materials {
        empty_material: materials.add(
            /* Color::WHITE.into() */ server.load("empty.png").into(),
        ),
        wall_material: materials.add(Color::BLACK.into()),
        secret_door_material: materials.add(Color::RED.into()),
        secret_passage_material: materials.add(Color::LIME_GREEN.into()),
        treasure_chest_material: materials.add(Color::BLUE.into()),
        entrance_material: materials.add(Color::PINK.into()),
        exit_material: materials.add(Color::PURPLE.into()),
        player_material: materials.add(server.load("arrow.png").into()),
    })
}

pub struct Tile {
    tile_type: DungeonTile,
}

// REVIEW: Maybe make this a generic `Direction`, not tied to the player specifically?
//         It would make the code a bit more DRY, but might mess with parallelization
//         within bevy, since we need mutable access to the `PlayerDirection` every
//         frame LINK bevy/src/main.rs#key_press_handling
#[derive(Debug, PartialEq, Clone, Copy)]
enum PlayerDirection {
    Up,
    Left,
    Down,
    Right,
}

impl PlayerDirection {
    /// Returns the respective [`Quat`] for the [`PlayerDirection`], rotated
    /// along the z axis. Used for changing which direction the player is
    /// facing.
    ///
    /// ```no-run
    ///            0
    ///            ^
    ///            UP
    /// π/2 < LEFT    RIGHT > 3π/2
    ///           DOWN
    ///            v
    ///            π
    /// ```
    fn to_rotation(&self) -> Quat {
        Quat::from_rotation_z(match self {
            PlayerDirection::Up => 0.0,
            PlayerDirection::Right => 3.0 * PI / 2.0,
            PlayerDirection::Down => PI,
            PlayerDirection::Left => PI / 2.0,
        })
    }

    fn move_player(&self, from: &Point, floor: &Floor) -> Option<Point> {
        let new_point = match self {
            PlayerDirection::Up => from.sub_row(1).ok()?,
            PlayerDirection::Right => from.add_column(1).ok()?,
            PlayerDirection::Down => from.add_row(1).ok()?,
            PlayerDirection::Left => from.sub_column(1).ok()?,
        };
        if floor.at(new_point).is_wall() {
            None
        } else {
            Some(new_point)
        }
    }
}

pub struct Player;

struct Materials {
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

#[derive(Debug)]
struct KeyPressTime(StableHashMap<KeyCode, f32>);

// ANCHOR[id=key_press_handling]
fn key_press_handling(
    mut key_press_time: ResMut<KeyPressTime>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let pressed = keyboard_input.get_pressed().collect::<StableHashSet<_>>();
    // dbg!(&pressed);
    let newly_pressed_keys = keyboard_input
        .get_pressed()
        .filter(|x| !key_press_time.0.contains_key(*x))
        .map(|x| (*x, f32::zero()));

    // key_press_time.0.extend(newly_pressed_keys);

    key_press_time.0 = key_press_time
        .0
        .iter()
        .filter_map(|(k, v)| {
            if
            /* *v > 0.0 &&  */
            pressed.contains(k) {
                Some((*k, *v + time.delta_seconds()))
            } else {
                None
            }
        })
        .chain(newly_pressed_keys)
        .collect();
    dbg!(&*key_press_time);
}

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
            .spawn_bundle(tile_sprite_bundle(&materials, tile, point, floor))
            .insert(Tile { tile_type: *tile })
            .insert(Position(point));
    }
}

fn tile_sprite_bundle(
    materials: &Res<Materials>,
    tile: &DungeonTile,
    point: Point,
    floor: &dungeon::Floor,
) -> SpriteBundle {
    SpriteBundle {
        material: materials[*tile].clone(),
        sprite: Sprite::new(Vec2::new(SPRITE_SIZE, SPRITE_SIZE)),
        transform: point_to_transform(point, floor, TILE_Z_INDEX),
        ..Default::default()
    }
}

fn player_sprite_bundle(
    materials: &Res<Materials>,
    point: Point,
    floor: &dungeon::Floor,
) -> SpriteBundle {
    SpriteBundle {
        material: dbg!(materials.player_material.clone()),
        sprite: Sprite::new(Vec2::new(SPRITE_SIZE, SPRITE_SIZE)),
        transform: Transform::from_xyz(
            point.column.get().as_unbounded() as f32 * SPRITE_SIZE,
            (floor.height.as_unbounded() as f32 - point.row.get().as_unbounded() as f32)
                * SPRITE_SIZE,
            PLAYER_Z_INDEX,
        ),
        ..Default::default()
    }
}

fn point_to_transform(point: Point, floor: &dungeon::Floor, z_index: f32) -> Transform {
    Transform::from_xyz(
        point.column.get().as_unbounded() as f32 * SPRITE_SIZE,
        (floor.height.as_unbounded() as f32 - point.row.get().as_unbounded() as f32) * SPRITE_SIZE,
        z_index,
    )
}

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
fn camera_player_tracking(
    mut set: QuerySet<(
        Query<&Transform, With<Player>>,
        Query<&mut Transform, With<Camera>>,
    )>,
) {
    set.q1_mut().single_mut().unwrap().translation = set.q0().single().unwrap().translation;
}

/// Moves the player by changing it's internal [`dungeon::Point`] according to the input.
fn player_movement_input_handling(
    key_press_time: ResMut<KeyPressTime>,
    keyboard_input: Res<Input<KeyCode>>,
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
                    destination: if let Some(p) =
                        new_direction.move_player(&player_position.0, floor)
                    {
                        p
                    } else {
                        return;
                    },
                    timer: Timer::from_seconds(0.2, false),
                }
            }
        } else {
            return;
        }
    }
}
