use criterion::{black_box, criterion_group, criterion_main, Criterion, SamplingMode};
use game::dungeon::{Blank, Dungeon, DungeonTile, FloorBuilder, Point};
use std::num::NonZeroUsize;

fn bench_get_adjacent_walls(c: &mut Criterion) {
    let height = NonZeroUsize::new(10).unwrap();
    let width = NonZeroUsize::new(10).unwrap();
    let mut fb = FloorBuilder::<Blank>::blank(height, width);
    fb.map = MAP.iter().map(|i| i.to_vec()).collect();
    c.bench_function("adjacent walls", |b| {
        for x in (0..width.get()).step_by(2) {
            for y in (0..height.get()).step_by(2) {
                b.iter(|| {
                    fb.get_adjacent_walls(Point { x, y }, 1, 1);
                })
            }
        }
    });
}

criterion_group!(
    benches,
    /* bench_dungeon_generation, */ bench_get_adjacent_walls
);
criterion_main!(benches);

const MAP: [[DungeonTile; 10]; 10] = [
    [
        DungeonTile::Wall,
        DungeonTile::Wall,
        DungeonTile::Wall,
        DungeonTile::Wall,
        DungeonTile::Empty,
        DungeonTile::Empty,
        DungeonTile::Wall,
        DungeonTile::Empty,
        DungeonTile::Wall,
        DungeonTile::Wall,
    ],
    [
        DungeonTile::Empty,
        DungeonTile::Wall,
        DungeonTile::Empty,
        DungeonTile::Empty,
        DungeonTile::Wall,
        DungeonTile::Empty,
        DungeonTile::Empty,
        DungeonTile::Empty,
        DungeonTile::Wall,
        DungeonTile::Empty,
    ],
    [
        DungeonTile::Empty,
        DungeonTile::Wall,
        DungeonTile::Empty,
        DungeonTile::Wall,
        DungeonTile::Wall,
        DungeonTile::Empty,
        DungeonTile::Wall,
        DungeonTile::Wall,
        DungeonTile::Wall,
        DungeonTile::Wall,
    ],
    [
        DungeonTile::Wall,
        DungeonTile::Empty,
        DungeonTile::Empty,
        DungeonTile::Wall,
        DungeonTile::Empty,
        DungeonTile::Empty,
        DungeonTile::Wall,
        DungeonTile::Wall,
        DungeonTile::Empty,
        DungeonTile::Wall,
    ],
    [
        DungeonTile::Wall,
        DungeonTile::Empty,
        DungeonTile::Empty,
        DungeonTile::Wall,
        DungeonTile::Wall,
        DungeonTile::Wall,
        DungeonTile::Wall,
        DungeonTile::Empty,
        DungeonTile::Wall,
        DungeonTile::Wall,
    ],
    [
        DungeonTile::Wall,
        DungeonTile::Wall,
        DungeonTile::Empty,
        DungeonTile::Wall,
        DungeonTile::Wall,
        DungeonTile::Empty,
        DungeonTile::Wall,
        DungeonTile::Wall,
        DungeonTile::Empty,
        DungeonTile::Empty,
    ],
    [
        DungeonTile::Empty,
        DungeonTile::Empty,
        DungeonTile::Empty,
        DungeonTile::Empty,
        DungeonTile::Wall,
        DungeonTile::Wall,
        DungeonTile::Empty,
        DungeonTile::Empty,
        DungeonTile::Empty,
        DungeonTile::Empty,
    ],
    [
        DungeonTile::Wall,
        DungeonTile::Wall,
        DungeonTile::Wall,
        DungeonTile::Empty,
        DungeonTile::Empty,
        DungeonTile::Wall,
        DungeonTile::Wall,
        DungeonTile::Empty,
        DungeonTile::Wall,
        DungeonTile::Empty,
    ],
    [
        DungeonTile::Empty,
        DungeonTile::Empty,
        DungeonTile::Empty,
        DungeonTile::Empty,
        DungeonTile::Empty,
        DungeonTile::Empty,
        DungeonTile::Wall,
        DungeonTile::Wall,
        DungeonTile::Empty,
        DungeonTile::Wall,
    ],
    [
        DungeonTile::Empty,
        DungeonTile::Wall,
        DungeonTile::Wall,
        DungeonTile::Empty,
        DungeonTile::Wall,
        DungeonTile::Wall,
        DungeonTile::Wall,
        DungeonTile::Wall,
        DungeonTile::Wall,
        DungeonTile::Empty,
    ],
];
