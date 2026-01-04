use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use simd_sql::Parser;
use std::hint::black_box;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("sql parser 1");

    let parser = Parser::new().unwrap();

    let sql_1 = "SELECT * FROM table WHERE id = 1";
    let sql_len_1 = sql_1.len();
    group.throughput(Throughput::Elements(sql_1.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("sql parser 1", sql_len_1),
        sql_1,
        |b, i| {
            b.iter(|| {
                parser.parse(&i).unwrap();
            });
        },
    );

    let sql_2 = "SELECT id, name FROM employees WHERE salary > 50000;";
    let sql_len_2 = sql_2.len();
    group.throughput(Throughput::Elements(sql_2.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("sql parser 2", sql_len_2),
        sql_2,
        |b, i| {
            b.iter(|| {
                parser.parse(black_box(&i)).unwrap();
            });
        },
    );

    let sql_3 = "SELECT u.name, o.order_date, p.product_name
FROM users u
JOIN orders o ON u.id = o.user_id
JOIN order_items oi ON o.id = oi.order_id
JOIN products p ON oi.product_id = p.id
WHERE u.active = 1 AND o.status = 'completed';";

    let sql_len_3 = sql_3.len();
    group.throughput(Throughput::Elements(sql_3.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("sql parser 3", sql_len_3),
        sql_3,
        |b, i| {
            b.iter(|| {
                parser.parse(black_box(&i)).unwrap();
            });
        },
    );

    let sql_4 = "SELECT id,
       CASE 
           WHEN score >= 90 THEN 'A'
           WHEN score >= 80 THEN 'B'
           WHEN score >= 70 THEN 'C'
           WHEN score >= 60 THEN 'D'
           ELSE 'F'
       END as grade,
       CASE department
           WHEN 'IT' THEN 'Technology'
           WHEN 'HR' THEN 'Human Resources'
           WHEN 'FIN' THEN 'Finance'
           ELSE 'Other'
       END as dept_full_name
FROM students;";

    let sql_len_4 = sql_4.len();
    group.throughput(Throughput::Elements(sql_4.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("sql parser 4", sql_len_4),
        sql_4,
        |b, i| {
            b.iter(|| {
                parser.parse(black_box(&i)).unwrap();
            });
        },
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
