use criterion::{Criterion, criterion_group, criterion_main};
use simd_sql::Parser;
use std::hint::black_box;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("sql parser 1", |b| {
        let parser = Parser::new().unwrap();
        b.iter(|| {
            let sql = "SELECT * FROM table WHERE id = 1";

            parser.parse(&sql).unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
