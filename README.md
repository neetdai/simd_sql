# simd_sql — A High-Performance SIMD-Accelerated SQL Parser

**Pure Rust · SSE2 Lexing · Zero-Copy AST · 67 Keywords Covered**

[中文版](./README_zh.md)

`simd_sql` is a SQL parser built for parsing throughput. It produces a lifetime-tagged AST where every node references `&str` slices directly from the source text — no string copies, no intermediate allocations. The lexer uses SSE2 SIMD intrinsics to scan 16 bytes per batch. On non-x86 platforms it falls back to scalar code automatically.

---

## Quick Start

```rust
use simd_sql::Parser;

let parser = Parser::new()?;
let stmt = parser.parse("SELECT id, name FROM users WHERE age > 18")?;
// stmt is a Statement<'input> — all identifiers and literals
// are direct &str borrows of the original SQL text
```

```toml
[dependencies]
simd_sql = { git = "https://github.com/neetdai/simd_sql" }
```

---

## Feature Coverage

### SELECT Statement — Full Support

| Clause | Status | Notes |
|--------|--------|-------|
| SELECT column list | ✅ | Aliases (`AS`), `table.column`, `*`, `table.*` |
| DISTINCT | ✅ | |
| FROM / subquery / table alias | ✅ | Supports `(SELECT ...) AS t` |
| JOIN | ✅ | `JOIN`, `INNER`, `LEFT`, `RIGHT`, `FULL`, `CROSS`; `OUTER` keyword is optional |
| WHERE | ✅ | Full expression system |
| GROUP BY / HAVING | ✅ | |
| ORDER BY | ✅ | `ASC`/`DESC`, `NULLS FIRST`/`NULLS LAST` |
| LIMIT / OFFSET | ✅ | `LIMIT n`, `LIMIT n OFFSET m`, `LIMIT m, n` |

### CTE — Full Support

```sql
WITH RECURSIVE t(n) AS (
  SELECT 1 UNION ALL SELECT n + 1 FROM t WHERE n < 10
)
SELECT n FROM t
```

✅ Multiple bindings, column renaming, nested set operations inside CTE body

### Window Functions

```sql
SELECT ROW_NUMBER() OVER (PARTITION BY dept ORDER BY salary DESC) as rn
FROM employees
```

✅ `ROW_NUMBER` / `RANK` / `DENSE_RANK` + `OVER (PARTITION BY ... ORDER BY ...)`

### Set Operations

✅ `UNION` / `UNION ALL` / `INTERSECT` / `EXCEPT` — correct precedence, ORDER BY / LIMIT bind to the outermost query

### DML — Basic Support

| Statement | Supported | Missing |
|-----------|-----------|---------|
| INSERT | ✅ table + columns + multi-row VALUES | `INSERT ... SELECT`, `ON CONFLICT`, `DEFAULT VALUES`, `RETURNING` |
| UPDATE | ✅ multi-column SET + WHERE | `FROM`, `JOIN`, `RETURNING` |
| DELETE | ✅ FROM + WHERE | `USING`, `RETURNING` |

### Expression System — 17 Expr Variants

| Category | Includes |
|----------|----------|
| Arithmetic | `+` `-` `*` `/` `%` (7-level Pratt precedence, all left-associative) |
| Comparison | `=` `<>` `<` `<=` `>` `>=` |
| Logical | `AND` `OR` |
| Postfix operators | `BETWEEN` `IN` `LIKE` `IS NULL` `OVER` (window) |
| Function calls | `func(args...)`, supports `DISTINCT` inside aggregates |
| CASE | `CASE WHEN ... THEN ... ELSE ... END` |
| Existence | `EXISTS (subquery)`, `NOT EXISTS (subquery)` |
| Literals | decimal `123`, hex `0xFF`, octal `0o777`, binary `0b1010`, underscore separator `1_000_000`, scientific notation `1.5E10`, strings (backslash escaping), `TRUE`, `FALSE`, `NULL` |

### Comments

✅ `-- line comments`, `/* block comments */`, `// line comments` — skipped via SSE sequence scanning

---

## Performance

### SIMD-Accelerated Paths

| Lexer Function | Algorithm | Throughput |
|----------------|-----------|------------|
| Whitespace skipping | SSE2 `_mm_cmpeq_epi8` / `_mm_movemask_epi8` | 16 bytes/batch |
| Number scanning | SSE2 range detection | 16 bytes/batch |
| Identifier scanning | SSE2 mixed match (ranges + exact) | 16 bytes/batch |
| String quote search | SSE2 `skip_until_match` | 16 bytes/batch |
| Block comment `*/` search | SSE2 `skip_until_sequence` (2-byte sequence + boundary overlap) | 15 + 1 overlap bytes/batch |
| Line comment `\n` search | SSE2 `skip_until_match` | 16 bytes/batch |
| Keyword matching | Aho-Corasick automaton | O(n) single pass, 67 keywords, case-insensitive |

### Design Highlights

- **Zero-copy**: `TokenTable` stores `&'a str` slices; AST nodes borrow directly — no string duplication
- **Compile-time dispatch**: `cfg_select!` compiles SIMD path when SSE2 is available, scalar fallback otherwise
- **Backslash escaping**: `is_escaped` uses parity counting for `\`, `\\`, `\\\'`
- **Cross-boundary safety**: multi-byte sequence searchers advance by `16 - (N - 1)` bytes to prevent missing matches across chunk boundaries

---

## Architecture

```
SQL text (&str)
    │
    ▼
Lexer (SIMD accelerated)
    │
    ▼
TokenTable<'a> ── Vec<TokenKind> + Vec<&'a str>
    │                      │               │
    │                      ▼               ▼
    │               token kind flags   &str slices of source text
    │
    ▼
Parser (Pratt recursive descent)
    │
    ▼
Statement<'a> ── AST with &'a str fields
```

Lifetime `'a` ties everything to the original SQL text. Once the AST is built, `TokenTable` can be dropped; the AST remains valid.

---

## Missing Features (by Priority)

| Feature | Priority |
|---------|----------|
| DDL (CREATE / DROP / ALTER TABLE) | ⭐⭐⭐ |
| Transactions (BEGIN / COMMIT / ROLLBACK) | ⭐⭐⭐ |
| Window function frame clause (ROWS / RANGE) | ⭐⭐ |
| CAST / COALESCE / NULLIF expressions | ⭐⭐ |
| INSERT ... SELECT / ON CONFLICT | ⭐⭐ |
| Scalar subqueries `WHERE x = (SELECT ...)` | ⭐⭐ |
| RETURNING clause | ⭐⭐ |
| ARM NEON SIMD support | ⭐⭐ |
| AVX2 (256-bit) SIMD path | ⭐ |
| Parameter placeholders `$1` / `?` | ⭐ |

---

## Build Requirements

- **Rust edition 2024** (rustc 1.95.0+)
- **x86_64 CPU**: SSE2 support (all x86_64 CPUs since 2004)
- Non-x86 platforms: automatic scalar fallback, full feature parity

---

## License

Apache 2.0 © 2026 neetdai

---

[中文文档](./README_zh.md)