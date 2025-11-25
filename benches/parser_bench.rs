use bumpalo::Bump;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use simd_sql::Parser;
use std::hint::black_box;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("sql parser 1");

    let bump = Bump::new();
    let parser = Parser::new(&bump).unwrap();

    let sql_1 = "SELECT * FROM table WHERE id = 1";
    let sql_len_1 = sql_1.len();
    group.throughput(Throughput::Elements(sql_1.len() as u64));
    group.bench_with_input(BenchmarkId::new("sql parser 1", sql_len_1), sql_1, |b, i| {
        b.iter(|| {
            parser.parse(&i).unwrap();
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
