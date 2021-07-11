use std::ops::Index;
use std::{convert::TryInto, num::NonZeroU16};

use bevy::core::FixedTimestep;
use bevy::{prelude::*, render::camera::Camera};
use dungeon::{Column, Dungeon, DungeonTile, DungeonType, Point, PointIndex, Row};

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "game".to_string(),
            width: 500.0,
            height: 500.0,
            ..Default::default()
        })
        .insert_resource(PlayerState::Still)
        .add_startup_system(setup.system())
        .add_startup_stage(
            "game setup",
            SystemStage::single(spawn_player_and_board.system()),
        )
        .add_plugins(DefaultPlugins)
        .add_system(player_movement_input_handling.system())
        .add_system(camera_player_tracking.system())
        .add_system(smooth_player_movement.system())
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

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.insert_resource(Dungeon::new(
        80_i32.try_into().unwrap(),
        50_i32.try_into().unwrap(),
        NonZeroU16::new(1).unwrap(),
        DungeonType::Cave,
        false,
    ));
    commands.insert_resource(Materials {
        empty_material: materials.add(Color::WHITE.into()),
        wall_material: materials.add(Color::BLACK.into()),
        secret_door_material: materials.add(Color::RED.into()),
        secret_passage_material: materials.add(Color::LIME_GREEN.into()),
        treasure_chest_material: materials.add(Color::BLUE.into()),
        entrance_material: materials.add(Color::PINK.into()),
        exit_material: materials.add(Color::PURPLE.into()),
        player_material: materials.add(Color::SILVER.into()),
    })
}

pub struct Tile {
    tile_type: DungeonTile,
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

pub struct Position(Point);

const SPRITE_SIZE: f32 = 5.0;

fn spawn_player_and_board(
    mut commands: Commands,
    dungeon: Res<Dungeon>,
    materials: Res<Materials>,
    mut camera: Query<&mut Transform, With<Camera>>,
) {
    let floor = dungeon.floors.first().unwrap();

    for column in floor
        .width
        .expand_lower()
        .range_from(&0.try_into().unwrap())
    {
        for row in floor
            .height
            .expand_lower()
            .range_from(&0.try_into().unwrap())
        {
            let point = Point {
                column: Column::new(column),
                row: Row::new(row),
            };

            let tile = floor.data.at(point, floor.width);

            if matches!(tile, DungeonTile::Entrance) {
                commands
                    .spawn_bundle(SpriteBundle {
                        material: materials.player_material.clone(),
                        sprite: Sprite::new(Vec2::new(SPRITE_SIZE, SPRITE_SIZE)),
                        transform: Transform::from_xyz(
                            point.column.get().as_unbounded() as f32 * SPRITE_SIZE,
                            (floor.height.as_unbounded() as f32
                                - point.row.get().as_unbounded() as f32)
                                * SPRITE_SIZE,
                            20.0,
                        ),
                        ..Default::default()
                    })
                    .insert(Player)
                    .insert(Position(point));

                for mut transform in camera.iter_mut() {
                    transform.translation.x =
                        point.column.get().as_unbounded() as f32 * SPRITE_SIZE;
                    transform.translation.y = (floor.height.as_unbounded() as f32
                        - point.row.get().as_unbounded() as f32)
                        * SPRITE_SIZE;
                }
            }

            commands
                .spawn_bundle(SpriteBundle {
                    material: materials[*tile].clone(),
                    sprite: Sprite::new(Vec2::new(SPRITE_SIZE, SPRITE_SIZE)),
                    transform: point_to_transform(point, floor),
                    ..Default::default()
                })
                .insert(Tile { tile_type: *tile })
                .insert(Position(point));
        }
    }
}

fn point_to_transform(point: Point, floor: &dungeon::Floor) -> Transform {
    Transform::from_xyz(
        point.column.get().as_unbounded() as f32 * SPRITE_SIZE,
        (floor.height.as_unbounded() as f32 - point.row.get().as_unbounded() as f32) * SPRITE_SIZE,
        10.0,
    )
}

fn smooth_player_movement(
    time: Res<Time>,
    dungeon: Res<Dungeon>,
    mut player_state: ResMut<PlayerState>,
    mut player_position: Query<&mut Position, With<Player>>,
    mut player_transform: Query<&mut Transform, With<Player>>,
) {
    let floor = dungeon.floors.first().unwrap();
    let mut position = match player_position.single_mut() {
        Ok(it) => it,
        _ => return,
    };

    let mut transform = match player_transform.single_mut() {
        Ok(it) => it,
        _ => return,
    };

    let mut done_moving = false;
    // dbg!(&*player_state);
    match *player_state {
        PlayerState::Moving {
            destination,
            ref mut timer,
        } => {
            timer.tick(time.delta());

            transform.translation = point_to_transform(position.0, floor).translation.lerp(
                point_to_transform(destination, floor).translation,
                timer.percent(),
            );

            if timer.finished() {
                done_moving = true;
                position.0 = destination;
                println!("timer finished");
            }
            // dbg!(timer);
        }
        PlayerState::Still => return,
    }

    if done_moving {
        println!("set player to still");
        *player_state = PlayerState::Still;
    }
    dbg!(&*player_state);
}

fn camera_player_tracking(
    mut set: QuerySet<(
        Query<&Transform, With<Player>>,
        Query<&mut Transform, With<Camera>>,
    )>,
) {
    set.q1_mut().single_mut().unwrap().translation = set.q0().single().unwrap().translation;
}

fn distance(t1: &Transform, t2: &Transform) -> f32 {
    ((t1.translation.x - t2.translation.x).powi(2) + (t1.translation.y - t2.translation.y).powi(2))
        .sqrt()
}

fn player_movement_input_handling(
    keyboard_input: Res<Input<KeyCode>>,
    dungeon: Res<Dungeon>,
    mut player_state: ResMut<PlayerState>,
    player_position: Query<&Position, With<Player>>,
    mut transforms: Query<&mut Transform, Or<(With<Camera>, With<Player>)>>,
) {
    let floor = &dungeon.floors[0];

    if let Ok(position) = player_position.single() {
        if matches!(*player_state, PlayerState::Moving { .. }) {
            return;
        };

        let mut new_pos = Position { ..*position };
        if keyboard_input.pressed(KeyCode::Left) {
            new_pos.0.column = if let Ok(col) = position.0.column - 1 {
                println!("key left");
                if col.get() < floor.width.expand_lower() {
                    col
                } else {
                    return;
                }
            } else {
                return;
            };
        } else if keyboard_input.pressed(KeyCode::Right) {
            new_pos.0.column = if let Ok(col) = position.0.column + 1 {
                if col.get() < floor.width.expand_lower() {
                    col
                } else {
                    return;
                }
            } else {
                return;
            };
        } else if keyboard_input.pressed(KeyCode::Down) {
            new_pos.0.row = if let Ok(row) = position.0.row + 1 {
                if row.get() < floor.height.expand_lower() {
                    row
                } else {
                    return;
                }
            } else {
                return;
            };
        } else if keyboard_input.pressed(KeyCode::Up) {
            new_pos.0.row = if let Ok(row) = position.0.row - 1 {
                if row.get() < floor.height.expand_lower() {
                    row
                } else {
                    return;
                }
            } else {
                return;
            };
        }

        if new_pos.0 == position.0 {
            return;
        }

        if floor.data.at(new_pos.0, floor.width).is_wall() {
            return;
        } else {
            *player_state = PlayerState::Moving {
                destination: new_pos.0,
                timer: Timer::from_seconds(0.2, false),
            };
        }

        // for mut transform in transforms.iter_mut() {
        //     if keyboard_input.pressed(KeyCode::Left) {
        //         transform.translation.x -= SPRITE_SIZE;
        //     }
        //     if keyboard_input.pressed(KeyCode::Right) {
        //         transform.translation.x += SPRITE_SIZE;
        //     }
        //     if keyboard_input.pressed(KeyCode::Down) {
        //         transform.translation.y -= SPRITE_SIZE;
        //     }
        //     if keyboard_input.pressed(KeyCode::Up) {
        //         transform.translation.y += SPRITE_SIZE;
        //     }
        // }
    }
}
