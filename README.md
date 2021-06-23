# A backend for my games

Writing top level headings is hard.

## Modules

### `bounded_int`

Utility module. Leverages const generics to enforce compile time bounds on integers.

**Note:** due to current limitations in Rust, the `bounded_int` type uses an `i32` behind the scenes. This could be wasteful if you limit the bounds to smaller than, say, an `i16`, or may not be enough if you need a number larger than an `i32` can hold.

### `dungeon`

Dungeon creation. Creates 2d cave-like dungeons using cellular automata (among other techniques).
