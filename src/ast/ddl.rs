use crate::{
    ParserError,
    common::utils::{expect_kind, maybe_kind},
    keyword::Keyword,
    token::{TokenKind, TokenTable},
    SelectStatement,
};

#[derive(Debug, PartialEq, Clone)]
pub struct ColumnConstraint<'a> {
    pub not_null: bool,
    pub default: Option<&'a str>,
    pub primary_key: bool,
    pub unique: bool,
}

#[derive(Debug, PartialEq)]
pub struct ColumnDef<'a> {
    pub name: &'a str,
    pub col_type: &'a str,
    pub col_type_params: Option<&'a str>,
    pub constraint: ColumnConstraint<'a>,
}

#[derive(Debug, PartialEq)]
pub enum CreateTable<'a> {
    Table {
        if_not_exists: bool,
        name: &'a str,
        columns: Vec<ColumnDef<'a>>,
    },
    AsSelect {
        name: &'a str,
        columns: Option<Vec<&'a str>>,
        select: Box<SelectStatement<'a>>,
    },
}

#[derive(Debug, PartialEq)]
pub struct DropTable<'a> {
    pub if_exists: bool,
    pub names: Vec<&'a str>,
    pub cascade: bool,
}

#[derive(Debug, PartialEq)]
pub enum AlterTableOperation<'a> {
    AddColumn {
        column: ColumnDef<'a>,
    },
    DropColumn {
        name: &'a str,
        cascade: bool,
    },
    RenameTo(&'a str),
    RenameColumn {
        old: &'a str,
        new: &'a str,
    },
}

#[derive(Debug, PartialEq)]
pub struct AlterTable<'a> {
    pub name: &'a str,
    pub operation: AlterTableOperation<'a>,
}

#[derive(Debug, PartialEq)]
pub enum DdlStatement<'a> {
    CreateTable(CreateTable<'a>),
    DropTable(DropTable<'a>),
    AlterTable(AlterTable<'a>),
}

impl<'a> DdlStatement<'a> {
    pub(crate) fn build(
        token_table: &TokenTable<'a>,
        cursor: &mut usize,
    ) -> Result<Self, ParserError> {
        match token_table.get_kind(*cursor) {
            Some(TokenKind::Keyword(Keyword::Create)) => {
                Self::parse_create(token_table, cursor)
            }
            Some(TokenKind::Keyword(Keyword::Drop)) => {
                Self::parse_drop(token_table, cursor)
            }
            Some(TokenKind::Keyword(Keyword::Alter)) => {
                Self::parse_alter(token_table, cursor)
            }
            _ => Err(ParserError::SyntaxError(*cursor, *cursor)),
        }
    }

    // ========================================================================
    // CREATE
    // ========================================================================

    fn parse_create(
        token_table: &TokenTable<'a>,
        cursor: &mut usize,
    ) -> Result<Self, ParserError> {
        expect_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Create))?;
        *cursor += 1;

        match token_table.get_kind(*cursor) {
            Some(TokenKind::Keyword(Keyword::Table)) => {
                *cursor += 1;
                Self::parse_create_table(token_table, cursor)
            }
            _ => Err(ParserError::SyntaxError(*cursor, *cursor)),
        }
    }

    fn parse_create_table(
        token_table: &TokenTable<'a>,
        cursor: &mut usize,
    ) -> Result<Self, ParserError> {
        let if_not_exists =
            if let Some(TokenKind::Keyword(Keyword::If)) = token_table.get_kind(*cursor) {
                *cursor += 1;
                expect_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Not))?;
                *cursor += 1;
                expect_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Exists))?;
                *cursor += 1;
                true
            } else {
                false
            };

        let name = match token_table.get_kind(*cursor) {
            Some(TokenKind::Identifier) => token_table.source_at(*cursor),
            _ => return Err(ParserError::SyntaxError(*cursor, *cursor)),
        };
        *cursor += 1;

        match token_table.get_kind(*cursor) {
            // CREATE TABLE name (col_def, ...)
            Some(TokenKind::LeftParen) => {
                *cursor += 1;
                let mut columns = Vec::new();
                loop {
                    match token_table.get_kind(*cursor) {
                        Some(TokenKind::RightParen) => {
                            *cursor += 1;
                            break;
                        }
                        Some(TokenKind::Comma) => {
                            *cursor += 1;
                        }
                        Some(TokenKind::Identifier) => {
                            columns.push(Self::parse_column_def(token_table, cursor)?);
                        }
                        _ => return Err(ParserError::SyntaxError(*cursor, *cursor)),
                    }
                }
                Ok(DdlStatement::CreateTable(CreateTable::Table {
                    if_not_exists,
                    name,
                    columns,
                }))
            }
            // CREATE TABLE name AS SELECT ...
            Some(TokenKind::Keyword(Keyword::As)) => {
                *cursor += 1;
                expect_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Select))?;
                let select = SelectStatement::new(token_table, cursor)?;
                Ok(DdlStatement::CreateTable(CreateTable::AsSelect {
                    name,
                    columns: None,
                    select: Box::new(select),
                }))
            }
            // CREATE TABLE name (col, ...) AS SELECT ...
            Some(TokenKind::Keyword(Keyword::Select)) => {
                let select = SelectStatement::new(token_table, cursor)?;
                Ok(DdlStatement::CreateTable(CreateTable::AsSelect {
                    name,
                    columns: None,
                    select: Box::new(select),
                }))
            }
            _ => Err(ParserError::SyntaxError(*cursor, *cursor)),
        }
    }

    fn parse_column_def(
        token_table: &TokenTable<'a>,
        cursor: &mut usize,
    ) -> Result<ColumnDef<'a>, ParserError> {
        let name = match token_table.get_kind(*cursor) {
            Some(TokenKind::Identifier) => token_table.source_at(*cursor),
            _ => return Err(ParserError::SyntaxError(*cursor, *cursor)),
        };
        *cursor += 1;

        let (col_type, col_type_params) = Self::parse_column_type(token_table, cursor)?;

        let mut constraint = ColumnConstraint {
            not_null: false,
            default: None,
            primary_key: false,
            unique: false,
        };

        loop {
            match token_table.get_kind(*cursor) {
                Some(TokenKind::Keyword(Keyword::Not)) => {
                    *cursor += 1;
                    expect_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Null))?;
                    *cursor += 1;
                    constraint.not_null = true;
                }
                Some(TokenKind::Keyword(Keyword::Default)) => {
                    *cursor += 1;
                    constraint.default = Some(match token_table.get_kind(*cursor) {
                        Some(TokenKind::StringLiteral) => token_table.source_at(*cursor),
                        Some(TokenKind::Number) => token_table.source_at(*cursor),
                        Some(TokenKind::Identifier) => token_table.source_at(*cursor),
                        Some(TokenKind::Keyword(Keyword::True))
                        | Some(TokenKind::Keyword(Keyword::False)) => token_table.source_at(*cursor),
                        Some(TokenKind::Keyword(Keyword::Null)) => token_table.source_at(*cursor),
                        _ => return Err(ParserError::SyntaxError(*cursor, *cursor)),
                    });
                    *cursor += 1;
                }
                Some(TokenKind::Keyword(Keyword::Primary)) => {
                    *cursor += 1;
                    expect_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Key))?;
                    *cursor += 1;
                    constraint.primary_key = true;
                }
                Some(TokenKind::Keyword(Keyword::Unique)) => {
                    *cursor += 1;
                    constraint.unique = true;
                }
                Some(TokenKind::Comma) | Some(TokenKind::RightParen) => {
                    break;
                }
                _ => break,
            }
        }

        Ok(ColumnDef {
            name,
            col_type,
            col_type_params,
            constraint,
        })
    }

    fn parse_column_type(
        token_table: &TokenTable<'a>,
        cursor: &mut usize,
    ) -> Result<(&'a str, Option<&'a str>), ParserError> {
        let col_type = match token_table.get_kind(*cursor) {
            Some(TokenKind::Identifier) => token_table.source_at(*cursor),
            _ => return Err(ParserError::SyntaxError(*cursor, *cursor)),
        };
        *cursor += 1;

        // Check for parameterized type: VARCHAR(100), DECIMAL(10,2)
        // For P0, we skip the params (they're stored as part of the raw SQL, not AST)
        let params = if let Some(TokenKind::LeftParen) = token_table.get_kind(*cursor) {
            *cursor += 1;
            let mut depth = 1u32;
            loop {
                match token_table.get_kind(*cursor) {
                    Some(TokenKind::LeftParen) => { *cursor += 1; depth += 1; }
                    Some(TokenKind::RightParen) => {
                        depth -= 1;
                        *cursor += 1;
                        if depth == 0 { break Some(""); }
                    }
                    Some(_) => { *cursor += 1; }
                    None => return Err(ParserError::SyntaxError(*cursor, *cursor)),
                }
            }
        } else {
            None
        };

        Ok((col_type, params))
    }

    // ========================================================================
    // DROP
    // ========================================================================

    fn parse_drop(
        token_table: &TokenTable<'a>,
        cursor: &mut usize,
    ) -> Result<Self, ParserError> {
        expect_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Drop))?;
        *cursor += 1;

        match token_table.get_kind(*cursor) {
            Some(TokenKind::Keyword(Keyword::Table)) => {
                *cursor += 1;
                Self::parse_drop_table(token_table, cursor)
            }
            _ => Err(ParserError::SyntaxError(*cursor, *cursor)),
        }
    }

    fn parse_drop_table(
        token_table: &TokenTable<'a>,
        cursor: &mut usize,
    ) -> Result<Self, ParserError> {
        let if_exists =
            if let Some(TokenKind::Keyword(Keyword::If)) = token_table.get_kind(*cursor) {
                *cursor += 1;
                expect_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Exists))?;
                *cursor += 1;
                true
            } else {
                false
            };

        let mut names = Vec::new();
        loop {
            match token_table.get_kind(*cursor) {
                Some(TokenKind::Identifier) => {
                    names.push(token_table.source_at(*cursor));
                    *cursor += 1;
                }
                Some(TokenKind::Comma) => {
                    *cursor += 1;
                }
                _ => break,
            }
        }

        if names.is_empty() {
            return Err(ParserError::SyntaxError(*cursor, *cursor));
        }

        let cascade = if let Some(TokenKind::Keyword(Keyword::Cascade)) =
            token_table.get_kind(*cursor)
        {
            *cursor += 1;
            true
        } else if let Some(TokenKind::Keyword(Keyword::Restrict)) =
            token_table.get_kind(*cursor)
        {
            *cursor += 1;
            false
        } else {
            false
        };

        Ok(DdlStatement::DropTable(DropTable {
            if_exists,
            names,
            cascade,
        }))
    }

    // ========================================================================
    // ALTER
    // ========================================================================

    fn parse_alter(
        token_table: &TokenTable<'a>,
        cursor: &mut usize,
    ) -> Result<Self, ParserError> {
        expect_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Alter))?;
        *cursor += 1;

        match token_table.get_kind(*cursor) {
            Some(TokenKind::Keyword(Keyword::Table)) => {
                *cursor += 1;
                Self::parse_alter_table(token_table, cursor)
            }
            _ => Err(ParserError::SyntaxError(*cursor, *cursor)),
        }
    }

    fn parse_alter_table(
        token_table: &TokenTable<'a>,
        cursor: &mut usize,
    ) -> Result<Self, ParserError> {
        let name = match token_table.get_kind(*cursor) {
            Some(TokenKind::Identifier) => token_table.source_at(*cursor),
            _ => return Err(ParserError::SyntaxError(*cursor, *cursor)),
        };
        *cursor += 1;

        match token_table.get_kind(*cursor) {
            Some(TokenKind::Keyword(Keyword::Add)) => {
                *cursor += 1;
                // OPTIONAL COLUMN keyword
                if let Some(TokenKind::Keyword(Keyword::Column)) = token_table.get_kind(*cursor) {
                    *cursor += 1;
                }
                let column = Self::parse_column_def(token_table, cursor)?;
                Ok(DdlStatement::AlterTable(AlterTable {
                    name,
                    operation: AlterTableOperation::AddColumn { column },
                }))
            }
            Some(TokenKind::Keyword(Keyword::Drop)) => {
                *cursor += 1;
                // OPTIONAL COLUMN keyword
                if let Some(TokenKind::Keyword(Keyword::Column)) = token_table.get_kind(*cursor) {
                    *cursor += 1;
                }
                let col_name = match token_table.get_kind(*cursor) {
                    Some(TokenKind::Identifier) => token_table.source_at(*cursor),
                    _ => return Err(ParserError::SyntaxError(*cursor, *cursor)),
                };
                *cursor += 1;

                let cascade =
                    if let Some(TokenKind::Keyword(Keyword::Cascade)) = token_table.get_kind(*cursor)
                    {
                        *cursor += 1;
                        true
                    } else if let Some(TokenKind::Keyword(Keyword::Restrict)) =
                        token_table.get_kind(*cursor)
                    {
                        *cursor += 1;
                        false
                    } else {
                        false
                    };

                Ok(DdlStatement::AlterTable(AlterTable {
                    name,
                    operation: AlterTableOperation::DropColumn {
                        name: col_name,
                        cascade,
                    },
                }))
            }
            Some(TokenKind::Keyword(Keyword::Rename)) => {
                *cursor += 1;
                // RENAME TO new_name or RENAME [COLUMN] old_name TO new_name
                let opt_column =
                    if let Some(TokenKind::Keyword(Keyword::Column)) = token_table.get_kind(*cursor)
                    {
                        *cursor += 1;
                        true
                    } else {
                        false
                    };

                match token_table.get_kind(*cursor) {
                    Some(TokenKind::Keyword(Keyword::To)) => {
                        // RENAME TO new_name (COLUMN keyword not allowed here)
                        if opt_column {
                            return Err(ParserError::SyntaxError(*cursor, *cursor));
                        }
                        *cursor += 1;
                        let new_name = match token_table.get_kind(*cursor) {
                            Some(TokenKind::Identifier) => token_table.source_at(*cursor),
                            _ => return Err(ParserError::SyntaxError(*cursor, *cursor)),
                        };
                        *cursor += 1;
                        Ok(DdlStatement::AlterTable(AlterTable {
                            name,
                            operation: AlterTableOperation::RenameTo(new_name),
                        }))
                    }
                    Some(TokenKind::Identifier) if !opt_column => {
                        // Ambiguous: could be RENAME old TO new or RENAME TO old (syntax error)
                        // Check if next token is TO
                        let first = token_table.source_at(*cursor);
                        *cursor += 1;
                        if maybe_kind(token_table, cursor, &TokenKind::Keyword(Keyword::To)) {
                            *cursor += 1;
                            let new = match token_table.get_kind(*cursor) {
                                Some(TokenKind::Identifier) => token_table.source_at(*cursor),
                                _ => return Err(ParserError::SyntaxError(*cursor, *cursor)),
                            };
                            *cursor += 1;
                            Ok(DdlStatement::AlterTable(AlterTable {
                                name,
                                operation: AlterTableOperation::RenameColumn {
                                    old: first,
                                    new,
                                },
                            }))
                        } else {
                            Err(ParserError::SyntaxError(*cursor, *cursor))
                        }
                    }
                    Some(TokenKind::Identifier) => {
                        // RENAME COLUMN old_name TO new_name
                        let old = token_table.source_at(*cursor);
                        *cursor += 1;
                        expect_kind(token_table, cursor, &TokenKind::Keyword(Keyword::To))?;
                        *cursor += 1;
                        let new = match token_table.get_kind(*cursor) {
                            Some(TokenKind::Identifier) => token_table.source_at(*cursor),
                            _ => return Err(ParserError::SyntaxError(*cursor, *cursor)),
                        };
                        *cursor += 1;
                        Ok(DdlStatement::AlterTable(AlterTable {
                            name,
                            operation: AlterTableOperation::RenameColumn { old, new },
                        }))
                    }
                    _ => Err(ParserError::SyntaxError(*cursor, *cursor)),
                }
            }
            _ => Err(ParserError::SyntaxError(*cursor, *cursor)),
        }
    }
}
