/// The z-index of the player, this should be higher than most
/// other sprites, if not all.
pub const PLAYER_Z_INDEX: f32 = 20.0;

/// The z-index of the background tiles.
pub const TILE_Z_INDEX: f32 = 10.0;

/// The size of the sprites on the screen.
///
/// REVIEW: What unit is this measured in? Pixels?
///
/// FIXME: This shouldn't really be a constant, but it's the only
///        way I could get the sprites to display properly.
///
/// TODO: Generate this based on the window size and however many
///       tiles I decide to have on screen (i.e. zoom level)
pub const SPRITE_SIZE: f32 = 15.0;

/// The delay, in seconds, between when the user presses an arrow
/// key and when the player moves. Allows for tapping the keys to
/// change the direction of the player without moving.
/// 
/// TODO: If the player is facing the direction tapped, move that
///       direction regardless of how long the key is pressed
pub const PLAYER_MOVEMENT_DELAY_SECONDS: f32 = 0.05;
