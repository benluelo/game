use bevy::prelude::*;
use dungeon::{DungeonTile, Point};

use crate::{
    constants::{PLAYER_Z_INDEX, SPRITE_SIZE, TILE_Z_INDEX},
    Materials,
};

pub fn tile_sprite_bundle(
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

pub fn player_sprite_bundle(
    materials: &Res<Materials>,
    point: Point,
    floor: &dungeon::Floor,
) -> SpriteBundle {
    SpriteBundle {
        material: materials.player_material.clone(),
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

pub fn point_to_transform(point: Point, floor: &dungeon::Floor, z_index: f32) -> Transform {
    Transform::from_xyz(
        point.column.get().as_unbounded() as f32 * SPRITE_SIZE,
        (floor.height.as_unbounded() as f32 - point.row.get().as_unbounded() as f32) * SPRITE_SIZE,
        z_index,
    )
}
