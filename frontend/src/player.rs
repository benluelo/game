use std::f32::consts::PI;

use bevy::prelude::*;
use dungeon::{Floor, Point};

pub struct Player;

#[derive(Debug)]
pub enum PlayerState {
    Moving { destination: Point, timer: Timer },
    Still,
}

// REVIEW: Maybe make this a generic `Direction`, not tied to the player specifically?
//         It would make the code a bit more DRY, but might mess with parallelization
//         within bevy, since we need mutable access to the `PlayerDirection` every
//         frame LINK frontend/src/key_press_handling.rs#key_press_handling
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PlayerDirection {
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
    pub fn to_rotation(&self) -> Quat {
        Quat::from_rotation_z(match self {
            PlayerDirection::Up => 0.0,
            PlayerDirection::Right => 3.0 * PI / 2.0,
            PlayerDirection::Down => PI,
            PlayerDirection::Left => PI / 2.0,
        })
    }

    /// Tries to move to the point provided, in the direction of `self`.
    pub fn try_move_to_point(&self, from: &Point, floor: &Floor) -> Option<Point> {
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
