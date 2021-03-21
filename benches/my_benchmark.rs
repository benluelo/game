use criterion::{black_box, criterion_group, criterion_main, Criterion};
use game::floor_builder;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("floor_builder", |b| b.iter(|| floor_builder()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);