use criterion::{black_box, criterion_group, criterion_main, Criterion, SamplingMode};
use game::dungeon::{Dungeon, DungeonType};
use std::num::NonZeroUsize;

fn bench_dungeon_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench_dungeon_generation");
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(20);
    group.bench_function("50 by 100", |b| {
        b.iter(|| {
            let _ = Dungeon::new(
                NonZeroUsize::new(black_box(50)).unwrap(),
                NonZeroUsize::new(black_box(100)).unwrap(),
                NonZeroUsize::new(10).unwrap(),
                DungeonType::Cave,
            );
        })
    });
    group.bench_function("100 by 200", |b| {
        b.iter(|| {
            let _ = Dungeon::new(
                NonZeroUsize::new(black_box(100)).unwrap(),
                NonZeroUsize::new(black_box(200)).unwrap(),
                NonZeroUsize::new(10).unwrap(),
                DungeonType::Cave,
            );
        })
    });
    group.bench_function("500 by 500", |b| {
        b.iter(|| {
            let _ = Dungeon::new(
                NonZeroUsize::new(black_box(500)).unwrap(),
                NonZeroUsize::new(black_box(500)).unwrap(),
                NonZeroUsize::new(10).unwrap(),
                DungeonType::Cave,
            );
        })
    });
    group.finish();
}

criterion_group!(benches, bench_dungeon_generation);
criterion_main!(benches);
