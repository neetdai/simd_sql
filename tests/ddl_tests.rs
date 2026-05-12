use simd_sql::{
    Parser, Statement,
    ast::{
        ddl::{
            AlterTable, AlterTableOperation, ColumnConstraint, ColumnDef, CreateTable,
            DdlStatement, DropTable,
        },
        statement::StatementInner,
    },
};

// ============================================================================
// CREATE TABLE
// ============================================================================

#[test]
fn test_create_table_basic() {
    let p = Parser::new().unwrap();
    let result = p.parse("CREATE TABLE t (id INT)").unwrap();
    assert_eq!(
        result,
        Statement {
            list: vec![StatementInner::Ddl(DdlStatement::CreateTable(
                CreateTable::Table {
                    if_not_exists: false,
                    name: "t",
                    columns: vec![ColumnDef {
                        name: "id",
                        col_type: "INT",
                        col_type_params: None,
                        constraint: ColumnConstraint {
                            not_null: false,
                            default: None,
                            primary_key: false,
                            unique: false,
                        },
                    }],
                }
            ))]
        }
    );
}

#[test]
fn test_create_table_if_not_exists() {
    let p = Parser::new().unwrap();
    let result = p.parse("CREATE TABLE IF NOT EXISTS t (id INT)").unwrap();
    match result {
        Statement { list } => match &list[0] {
            StatementInner::Ddl(DdlStatement::CreateTable(CreateTable::Table {
                if_not_exists,
                name,
                ..
            })) => {
                assert!(*if_not_exists, "IF NOT EXISTS should be true");
                assert_eq!(*name, "t");
            }
            _ => panic!("expected CreateTable"),
        },
    }
}

#[test]
fn test_create_table_multiple_columns() {
    let p = Parser::new().unwrap();
    let sql = "CREATE TABLE users (id INT, name TEXT, age INT)";
    let result = p.parse(sql).unwrap();
    match result {
        Statement { list } => match &list[0] {
            StatementInner::Ddl(DdlStatement::CreateTable(CreateTable::Table {
                columns, ..
            })) => {
                assert_eq!(columns.len(), 3);
                assert_eq!(columns[0].name, "id");
                assert_eq!(columns[0].col_type, "INT");
                assert_eq!(columns[1].name, "name");
                assert_eq!(columns[1].col_type, "TEXT");
                assert_eq!(columns[2].name, "age");
                assert_eq!(columns[2].col_type, "INT");
            }
            _ => panic!("expected CreateTable"),
        },
    }
}

#[test]
fn test_create_table_not_null() {
    let p = Parser::new().unwrap();
    let result = p.parse("CREATE TABLE t (id INT NOT NULL)").unwrap();
    match result {
        Statement { list } => match &list[0] {
            StatementInner::Ddl(DdlStatement::CreateTable(CreateTable::Table {
                columns, ..
            })) => {
                assert_eq!(columns.len(), 1);
                assert!(columns[0].constraint.not_null);
            }
            _ => panic!("expected CreateTable"),
        },
    }
}

#[test]
fn test_create_table_primary_key() {
    let p = Parser::new().unwrap();
    let result = p.parse("CREATE TABLE t (id INT PRIMARY KEY)").unwrap();
    match result {
        Statement { list } => match &list[0] {
            StatementInner::Ddl(DdlStatement::CreateTable(CreateTable::Table {
                columns, ..
            })) => {
                assert_eq!(columns.len(), 1);
                assert!(columns[0].constraint.primary_key);
            }
            _ => panic!("expected CreateTable"),
        },
    }
}

#[test]
fn test_create_table_unique() {
    let p = Parser::new().unwrap();
    let result = p.parse("CREATE TABLE t (email TEXT UNIQUE)").unwrap();
    match result {
        Statement { list } => match &list[0] {
            StatementInner::Ddl(DdlStatement::CreateTable(CreateTable::Table {
                columns, ..
            })) => {
                assert_eq!(columns.len(), 1);
                assert!(columns[0].constraint.unique);
            }
            _ => panic!("expected CreateTable"),
        },
    }
}

#[test]
fn test_create_table_default_value() {
    let p = Parser::new().unwrap();
    let result = p.parse("CREATE TABLE t (score INT DEFAULT 0)").unwrap();
    match result {
        Statement { list } => match &list[0] {
            StatementInner::Ddl(DdlStatement::CreateTable(CreateTable::Table {
                columns, ..
            })) => {
                assert_eq!(columns.len(), 1);
                assert_eq!(columns[0].constraint.default, Some("0"));
            }
            _ => panic!("expected CreateTable"),
        },
    }
}

#[test]
fn test_create_table_default_string() {
    let p = Parser::new().unwrap();
    let result = p.parse("CREATE TABLE t (name TEXT DEFAULT 'Alice')").unwrap();
    match result {
        Statement { list } => match &list[0] {
            StatementInner::Ddl(DdlStatement::CreateTable(CreateTable::Table {
                columns, ..
            })) => {
                assert_eq!(columns.len(), 1);
                assert_eq!(columns[0].constraint.default, Some("'Alice'"));
            }
            _ => panic!("expected CreateTable"),
        },
    }
}

#[test]
fn test_create_table_all_constraints() {
    let p = Parser::new().unwrap();
    let sql = "\
CREATE TABLE products (\
    id INT PRIMARY KEY NOT NULL,\
    name TEXT NOT NULL UNIQUE,\
    price DECIMAL DEFAULT 0,\
    active BOOLEAN DEFAULT TRUE\
)";
    let result = p.parse(sql).unwrap();
    match result {
        Statement { list } => match &list[0] {
            StatementInner::Ddl(DdlStatement::CreateTable(CreateTable::Table {
                columns, ..
            })) => {
                assert_eq!(columns.len(), 4);
                // id: INT PRIMARY KEY NOT NULL
                assert!(columns[0].constraint.primary_key);
                assert!(columns[0].constraint.not_null);
                // name: TEXT NOT NULL UNIQUE
                assert!(columns[1].constraint.not_null);
                assert!(columns[1].constraint.unique);
                // price: DECIMAL DEFAULT 0
                assert_eq!(columns[2].constraint.default, Some("0"));
                // active: BOOLEAN DEFAULT TRUE
                assert_eq!(columns[3].constraint.default, Some("TRUE"));
            }
            _ => panic!("expected CreateTable"),
        },
    }
}

#[test]
fn test_create_table_varchar_type() {
    let p = Parser::new().unwrap();
    let sql = "CREATE TABLE t (name VARCHAR(100) NOT NULL)";
    let result = p.parse(sql).unwrap();
    match result {
        Statement { list } => match &list[0] {
            StatementInner::Ddl(DdlStatement::CreateTable(CreateTable::Table {
                columns, ..
            })) => {
                assert_eq!(columns.len(), 1);
                assert_eq!(columns[0].name, "name");
                assert_eq!(columns[0].col_type, "VARCHAR");
                assert!(columns[0].col_type_params.is_some());
                assert!(columns[0].constraint.not_null);
            }
            _ => panic!("expected CreateTable"),
        },
    }
}

#[test]
fn test_create_table_as_select() {
    let p = Parser::new().unwrap();
    let result = p.parse("CREATE TABLE t AS SELECT * FROM s").unwrap();
    match result {
        Statement { list } => match &list[0] {
            StatementInner::Ddl(DdlStatement::CreateTable(CreateTable::AsSelect {
                name, ..
            })) => {
                assert_eq!(*name, "t");
            }
            _ => panic!("expected CreateTable::AsSelect"),
        },
    }
}

// ============================================================================
// DROP TABLE
// ============================================================================

#[test]
fn test_drop_table_basic() {
    let p = Parser::new().unwrap();
    let result = p.parse("DROP TABLE t").unwrap();
    assert_eq!(
        result,
        Statement {
            list: vec![StatementInner::Ddl(DdlStatement::DropTable(DropTable {
                if_exists: false,
                names: vec!["t"],
                cascade: false,
            }))]
        }
    );
}

#[test]
fn test_drop_table_if_exists() {
    let p = Parser::new().unwrap();
    let result = p.parse("DROP TABLE IF EXISTS t").unwrap();
    match result {
        Statement { list } => match &list[0] {
            StatementInner::Ddl(DdlStatement::DropTable(DropTable {
                if_exists, ..
            })) => {
                assert!(*if_exists);
            }
            _ => panic!("expected DropTable"),
        },
    }
}

#[test]
fn test_drop_table_multiple() {
    let p = Parser::new().unwrap();
    let result = p.parse("DROP TABLE t1, t2, t3").unwrap();
    match result {
        Statement { list } => match &list[0] {
            StatementInner::Ddl(DdlStatement::DropTable(DropTable {
                names, ..
            })) => {
                assert_eq!(names.len(), 3);
                assert_eq!(names[0], "t1");
                assert_eq!(names[1], "t2");
                assert_eq!(names[2], "t3");
            }
            _ => panic!("expected DropTable"),
        },
    }
}

#[test]
fn test_drop_table_cascade() {
    let p = Parser::new().unwrap();
    let result = p.parse("DROP TABLE t CASCADE").unwrap();
    match result {
        Statement { list } => match &list[0] {
            StatementInner::Ddl(DdlStatement::DropTable(DropTable {
                cascade, ..
            })) => {
                assert!(*cascade);
            }
            _ => panic!("expected DropTable"),
        },
    }
}

#[test]
fn test_drop_table_restrict() {
    let p = Parser::new().unwrap();
    let result = p.parse("DROP TABLE t RESTRICT").unwrap();
    match result {
        Statement { list } => match &list[0] {
            StatementInner::Ddl(DdlStatement::DropTable(DropTable {
                cascade, ..
            })) => {
                assert!(!*cascade, "RESTRICT means cascade=false");
            }
            _ => panic!("expected DropTable"),
        },
    }
}

// ============================================================================
// ALTER TABLE
// ============================================================================

#[test]
fn test_alter_table_add_column() {
    let p = Parser::new().unwrap();
    let result = p.parse("ALTER TABLE t ADD COLUMN x INT").unwrap();
    assert_eq!(
        result,
        Statement {
            list: vec![StatementInner::Ddl(DdlStatement::AlterTable(AlterTable {
                name: "t",
                operation: AlterTableOperation::AddColumn {
                    column: ColumnDef {
                        name: "x",
                        col_type: "INT",
                        col_type_params: None,
                        constraint: ColumnConstraint {
                            not_null: false,
                            default: None,
                            primary_key: false,
                            unique: false,
                        },
                    },
                },
            }))]
        }
    );
}

#[test]
fn test_alter_table_add_without_column_keyword() {
    let p = Parser::new().unwrap();
    let result = p.parse("ALTER TABLE t ADD x INT NOT NULL").unwrap();
    match result {
        Statement { list } => match &list[0] {
            StatementInner::Ddl(DdlStatement::AlterTable(AlterTable {
                operation: AlterTableOperation::AddColumn { column },
                ..
            })) => {
                assert_eq!(column.name, "x");
                assert_eq!(column.col_type, "INT");
                assert!(column.constraint.not_null);
            }
            _ => panic!("expected AlterTable"),
        },
    }
}

#[test]
fn test_alter_table_drop_column() {
    let p = Parser::new().unwrap();
    let result = p.parse("ALTER TABLE t DROP COLUMN x").unwrap();
    assert_eq!(
        result,
        Statement {
            list: vec![StatementInner::Ddl(DdlStatement::AlterTable(AlterTable {
                name: "t",
                operation: AlterTableOperation::DropColumn {
                    name: "x",
                    cascade: false,
                },
            }))]
        }
    );
}

#[test]
fn test_alter_table_drop_without_column_keyword() {
    let p = Parser::new().unwrap();
    let result = p.parse("ALTER TABLE t DROP x").unwrap();
    match result {
        Statement { list } => match &list[0] {
            StatementInner::Ddl(DdlStatement::AlterTable(AlterTable {
                operation: AlterTableOperation::DropColumn { name, cascade },
                ..
            })) => {
                assert_eq!(*name, "x");
                assert!(!cascade);
            }
            _ => panic!("expected AlterTable"),
        },
    }
}

#[test]
fn test_alter_table_drop_column_cascade() {
    let p = Parser::new().unwrap();
    let result = p.parse("ALTER TABLE t DROP COLUMN x CASCADE").unwrap();
    match result {
        Statement { list } => match &list[0] {
            StatementInner::Ddl(DdlStatement::AlterTable(AlterTable {
                operation: AlterTableOperation::DropColumn { name, cascade },
                ..
            })) => {
                assert_eq!(*name, "x");
                assert!(*cascade);
            }
            _ => panic!("expected AlterTable"),
        },
    }
}

#[test]
fn test_alter_table_rename_to() {
    let p = Parser::new().unwrap();
    let result = p.parse("ALTER TABLE t RENAME TO t2").unwrap();
    assert_eq!(
        result,
        Statement {
            list: vec![StatementInner::Ddl(DdlStatement::AlterTable(AlterTable {
                name: "t",
                operation: AlterTableOperation::RenameTo("t2"),
            }))]
        }
    );
}

#[test]
fn test_alter_table_rename_column() {
    let p = Parser::new().unwrap();
    let result = p.parse("ALTER TABLE t RENAME COLUMN old TO new").unwrap();
    match result {
        Statement { list } => match &list[0] {
            StatementInner::Ddl(DdlStatement::AlterTable(AlterTable {
                operation: AlterTableOperation::RenameColumn { old, new },
                ..
            })) => {
                assert_eq!(*old, "old");
                assert_eq!(*new, "new");
            }
            _ => panic!("expected AlterTable"),
        },
    }
}

#[test]
fn test_alter_table_rename_without_column_keyword() {
    let p = Parser::new().unwrap();
    let result = p.parse("ALTER TABLE t RENAME old TO new").unwrap();
    match result {
        Statement { list } => match &list[0] {
            StatementInner::Ddl(DdlStatement::AlterTable(AlterTable {
                operation: AlterTableOperation::RenameColumn { old, new },
                ..
            })) => {
                assert_eq!(*old, "old");
                assert_eq!(*new, "new");
            }
            _ => panic!("expected AlterTable"),
        },
    }
}

// ============================================================================
// 错误路径
// ============================================================================

#[test]
fn test_ddl_create_table_no_name() {
    let p = Parser::new().unwrap();
    assert!(p.parse("CREATE TABLE").is_err(), "CREATE TABLE without name should fail");
}

#[test]
fn test_ddl_drop_table_no_name() {
    let p = Parser::new().unwrap();
    assert!(p.parse("DROP TABLE").is_err(), "DROP TABLE without name should fail");
}

#[test]
fn test_ddl_alter_table_no_op() {
    let p = Parser::new().unwrap();
    assert!(p.parse("ALTER TABLE t").is_err(), "ALTER TABLE without operation should fail");
}

// ============================================================================
// 混合语句测试（DDL + DML 在同一输入）
// ============================================================================

#[test]
fn test_ddl_mixed_with_select() {
    let p = Parser::new().unwrap();
    let sql = "CREATE TABLE t (id INT); SELECT * FROM t";
    let result = p.parse(sql).unwrap();
    assert_eq!(result.list.len(), 2, "should parse 2 statements");
    match &result.list[0] {
        StatementInner::Ddl(_) => {} // first is DDL
        _ => panic!("expected DDL as first statement"),
    }
    match &result.list[1] {
        StatementInner::Query(_) => {} // second is SELECT
        _ => panic!("expected Query as second statement"),
    }
}
