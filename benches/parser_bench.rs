use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use simd_sql::Parser;
use std::hint::black_box;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("sql parser 1");

    let parser = Parser::new().unwrap();

    let sql_1 = "SELECT * FROM a WHERE id = 1";
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

    let sql_5 = "SELECT COUNT(*), SUM(amount), AVG(price), MAX(created_at), MIN(updated_at) FROM orders WHERE status = 'active' GROUP BY user_id, category HAVING COUNT(*) > 5 ORDER BY SUM(amount) DESC LIMIT 100 OFFSET 10";
    let sql_len_5 = sql_5.len();
    group.throughput(Throughput::Elements(sql_5.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("sql parser 5 - aggregation", sql_len_5),
        sql_5,
        |b, i| {
            b.iter(|| {
                parser.parse(black_box(&i)).unwrap();
            });
        },
    );

    let sql_6 = "SELECT id, name, email, created_at FROM users WHERE id IN (1, 2, 3, 4, 5, 6, 7, 8, 9, 10) AND status = 'active' ORDER BY name ASC, created_at DESC";
    let sql_len_6 = sql_6.len();
    group.throughput(Throughput::Elements(sql_6.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("sql parser 6 - IN clause", sql_len_6),
        sql_6,
        |b, i| {
            b.iter(|| {
                parser.parse(black_box(&i)).unwrap();
            });
        },
    );

    let sql_7 = "SELECT * FROM (SELECT id, name FROM users WHERE active = true) AS active_users JOIN (SELECT user_id, COUNT(*) as order_count FROM orders GROUP BY user_id) AS user_orders ON active_users.id = user_orders.user_id";
    let sql_len_7 = sql_7.len();
    group.throughput(Throughput::Elements(sql_7.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("sql parser 7 - subquery", sql_len_7),
        sql_7,
        |b, i| {
            b.iter(|| {
                parser.parse(black_box(&i)).unwrap();
            });
        },
    );

    // let sql_8 = "SELECT u.id, u.name, o.total as order_total, (SELECT COUNT(*) FROM order_items WHERE order_id = o.id) as item_count FROM users u LEFT JOIN orders o ON u.id = o.user_id WHERE u.created_at > '2024-01-01'";
    // let sql_len_8 = sql_8.len();
    // group.throughput(Throughput::Elements(sql_8.len() as u64));
    // group.bench_with_input(
    //     BenchmarkId::new("sql parser 8 - correlated subquery", sql_len_8),
    //     sql_8,
    //     |b, i| {
    //         b.iter(|| {
    //             parser.parse(black_box(&i)).unwrap();
    //         });
    //     },
    // );

    let sql_9 = "SELECT id, name, LOWER(email) as email_lower, UPPER(name) as name_upper, CONCAT(first_name, ' ', last_name) as full_name, LENGTH(name) as name_len FROM users WHERE TRIM(status) = 'active'";
    let sql_len_9 = sql_9.len();
    group.throughput(Throughput::Elements(sql_9.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("sql parser 9 - functions", sql_len_9),
        sql_9,
        |b, i| {
            b.iter(|| {
                parser.parse(black_box(&i)).unwrap();
            });
        },
    );

    let sql_10 = "SELECT id, name, price * 1.1 as tax_price, price + 100 as shipping, amount - discount as final_amount, total / quantity as unit_price, age % 10 as age_mod FROM products";
    let sql_len_10 = sql_10.len();
    group.throughput(Throughput::Elements(sql_10.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("sql parser 10 - arithmetic", sql_len_10),
        sql_10,
        |b, i| {
            b.iter(|| {
                parser.parse(black_box(&i)).unwrap();
            });
        },
    );

    let sql_11 = "SELECT DISTINCT category, region FROM sales WHERE year = 2024 AND (sales > 1000 OR profit > 500) AND region IN ('North', 'South', 'East', 'West') ORDER BY category, region";
    let sql_len_11 = sql_11.len();
    group.throughput(Throughput::Elements(sql_11.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("sql parser 11 - distinct and complex where", sql_len_11),
        sql_11,
        |b, i| {
            b.iter(|| {
                parser.parse(black_box(&i)).unwrap();
            });
        },
    );

    let sql_12 = "SELECT * FROM a UNION SELECT * FROM b UNION ALL SELECT * FROM c INTERSECT SELECT * FROM d EXCEPT SELECT * FROM e ORDER BY id LIMIT 50";
    let sql_len_12 = sql_12.len();
    group.throughput(Throughput::Elements(sql_12.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("sql parser 12 - set operations", sql_len_12),
        sql_12,
        |b, i| {
            b.iter(|| {
                parser.parse(black_box(&i)).unwrap();
            });
        },
    );

    let sql_13 = "SELECT a.id, b.name, c.value FROM table1 a CROSS JOIN table2 b, table3 c WHERE a.id = b.id AND b.id = c.id AND a.status <> 'deleted' AND a.amount BETWEEN 100 AND 500";
    let sql_len_13 = sql_13.len();
    group.throughput(Throughput::Elements(sql_13.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("sql parser 13 - complex joins", sql_len_13),
        sql_13,
        |b, i| {
            b.iter(|| {
                parser.parse(black_box(&i)).unwrap();
            });
        },
    );

    let sql_14 = "UPDATE users SET name = 'John', email = 'john@example.com', status = 'active', updated_at = NOW() WHERE id = 1 AND version = 5";
    let sql_len_14 = sql_14.len();
    group.throughput(Throughput::Elements(sql_14.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("sql parser 14 - update", sql_len_14),
        sql_14,
        |b, i| {
            b.iter(|| {
                parser.parse(black_box(&i)).unwrap();
            });
        },
    );

    let sql_15 = "INSERT INTO users (id, name, email, created_at) VALUES (1, 'Test', 'test@example.com', '2024-01-01'), (2, 'Test2', 'test2@example.com', '2024-01-02'), (3, 'Test3', 'test3@example.com', '2024-01-03')";
    let sql_len_15 = sql_15.len();
    group.throughput(Throughput::Elements(sql_15.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("sql parser 15 - insert", sql_len_15),
        sql_15,
        |b, i| {
            b.iter(|| {
                parser.parse(black_box(&i)).unwrap();
            });
        },
    );

    let sql_16 = "DELETE FROM orders WHERE status = 'cancelled' AND created_at < '2023-01-01' AND id NOT IN (SELECT order_id FROM invoices WHERE paid = true)";
    let sql_len_16 = sql_16.len();
    group.throughput(Throughput::Elements(sql_16.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("sql parser 16 - delete with subquery", sql_len_16),
        sql_16,
        |b, i| {
            b.iter(|| {
                parser.parse(black_box(&i)).unwrap();
            });
        },
    );

    let sql_17 = "SELECT COALESCE(a.name, b.name, 'Unknown') as final_name, NULLIF(a.price, 0) as valid_price, IF(active = 1, 'Yes', 'No') as is_active, CASE WHEN a.quantity > 10 THEN 'In Stock' WHEN a.quantity > 0 THEN 'Low Stock' ELSE 'Out of Stock' END as stock_status FROM products a, categories b";
    let sql_len_17 = sql_17.len();
    group.throughput(Throughput::Elements(sql_17.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("sql parser 17 - null handling", sql_len_17),
        sql_17,
        |b, i| {
            b.iter(|| {
                parser.parse(black_box(&i)).unwrap();
            });
        },
    );

    let sql_18 = "WITH active_users AS (SELECT id, name FROM users WHERE status = 'active'), recent_orders AS (SELECT user_id, SUM(amount) as total FROM orders WHERE created_at > '2024-01-01' GROUP BY user_id) SELECT au.id, au.name, ro.total FROM active_users au LEFT JOIN recent_orders ro ON au.id = ro.user_id";
    let sql_len_18 = sql_18.len();
    group.throughput(Throughput::Elements(sql_18.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("sql parser 18 - CTE", sql_len_18),
        sql_18,
        |b, i| {
            b.iter(|| {
                parser.parse(black_box(&i)).unwrap();
            });
        },
    );

    let sql_19 = "SELECT DATE(created_at) as date, YEAR(created_at) as year, MONTH(created_at) as month, DAY(created_at) as day, HOUR(created_at) as hour, MINUTE(created_at) as minute FROM events WHERE created_at BETWEEN '2024-01-01' AND '2024-12-31'";
    let sql_len_19 = sql_19.len();
    group.throughput(Throughput::Elements(sql_19.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("sql parser 19 - date functions", sql_len_19),
        sql_19,
        |b, i| {
            b.iter(|| {
                parser.parse(black_box(&i)).unwrap();
            });
        },
    );

    let sql_20 = "SELECT p.id, p.name, p.price, ROW_NUMBER() OVER (PARTITION BY category ORDER BY price DESC) as rank, RANK() OVER (ORDER BY price) as price_rank, DENSE_RANK() OVER (ORDER BY price) as dense_price_rank FROM products p";
    let sql_len_20 = sql_20.len();
    group.throughput(Throughput::Elements(sql_20.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("sql parser 20 - window functions", sql_len_20),
        sql_20,
        |b, i| {
            b.iter(|| {
                parser.parse(black_box(&i)).unwrap();
            });
        },
    );

    let sql_21 = "SELECT id, name, email, phone, address, city, state, zipcode, country, created_at, updated_at, deleted_at, status, version FROM users WHERE id = 1";
    let sql_len_21 = sql_21.len();
    group.throughput(Throughput::Elements(sql_21.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("sql parser 21 - many columns", sql_len_21),
        sql_21,
        |b, i| {
            b.iter(|| {
                parser.parse(black_box(&i)).unwrap();
            });
        },
    );

    let sql_22 = "SELECT * FROM (SELECT * FROM (SELECT * FROM users WHERE active = true) AS t1 WHERE id > 100) AS t2 WHERE name LIKE '%John%' ORDER BY created_at DESC LIMIT 20";
    let sql_len_22 = sql_22.len();
    group.throughput(Throughput::Elements(sql_22.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("sql parser 22 - nested subqueries", sql_len_22),
        sql_22,
        |b, i| {
            b.iter(|| {
                parser.parse(black_box(&i)).unwrap();
            });
        },
    );

    let sql_23 = "SELECT 'string literal' as str, 123 as num, 3.14 as pi, TRUE as flag, FALSE as not_flag, NULL as empty, 0xFF as hex, 0b1010 as binary FROM dual";
    let sql_len_23 = sql_23.len();
    group.throughput(Throughput::Elements(sql_23.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("sql parser 23 - literals", sql_len_23),
        sql_23,
        |b, i| {
            b.iter(|| {
                parser.parse(black_box(&i)).unwrap();
            });
        },
    );

    let sql_24 = "SELECT id, name FROM users WHERE (name = 'Alice' AND email LIKE '%@example.com') OR (name = 'Bob' AND email LIKE '%@test.com') OR (name = 'Charlie' AND status = 'active')";
    let sql_len_24 = sql_24.len();
    group.throughput(Throughput::Elements(sql_24.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("sql parser 24 - complex boolean", sql_len_24),
        sql_24,
        |b, i| {
            b.iter(|| {
                parser.parse(black_box(&i)).unwrap();
            });
        },
    );

    let sql_25 = "SELECT u.id, u.name, COUNT(o.id) as order_count, SUM(o.total) as total_spent, AVG(o.total) as avg_order_value FROM users u LEFT JOIN orders o ON u.id = o.user_id LEFT JOIN order_items oi ON o.id = oi.order_id WHERE u.status = 'active' GROUP BY u.id, u.name HAVING COUNT(o.id) > 0 ORDER BY total_spent DESC NULLS LAST";
    let sql_len_25 = sql_25.len();
    group.throughput(Throughput::Elements(sql_25.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("sql parser 25 - complex aggregation", sql_len_25),
        sql_25,
        |b, i| {
            b.iter(|| {
                parser.parse(black_box(&i)).unwrap();
            });
        },
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
