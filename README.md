# game (TODO: Name this game)

This is the monorepo for my (currently unnamed) game.

## Prerequisites

Everything is developed and tested on Ubuntu 20.04 LTS. Any OS supported by Rust (and bevy) should work fine, although they are untested. If you have difficulties getting anything to run, please open an issue.

Bevy dependencies for linux:
<https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md>

## Crates

### `bounded_int`

Utility crate. Leverages const generics to enforce compile time bounds on integers.

**Note:** due to current limitations in Rust, the `bounded_int` type uses an `i32` behind the scenes. This could be wasteful if you limit the bounds to smaller than, say, an `i16`, or may not be enough if you need a number larger than an `i32` can hold.

### `dungeon`

Dungeon creation. Creates 2d cave-like dungeons using cellular automata (among other techniques).

### `frontend`

The main frontend binary. Will most likely be seperated out into multiple crates at some point, with this binary consuming all of them.
