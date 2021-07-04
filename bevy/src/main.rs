use std::{convert::TryInto, num::NonZeroU16, ops::Index};

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
        .add_startup_system(setup.system())
        .add_startup_stage(
            "game setup",
            SystemStage::single(spawn_player_and_board.system()),
        )
        .add_plugins(DefaultPlugins)
        .add_system(snake_movement.system())
        // .add_system_set_to_stage(
        //     CoreStage::PostUpdate,
        //     SystemSet::new()
        //         .with_system(position_translation.system())
        //         .with_system(size_scaling.system()),
        // )
        .run();
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.insert_resource(Dungeon::new(
        80.try_into().unwrap(),
        80.try_into().unwrap(),
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

const SPRITE_SIZE: f32 = 50.0;

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
                            point.row.get().as_unbounded() as f32 * SPRITE_SIZE,
                            20.0,
                        ),
                        ..Default::default()
                    })
                    .insert(Player)
                    .insert(Position(point));

                for mut transform in camera.iter_mut() {
                    transform.translation.x =
                        point.column.get().as_unbounded() as f32 * SPRITE_SIZE;
                    transform.translation.y = point.row.get().as_unbounded() as f32 * SPRITE_SIZE;
                    println!("found camera");
                }
            }

            commands
                .spawn_bundle(SpriteBundle {
                    material: materials[*tile].clone(),
                    sprite: Sprite::new(Vec2::new(SPRITE_SIZE, SPRITE_SIZE)),
                    transform: Transform::from_xyz(
                        point.column.get().as_unbounded() as f32 * SPRITE_SIZE,
                        point.row.get().as_unbounded() as f32 * SPRITE_SIZE,
                        10.0,
                    ),
                    ..Default::default()
                })
                .insert(Tile { tile_type: *tile })
                .insert(Position(point));
        }
    }
}

fn snake_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut set: QuerySet<(
        Query<(&mut Transform, &mut Position), With<Player>>,
        Query<&mut Transform, With<Camera>>,
    )>,
) {
    let (mut player_transform, mut player_position) = set.q0_mut().single_mut().unwrap();
    let mut camera_transform = set.q1_mut().single_mut().unwrap();

    if keyboard_input.pressed(KeyCode::Left) {
        transform.translation.x -= SPRITE_SIZE;
    }
    if keyboard_input.pressed(KeyCode::Right) {
        transform.translation.x += SPRITE_SIZE;
    }
    if keyboard_input.pressed(KeyCode::Down) {
        transform.translation.y -= SPRITE_SIZE;
    }
    if keyboard_input.pressed(KeyCode::Up) {
        transform.translation.y += SPRITE_SIZE;
    }
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(
                pos.0.column.get().as_unbounded() as f32,
                window.width() as f32,
                ARENA_WIDTH as f32,
            ),
            convert(
                pos.0.row.get().as_unbounded() as f32,
                window.height() as f32,
                ARENA_HEIGHT as f32,
            ),
            0.0,
        );
    }
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Sprite)>) {
    let window = windows.get_primary().unwrap();
    for (sprite_size, mut sprite) in q.iter_mut() {
        sprite.size = Vec2::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width() as f32,
            sprite_size.height / ARENA_HEIGHT as f32 * window.height() as f32,
        );
    }
}

const ARENA_WIDTH: i32 = 10;
const ARENA_HEIGHT: i32 = 10;
