#[macro_use]
extern crate criterion;

use criterion::Criterion;
use pathutils::join;
use std::path::Path;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| {
        let path = Path::new("/test");
        b.iter(|| join(&path, "../me.txt"));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
