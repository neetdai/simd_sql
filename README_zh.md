# simd_sql — 高性能 SIMD 加速 SQL 解析器

**纯 Rust 实现 · SSE2 加速词法分析 · 零拷贝 AST · 67 关键词覆盖**

`simd_sql` 是一个专注于解析性能的 SQL 解析器。它将 SQL 文本解析为带生命周期的 AST，AST 节点直接引用原始文本的 `&str` 切片，无需额外分配。词法分析阶段通过 SSE2 SIMD 指令集并行扫描字节（16 字节/批），在非 x86 平台自动降级为标量实现。

---

## 快速开始

```rust
use simd_sql::Parser;

let parser = Parser::new()?;
let stmt = parser.parse("SELECT id, name FROM users WHERE age > 18")?;
// stmt 是 Statement<'input>，所有标识符和字面量直接引用原始 SQL 文本
```

```toml
[dependencies]
simd_sql = { git = "https://github.com/neetdai/simd_sql" }
```

---

## 功能覆盖

### SELECT 语句 — 完整支持

| 子句 | 状态 | 说明 |
|------|------|------|
| SELECT 列列表 | ✅ | 别名 (`AS`)、`table.column`、`*`、`table.*` |
| DISTINCT | ✅ | |
| FROM / 子查询 / 表别名 | ✅ | 支持 `(SELECT ...) AS t` |
| JOIN | ✅ | `JOIN`、`INNER`、`LEFT`、`RIGHT`、`FULL`、`CROSS` + `OUTER` 关键词可省略 |
| WHERE | ✅ | 完整表达式系统 |
| GROUP BY / HAVING | ✅ | |
| ORDER BY | ✅ | `ASC`/`DESC`、`NULLS FIRST`/`NULLS LAST` |
| LIMIT / OFFSET | ✅ | `LIMIT n`、`LIMIT n OFFSET m`、`LIMIT m, n` |

### CTE — 完整支持

```sql
WITH RECURSIVE t(n) AS (
  SELECT 1 UNION ALL SELECT n + 1 FROM t WHERE n < 10
)
SELECT n FROM t
```

✅ 多绑定、列重命名、集合操作嵌套

### 窗口函数

```sql
SELECT ROW_NUMBER() OVER (PARTITION BY dept ORDER BY salary DESC) as rn
FROM employees
```

✅ `ROW_NUMBER` / `RANK` / `DENSE_RANK` + `OVER (PARTITION BY ... ORDER BY ...)`

### 集合操作

✅ `UNION` / `UNION ALL` / `INTERSECT` / `EXCEPT`，优先级正确解析，ORDER BY / LIMIT 绑定最外层

### DML — 基本支持

| 语句 | 支持 | 缺失 |
|------|------|------|
| INSERT | ✅ 表名 + 列名 + 多行 VALUES | `INSERT ... SELECT`、`ON CONFLICT`、`DEFAULT VALUES`、`RETURNING` |
| UPDATE | ✅ 多列 SET + WHERE | `FROM`、`JOIN`、`RETURNING` |
| DELETE | ✅ FROM + WHERE | `USING`、`RETURNING` |

### 表达式系统 — 17 种 Expr 变体

| 类别 | 包含 |
|------|------|
| 算术 | `+` `-` `*` `/` `%`（7 级 Pratt 优先级，全部左结合） |
| 比较 | `=` `<>` `<` `<=` `>` `>=` |
| 逻辑 | `AND` `OR` |
| 后置操作 | `BETWEEN` `IN` `LIKE` `IS NULL` `OVER`（window） |
| 函数调用 | `func(args...)`，支持 `DISTINCT` |
| CASE 表达式 | `CASE WHEN ... THEN ... ELSE ... END` |
| 存在检测 | `EXISTS (subquery)`、`NOT EXISTS (subquery)` |
| 字面量 | 十进制 `123`、十六进制 `0xFF`、八进制 `0o777`、二进制 `0b1010`、下划线分隔 `1_000_000`、科学记数 `1.5E10`、字符串（反斜杠转义）、`TRUE`、`FALSE`、`NULL` |

### 注释

✅ `-- 行注释`、`/* 块注释 */`、`// 行注释` — SSE 序列扫描跳过

---

## 性能

### SIMD 加速路径

| 词法功能 | 算法 | 吞吐 |
|----------|------|------|
| 空白跳过 | SSE2 `_mm_cmpeq_epi8` / `_mm_movemask_epi8` | 16 字节/批 |
| 数字扫描 | SSE2 范围检测 | 16 字节/批 |
| 标识符扫描 | SSE2 混合匹配（范围 + 精确） | 16 字节/批 |
| 字符串引号查找 | SSE2 `skip_until_match` | 16 字节/批 |
| 块注释 `*/` 查找 | SSE2 `skip_until_sequence`（2 字节序列 + 跨边界重叠） | 15 字节/批 + 1 重叠 |
| 行注释 `\n` 查找 | SSE2 `skip_until_match` | 16 字节/批 |
| 关键词匹配 | Aho-Corasick 自动机 | O(n) 单遍，67 关键词大小写不敏感 |

### 设计特点

- **零拷贝**：TokenTable 存储 `&'a str`，AST 节点直接引用，无字符串复制
- **编译期分派**：`cfg_select!` 在 SSE2 可用时编译 SIMD 路径，否则编译标量回退
- **反斜杠转义**：`is_escaped` 函数正确处理 `\`、`\\`、`\\\'` 的奇偶计数
- **跨边界保护**：多字节序列搜索每次前进步长 = 16 - (N - 1) 防止跨 chunk 遗漏

---

## 架构

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
    │               token 类型标记   原始文本的 &str 切片
    │
    ▼
Parser (Pratt recursive descent)
    │
    ▼
Statement<'a> ── AST with &'a str fields
```

类型约束贯穿生命周期：`Statement<'a>`、`Expr<'a>`、`TokenTable<'a>` 均绑定到输入 SQL 文本的全周期，AST 构建完成后 TokenTable 可丢弃，AST 仍有效。

---

## 缺失特性（按优先级）

| 特性 | 优先级 |
|------|--------|
| DDL（CREATE / DROP / ALTER TABLE） | ⭐⭐⭐ |
| 事务（BEGIN / COMMIT / ROLLBACK） | ⭐⭐⭐ |
| 窗口函数 frame 子句（ROWS / RANGE） | ⭐⭐ |
| CAST / COALESCE / NULLIF 表达式 | ⭐⭐ |
| INSERT ... SELECT / ON CONFLICT | ⭐⭐ |
| 标量子查询 `WHERE x = (SELECT ...)` | ⭐⭐ |
| RETURNING 子句 | ⭐⭐ |
| ARM NEON SIMD 支持 | ⭐⭐ |
| AVX2 (256-bit) SIMD 路径 | ⭐ |
| 参数占位符 `$1` / `?` | ⭐ |

---

## 构建要求

- **Rust edition 2024**（需 rustc 1.95.0+）
- **x86_64 处理器**：SSE2 支持（2004 年后所有 x86_64 CPU 都支持）
- 非 x86 平台：自动使用标量回退，功能完整

---

## License

Apache License © 2026 neetdai
