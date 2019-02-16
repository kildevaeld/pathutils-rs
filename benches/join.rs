#[macro_use]
extern crate criterion;

use criterion::Criterion;
use pathutils::resolve;
use std::path::Path;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| {
        b.iter(|| resolve("/test", "../me.txt"));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
