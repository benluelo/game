use criterion::{criterion_group, criterion_main, Criterion};
use game::{DungeonTile, Point};

#[allow(unused_variables)]
fn bench_get_adjacent_walls(c: &mut Criterion) {
    let height = 10;
    let width = 10;
    let mut fb = FloorBuilder::<Blank>::blank(height, width);
    fb.map = MAP.to_vec();
    c.bench_function("adjacent walls", |b| {
        for x in (0..width).step_by(2) {
            for y in (0..height).step_by(2) {
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

#[allow(dead_code)]
const MAP: [DungeonTile; 100] = [
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
];
