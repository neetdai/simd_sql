use minivec::mini_vec;
use simd_sql::{
    Parser, Query, SelectStatement, InsertStatement, Statement,
    ast::insert::InsertValue,
    ast::statement::StatementInner,
    common::{
        alias::Alias,
        expr::{
            BinaryOp, BinaryOperator, Expr, Field, FunctionCall, NumericLiteral, Star,
            StringLiteral,
        },
        from::{From, Table},
        group::{Group, GroupByExpr},
        limit::Limit,
        order::{Order, OrderDirection, OrderItem},
    },
};

// ============================================================================
// P0 Bug 修复验证: expect_kind 对输入末尾检测
// ============================================================================

#[test]
fn test_p0_expect_kind_catches_eof() {
    let p = Parser::new().unwrap();
    // expect_kind now returns error on None (end of input)
    assert!(
        p.parse("SELECT 1 ORDER BY").is_err(),
        "ORDER BY without expression should fail"
    );
    // INSERT expects columns: INSERT INTO t must have follow-on content
    assert!(
        p.parse("INSERT INTO users").is_err(),
        "INSERT without columns/values should fail"
    );
}

// ============================================================================
// P0 Bug 修复验证: ORDER BY 支持数字序号
// ============================================================================

#[test]
fn test_p0_order_by_numeric_ordinal() {
    let p = Parser::new().unwrap();
    let result = p.parse("SELECT id, name FROM users ORDER BY 1, 2 DESC").unwrap();
    assert_eq!(
        result,
        Statement {
            list: vec![StatementInner::Query(Query::Select(SelectStatement {
                distinct: false,
                columns: vec![
                    Alias { name: None, value: Expr::Field(Field { prefix: None, name: "id" }) },
                    Alias { name: None, value: Expr::Field(Field { prefix: None, name: "name" }) },
                ],
                from: Some(mini_vec![From::Table(Table::Name(Alias {
                    name: None,
                    value: Expr::Field(Field { prefix: None, name: "users" })
                }))]),
                where_statement: None,
                group_by: None,
                having_statement: None,
                order_by: Some(Order {
                    columns: mini_vec![
                        OrderItem {
                            expr: Expr::NumericLiteral(NumericLiteral { value: "1" }),
                            direction: OrderDirection::ASC,
                            nulls_order: None,
                        },
                        OrderItem {
                            expr: Expr::NumericLiteral(NumericLiteral { value: "2" }),
                            direction: OrderDirection::DESC,
                            nulls_order: None,
                        },
                    ]
                }),
                limit: None,
            }))]
        }
    );
}

#[test]
fn test_p0_order_and_group_ordinal() {
    let p = Parser::new().unwrap();
    // ORDER BY 1 DESC silently skipped before the fix
    let r = p.parse("SELECT id FROM users ORDER BY 1 DESC").unwrap();
    match r {
        Statement { list } => match &list[0] {
            StatementInner::Query(Query::Select(stmt)) => {
                assert!(stmt.order_by.is_some(), "ORDER BY should not be empty");
                let order = stmt.order_by.as_ref().unwrap();
                assert_eq!(order.columns.len(), 1);
                assert_eq!(order.columns[0].direction, OrderDirection::DESC);
            }
            _ => panic!("expected Select"),
        },
    }
    // GROUP BY 2 should not be silently skipped
    let r = p.parse("SELECT COUNT(*), dept FROM emp GROUP BY 2").unwrap();
    match r {
        Statement { list } => match &list[0] {
            StatementInner::Query(Query::Select(stmt)) => {
                assert!(stmt.group_by.is_some(), "GROUP BY should not be empty");
            }
            _ => panic!("expected Select"),
        },
    }
}

// ============================================================================
// P0 Bug 修复验证: GROUP BY 支持数字序号
// ============================================================================

#[test]
fn test_p0_group_by_numeric_ordinal() {
    let p = Parser::new().unwrap();
    let result = p.parse("SELECT COUNT(*), dept FROM emp GROUP BY 2").unwrap();
    assert_eq!(
        result,
        Statement {
            list: vec![StatementInner::Query(Query::Select(SelectStatement {
                distinct: false,
                columns: vec![
                    Alias { name: None, value: Expr::FunctionCall(FunctionCall {
                        name: "COUNT",
                        args: mini_vec![Expr::Star(Star { prefix: None })],
                        distinct: false,
                    })},
                    Alias { name: None, value: Expr::Field(Field { prefix: None, name: "dept" })},
                ],
                from: Some(mini_vec![From::Table(Table::Name(Alias {
                    name: None,
                    value: Expr::Field(Field { prefix: None, name: "emp" })
                }))]),
                where_statement: None,
                group_by: Some(Group {
                    columns: mini_vec![GroupByExpr::Simple(Expr::NumericLiteral(NumericLiteral { value: "2" }))],
                }),
                having_statement: None,
                order_by: None,
                limit: None,
            }))]
        }
    );
}

// ============================================================================
// P1 修复验证: INSERT 无列列表语法
// ============================================================================

#[test]
fn test_p1_insert_without_column_list() {
    let p = Parser::new().unwrap();
    let result = p.parse("INSERT INTO users VALUES (1, 'Alice')").unwrap();
    assert_eq!(
        result,
        Statement {
            list: vec![StatementInner::Insert(InsertStatement {
                table: Table::Name(Alias {
                    name: None,
                    value: Expr::Field(Field { prefix: None, name: "users" }),
                }),
                insert_value: InsertValue::Values {
                    columns: mini_vec![],
                    values: mini_vec![mini_vec![
                        Expr::NumericLiteral(NumericLiteral { value: "1" }),
                        Expr::StringLiteral(StringLiteral { value: "'Alice'" }),
                    ]],
                },
            })]
        }
    );
}

#[test]
fn test_p1_insert_without_column_list_multiple_rows() {
    let p = Parser::new().unwrap();
    let sql = "INSERT INTO t VALUES (1, 'a'), (2, 'b')";
    assert!(p.parse(sql).is_ok(), "INSERT without columns, multi-row");
}

// ============================================================================
// P1 修复验证: 位运算符
// ============================================================================

#[test]
fn test_p1_bitwise_and_or_xor() {
    let p = Parser::new().unwrap();
    let result = p.parse("SELECT a & b | c ^ d FROM t").unwrap();
    let expected_expr = Expr::BinaryOp(Box::new(BinaryOp {
        op: BinaryOperator::Or,
        left: Expr::BinaryOp(Box::new(BinaryOp {
            op: BinaryOperator::BitAnd,
            left: Expr::Field(Field { prefix: None, name: "a" }),
            right: Expr::Field(Field { prefix: None, name: "b" }),
        })),
        right: Expr::BinaryOp(Box::new(BinaryOp {
            op: BinaryOperator::BitXor,
            left: Expr::Field(Field { prefix: None, name: "c" }),
            right: Expr::Field(Field { prefix: None, name: "d" }),
        })),
    }));
    assert_eq!(
        result,
        Statement {
            list: vec![StatementInner::Query(Query::Select(SelectStatement {
                distinct: false,
                columns: vec![Alias { name: None, value: expected_expr }],
                from: Some(mini_vec![From::Table(Table::Name(Alias {
                    name: None,
                    value: Expr::Field(Field { prefix: None, name: "t" })
                }))]),
                where_statement: None,
                group_by: None,
                having_statement: None,
                order_by: None,
                limit: None,
            }))]
        }
    );
}

#[test]
fn test_p1_shift_operators() {
    let p = Parser::new().unwrap();
    let sql = "SELECT a << 1, b >> 2 FROM t";
    assert!(p.parse(sql).is_ok(), "shift operators should parse");
}

// ============================================================================
// 原有的基本 SQL 验证（确保回归）
// ============================================================================

#[test]
fn test_regression_basic_select() {
    let p = Parser::new().unwrap();
    assert!(p.parse("SELECT 1").is_ok(), "basic SELECT should parse");
    assert!(p.parse("SELECT * FROM users").is_ok(), "SELECT * should parse");
    assert!(
        p.parse("SELECT id, name FROM users WHERE age > 18").is_ok(),
        "SELECT with WHERE should parse"
    );
}

#[test]
fn test_regression_select_star_from_table() {
    let p = Parser::new().unwrap();
    let result = p.parse("SELECT * FROM users").unwrap();
    assert_eq!(
        result,
        Statement {
            list: vec![StatementInner::Query(Query::Select(SelectStatement {
                distinct: false,
                columns: vec![Alias { name: None, value: Expr::Star(Star { prefix: None }) }],
                from: Some(mini_vec![From::Table(Table::Name(Alias {
                    name: None,
                    value: Expr::Field(Field { prefix: None, name: "users" })
                }))]),
                where_statement: None,
                group_by: None,
                having_statement: None,
                order_by: None,
                limit: None,
            }))]
        }
    );
}

// ============================================================================
// 完整 SELECT 子句验证
// ============================================================================

#[test]
fn test_full_select_all_clauses_with_ast() {
    let p = Parser::new().unwrap();
    let sql = "SELECT category, COUNT(*) AS cnt FROM products \
               WHERE price > 0 GROUP BY category HAVING cnt > 1 \
               ORDER BY cnt DESC LIMIT 5";
    let result = p.parse(sql).unwrap();
    // Verify the structure
    match result {
        Statement { list } => {
            assert_eq!(list.len(), 1);
            match &list[0] {
                StatementInner::Query(Query::Select(stmt)) => {
                    assert_eq!(stmt.columns.len(), 2);
                    assert!(stmt.from.is_some());
                    assert!(stmt.where_statement.is_some());
                    assert!(stmt.group_by.is_some());
                    assert!(stmt.having_statement.is_some());
                    assert!(stmt.order_by.is_some());
                    assert!(stmt.limit.is_some());
                    assert_eq!(stmt.distinct, false);
                }
                _ => panic!("Expected Query::Select"),
            }
        }
    }
}

// ============================================================================
// JOIN 验证
// ============================================================================

#[test]
fn test_all_join_types() {
    let p = Parser::new().unwrap();
    assert!(p.parse("SELECT * FROM a JOIN b ON a.id = b.id").is_ok());
    assert!(p.parse("SELECT * FROM a LEFT JOIN b ON a.id = b.id").is_ok());
    assert!(p.parse("SELECT * FROM a RIGHT JOIN b ON a.id = b.id").is_ok());
    assert!(p.parse("SELECT * FROM a INNER JOIN b ON a.id = b.id").is_ok());
    assert!(p.parse("SELECT * FROM a CROSS JOIN b").is_ok());
    assert!(p.parse("SELECT * FROM a FULL JOIN b ON a.id = b.id").is_ok());
}

// ============================================================================
// 子查询验证
// ============================================================================

#[test]
fn test_subquery_in_from() {
    let p = Parser::new().unwrap();
    assert!(
        p.parse("SELECT * FROM (SELECT id, name FROM users) AS u").is_ok(),
        "subquery in FROM should parse"
    );
    assert!(
        p.parse("SELECT * FROM (SELECT * FROM (SELECT id FROM users) AS inner_t) AS outer_t")
            .is_ok(),
        "nested subqueries should parse"
    );
}

// ============================================================================
// 表达式验证
// ============================================================================

#[test]
fn test_complex_boolean_expression() {
    let p = Parser::new().unwrap();
    let sql = "SELECT * FROM t WHERE (a > 1 AND b < 2) OR (c = 3 AND d != 4)";
    assert!(p.parse(sql).is_ok());
}

#[test]
fn test_not_between_not_in_not_like() {
    let p = Parser::new().unwrap();
    assert!(p.parse("SELECT * FROM t WHERE price NOT BETWEEN 10 AND 20").is_ok());
    assert!(p.parse("SELECT * FROM t WHERE id NOT IN (1, 2, 3)").is_ok());
    assert!(p.parse("SELECT * FROM t WHERE name NOT LIKE 'Test%'").is_ok());
}

#[test]
fn test_bitwise_expression_precedence() {
    let p = Parser::new().unwrap();
    // & has precedence 6, + has precedence 7, so + binds tighter
    // a + b & c → (a + b) & c
    let result = p.parse("SELECT a + b & c FROM t").unwrap();
    match result {
        Statement { list } => {
            match &list[0] {
                StatementInner::Query(Query::Select(stmt)) => {
                    assert_eq!(stmt.columns.len(), 1);
                    // verify the outermost op is BitAnd with Add on the left
                    let expected = Expr::BinaryOp(Box::new(BinaryOp {
                        op: BinaryOperator::BitAnd,
                        left: Expr::BinaryOp(Box::new(BinaryOp {
                            op: BinaryOperator::Add,
                            left: Expr::Field(Field { prefix: None, name: "a" }),
                            right: Expr::Field(Field { prefix: None, name: "b" }),
                        })),
                        right: Expr::Field(Field { prefix: None, name: "c" }),
                    }));
                    assert_eq!(&stmt.columns[0].value, &expected);
                }
                _ => panic!("expected Select"),
            }
        }
    }
}

// ============================================================================
// DML 验证
// ============================================================================

#[test]
fn test_dml_statements() {
    let p = Parser::new().unwrap();
    assert!(p.parse("INSERT INTO users (id, name) VALUES (1, 'Alice')").is_ok());
    assert!(p.parse("UPDATE users SET name = 'Bob' WHERE id = 1").is_ok());
    assert!(p.parse("DELETE FROM users WHERE id = 1").is_ok());
    assert!(p.parse("DELETE FROM users").is_ok());
}

#[test]
fn test_insert_select() {
    let p = Parser::new().unwrap();
    let result = p.parse("INSERT INTO t SELECT * FROM s").unwrap();
    match result {
        Statement { list } => match &list[0] {
            StatementInner::Insert(insert_stmt) => match &insert_stmt.insert_value {
                InsertValue::AllSelect { select } => {
                    assert_eq!(select.columns.len(), 1);
                }
                _ => panic!("expected AllSelect"),
            },
            _ => panic!("expected Insert"),
        },
    }
}

#[test]
fn test_insert_partof_select() {
    let p = Parser::new().unwrap();
    let result = p.parse("INSERT INTO t (id, name) SELECT id, name FROM s").unwrap();
    match result {
        Statement { list } => match &list[0] {
            StatementInner::Insert(insert_stmt) => match &insert_stmt.insert_value {
                InsertValue::PartOfSelect { columns, .. } => {
                    assert_eq!(columns.len(), 2);
                }
                _ => panic!("expected PartOfSelect"),
            },
            _ => panic!("expected Insert"),
        },
    }
}

// ============================================================================
// CTE 验证
// ============================================================================

#[test]
fn test_cte_basic() {
    let p = Parser::new().unwrap();
    assert!(p.parse("WITH cte AS (SELECT id FROM users) SELECT * FROM cte").is_ok());
}

#[test]
fn test_cte_recursive() {
    let p = Parser::new().unwrap();
    let sql = "WITH RECURSIVE t(n) AS (\
      SELECT 1 UNION ALL SELECT n + 1 FROM t WHERE n < 10\
    ) SELECT n FROM t";
    assert!(p.parse(sql).is_ok(), "CTE RECURSIVE should parse");
}

// ============================================================================
// 集合操作验证
// ============================================================================

#[test]
fn test_set_operations() {
    let p = Parser::new().unwrap();
    assert!(p.parse("SELECT id FROM users UNION SELECT id FROM admins").is_ok());
    assert!(p.parse("SELECT id FROM users INTERSECT SELECT id FROM admins").is_ok());
    assert!(p.parse("SELECT id FROM users EXCEPT SELECT id FROM admins").is_ok());
}

// ============================================================================
// Window 函数验证
// ============================================================================

#[test]
fn test_window_functions() {
    let p = Parser::new().unwrap();
    assert!(p.parse("SELECT ROW_NUMBER() OVER (ORDER BY id) FROM users").is_ok());
    assert!(
        p.parse("SELECT RANK() OVER (PARTITION BY dept ORDER BY salary DESC) as r FROM emp")
            .is_ok(),
        "partition by should parse"
    );
}

// ============================================================================
// 字面量验证
// ============================================================================

#[test]
fn test_literals() {
    let p = Parser::new().unwrap();
    let sql = "SELECT 'hello' as s, 123 as n, 3.14 as pi, TRUE as t, FALSE as f, NULL as e, 0xFF as hex FROM dual";
    assert!(p.parse(sql).is_ok(), "all literals should parse");
}

// ============================================================================
// 注释验证
// ============================================================================

#[test]
fn test_all_comment_styles() {
    let p = Parser::new().unwrap();
    assert!(p.parse("SELECT /* comment */ * FROM users").is_ok());
    assert!(p.parse("SELECT a -- line comment\nFROM users").is_ok());
    assert!(p.parse("SELECT a // line comment\nFROM users").is_ok());
}

// ============================================================================
// LIMIT / OFFSET 语法验证
// ============================================================================

#[test]
fn test_limit_syntaxes() {
    let p = Parser::new().unwrap();
    assert!(p.parse("SELECT * FROM users LIMIT 10").is_ok());
    assert!(p.parse("SELECT * FROM users LIMIT 10 OFFSET 5").is_ok());
    assert!(p.parse("SELECT * FROM users LIMIT 5, 10").is_ok());
}

// ============================================================================
// 错误路径验证
// ============================================================================

#[test]
fn test_error_handling() {
    let p = Parser::new().unwrap();
    assert!(p.parse("").is_err(), "empty input should fail");
    assert!(p.parse("   ").is_err(), "whitespace only should fail");
    assert!(p.parse("SELECT 'hello").is_err(), "unterminated string should fail");
}

// ============================================================================
// 大小写不敏感验证
// ============================================================================

#[test]
fn test_case_insensitive_keywords() {
    let p = Parser::new().unwrap();
    assert!(p.parse("SELECT * FROM users").is_ok());
    assert!(p.parse("select * from users").is_ok());
    assert!(p.parse("Select * From users").is_ok());
}

// ============================================================================
// 多语句验证（分号分隔）
// ============================================================================

#[test]
fn test_multiple_statements() {
    let p = Parser::new().unwrap();
    assert!(
        p.parse("SELECT 1; SELECT 2").is_ok(),
        "multiple statements should parse"
    );
}

// ============================================================================
// 函数调用验证
// ============================================================================

#[test]
fn test_function_calls() {
    let p = Parser::new().unwrap();
    assert!(p.parse("SELECT COUNT(*) FROM t").is_ok());
    assert!(p.parse("SELECT CONCAT(a, ' ', b) AS result FROM t").is_ok());
}

// ============================================================================
// DISTINCT 验证
// ============================================================================

#[test]
fn test_distinct() {
    let p = Parser::new().unwrap();
    let result = p.parse("SELECT DISTINCT category, region FROM sales").unwrap();
    match result {
        Statement { list } => match &list[0] {
            StatementInner::Query(Query::Select(stmt)) => {
                assert_eq!(stmt.distinct, true);
            }
            _ => panic!("expected Select"),
        },
    }
}

// ============================================================================
// 复杂真实 SQL 验证
// ============================================================================

#[test]
fn test_complex_real_world_query() {
    let p = Parser::new().unwrap();
    let sql = "\
SELECT u.id, u.name, o.total AS order_total, o.created_at \
FROM users u \
JOIN orders o ON u.id = o.user_id \
WHERE u.active = 1 AND o.total > 0 \
ORDER BY o.created_at DESC \
LIMIT 20";
    assert!(p.parse(sql).is_ok(), "complex real-world query should parse");
}

// ============================================================================
// 运算符优先级综合验证
// ============================================================================

#[test]
fn test_operator_precedence_comprehensive() {
    let p = Parser::new().unwrap();
    // OR < AND < = <> < NOT/BETWEEN/IN/LIKE < < <= > >= < << >> & | ^ < + - < * / %
    // Test a few key precedences:
    assert!(p.parse("SELECT * FROM t WHERE a = 1 OR b = 2 AND c = 3").is_ok());
    assert!(p.parse("SELECT * FROM t WHERE a + b * c > 10").is_ok());
    assert!(p.parse("SELECT a & b + c FROM t").is_ok());
}
