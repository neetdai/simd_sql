use std::{alloc::Allocator, borrow::Cow};

use minivec::MiniVec;

use crate::{
    ParserError,
    keyword::Keyword,
    token::{self, TokenKind, TokenTable},
};

pub(crate) fn expect_kind(
    token_table: &TokenTable,
    cursor: &usize,
    token_kind: &TokenKind,
) -> Result<(), ParserError> {
    if let Some(kind) = token_table.get_kind(*cursor) {
        if kind != token_kind {
            return Err(ParserError::UnexpectedToken {
                expected: token_kind.clone(),
                found: kind.clone(),
            });
        }
    }
    Ok(())
}

pub(crate) fn maybe_kind(token_table: &TokenTable, cursor: &usize, token_kind: &TokenKind) -> bool {
    if let Some(kind) = token_table.get_kind(*cursor) {
        kind == token_kind
    } else {
        false
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Mod,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
}

impl BinaryOperator {
    /// 从 TokenKind 创建 BinaryOperator
    pub fn from_token_kind(kind: &TokenKind) -> Option<Self> {
        match kind {
            TokenKind::Plus => Some(BinaryOperator::Add),
            TokenKind::Subtract => Some(BinaryOperator::Subtract),
            TokenKind::Multiply => Some(BinaryOperator::Multiply),
            TokenKind::Divide => Some(BinaryOperator::Divide),
            TokenKind::Mod => Some(BinaryOperator::Mod),
            TokenKind::Equal => Some(BinaryOperator::Equal),
            TokenKind::NotEqual => Some(BinaryOperator::NotEqual),
            TokenKind::Less => Some(BinaryOperator::Less),
            TokenKind::LessEqual => Some(BinaryOperator::LessEqual),
            TokenKind::Greater => Some(BinaryOperator::Greater),
            TokenKind::GreaterEqual => Some(BinaryOperator::GreaterEqual),
            TokenKind::Keyword(Keyword::And) => Some(BinaryOperator::And),
            TokenKind::Keyword(Keyword::Or) => Some(BinaryOperator::Or),
            _ => None,
        }
    }

    /// 获取运算符的优先级，数值越大优先级越高
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOperator::Or => 1,
            BinaryOperator::And => 2,
            BinaryOperator::Equal | BinaryOperator::NotEqual => 3,
            BinaryOperator::Less | BinaryOperator::LessEqual | BinaryOperator::Greater | BinaryOperator::GreaterEqual => 4,
            BinaryOperator::Add | BinaryOperator::Subtract => 5,
            BinaryOperator::Multiply | BinaryOperator::Divide | BinaryOperator::Mod => 6,
        }
    }

    /// 判断是否是左结合的运算符
    pub fn is_left_associative(&self) -> bool {
        true // 所有二元运算符都是左结合的
    }
}

#[derive(Debug, PartialEq)]
pub struct BinaryOp {
    pub op: BinaryOperator,
    pub left: Expr,
    pub right: Expr,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Field(Field),
    Star(Star),
    FunctionCall(FunctionCall),
    StringLiteral(StringLiteral),
    NumbericLiteral(NumbericLiteral),
    BinaryOp(Box<BinaryOp>),
}

impl Expr {
    pub(crate) fn class_field(
        token_table: &TokenTable,
        cursor: &mut usize,
    ) -> Result<Self, ParserError> {
        Field::from_token(token_table, cursor).map(Expr::Field)
    }

    pub(crate) fn class_function_call(
        token_table: &TokenTable,
        cursor: &mut usize,
    ) -> Result<Self, ParserError> {
        FunctionCall::from_token(token_table, cursor).map(Expr::FunctionCall)
    }

    pub(crate) fn class_string_literal(
        token_table: &TokenTable,
        cursor: &mut usize,
    ) -> Result<Self, ParserError> {
        StringLiteral::from_token(token_table, cursor).map(Expr::StringLiteral)
    }

    pub(crate) fn class_number_literal(
        token_table: &TokenTable,
        cursor: &mut usize,
    ) -> Result<Self, ParserError> {
        NumbericLiteral::from_token(token_table, cursor).map(Expr::NumbericLiteral)
    }

    pub(crate) fn class_star(
        token_table: &TokenTable,
        cursor: &mut usize,
    ) -> Result<Self, ParserError> {
        Star::from_token(token_table, cursor).map(Expr::Star)
    }

    /// 使用 Pratt Parser 解析表达式
    pub(crate) fn parse_expression(
        token_table: &TokenTable,
        cursor: &mut usize,
    ) -> Result<Self, ParserError> {
        Self::parse_expression_with_min_precedence(token_table, cursor, 0)
    }

    /// 使用 Pratt Parser 解析表达式，支持指定最小优先级
    fn parse_expression_with_min_precedence(
        token_table: &TokenTable,
        cursor: &mut usize,
        min_precedence: u8,
    ) -> Result<Self, ParserError> {
        // 解析左侧表达式（原子表达式）
        let mut left = Self::parse_primary(token_table, cursor)?;

        // 循环处理二元运算符
        loop {
            // 检查当前 token 是否是二元运算符
            let op = match token_table.get_kind(*cursor).and_then(|kind| BinaryOperator::from_token_kind(kind)) {
                Some(op) => op,
                None => break,
            };

            // 如果运算符优先级低于最小优先级，停止解析
            if op.precedence() < min_precedence {
                break;
            }

            // 消耗运算符 token
            *cursor += 1;

            // 计算下一个表达式的最小优先级
            let next_min_precedence = if op.is_left_associative() {
                op.precedence() + 1
            } else {
                op.precedence()
            };

            // 递归解析右侧表达式
            let right = Self::parse_expression_with_min_precedence(
                token_table,
                cursor,
                next_min_precedence,
            )?;

            // 构建二元运算表达式
            left = Expr::BinaryOp(Box::new(BinaryOp {
                op,
                left,
                right,
            }));
        }

        Ok(left)
    }

    /// 解析原子表达式（最基础的表达式）
    fn parse_primary(
        token_table: &TokenTable,
        cursor: &mut usize,
    ) -> Result<Self, ParserError> {
        match token_table.get_kind(*cursor) {
            Some(TokenKind::Number) => Self::class_number_literal(token_table, cursor),
            Some(TokenKind::StringLiteral) => Self::class_string_literal(token_table, cursor),
            Some(TokenKind::Identifier) => {
                // 检查是否是函数调用
                if let Some(TokenKind::LeftParen) = token_table.get_kind(*cursor + 1) {
                    Self::class_function_call(token_table, cursor)
                } else {
                    if let Ok(star) = Self::class_star(token_table, cursor) {
                        return Ok(star);
                    } else {
                        Self::class_field(token_table, cursor)
                    }
                }
            }
            Some(TokenKind::Multiply) => {
                Self::class_star(token_table, cursor)
            },
            Some(TokenKind::LeftParen) => {
                // 处理括号表达式
                *cursor += 1;
                let expr = Self::parse_expression(token_table, cursor)?;
                expect_kind(token_table, cursor, &TokenKind::RightParen)?;
                *cursor += 1;
                Ok(expr)
            }
            _ => Err(ParserError::SyntaxError(*cursor, *cursor)),
        }
    }

    pub(crate) fn build(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> {
        Self::parse_expression(token_table, cursor)
    }
}

trait FromToken {
    fn from_token(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError>
    where
        Self: Sized;
}

#[derive(Debug, PartialEq)]
pub struct Alias {
    name: Option<usize>,
    value: Expr,
}

impl Alias {
    pub(crate) fn new(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> {
        let value = Expr::build(token_table, cursor)?;
        match token_table.get_kind(*cursor) {
            Some(TokenKind::Keyword(Keyword::As)) => {
                *cursor += 1;
                if let Some(TokenKind::Identifier) = token_table.get_kind(*cursor) {
                    let name = *cursor;
                    *cursor += 1;
                    Ok(Alias {
                        name: Some(name),
                        value,
                    })
                } else {
                    Err(ParserError::SyntaxError(*cursor, *cursor))
                }
            }
            Some(TokenKind::Identifier) => {
                if let Some(TokenKind::Identifier) = token_table.get_kind(*cursor) {
                    let name = *cursor;
                    *cursor += 1;
                    Ok(Alias {
                        name: Some(name),
                        value,
                    })
                } else {
                    Ok(Alias { name: None, value })
                }
            }
            _ => Ok(Alias { name: None, value }),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Field {
    prefix: Option<usize>,
    value: usize,
}

impl FromToken for Field {
    fn from_token(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError>
    where
        Self: Sized,
    {
        let first = token_table
            .get_kind(*cursor)
            .map(|kind| kind == &TokenKind::Identifier)
            .unwrap_or(false);
        let dot = token_table
            .get_kind(*cursor + 1)
            .map(|kind| kind == &TokenKind::Dot)
            .unwrap_or(false);
        let second = token_table
            .get_kind(*cursor + 2)
            .map(|kind| kind == &TokenKind::Identifier)
            .unwrap_or(false);

        let sum = (first as usize) + (dot as usize) + (second as usize);

        let (prefix, value) = match (first, sum) {
            (true, 1) => (None, *cursor),
            (true, 3) => (Some(*cursor), *cursor + 2),
            _ => return Err(ParserError::SyntaxError(*cursor, *cursor)),
        };

        *cursor += sum;

        Ok(Self { prefix, value })

        // match token_table.get_entry(*cursor) {
        //     Some((TokenKind::Identifier, (current_start, current_end))) => {
        //         *cursor += 1;
        //         if let Some([TokenKind::Dot, TokenKind::Identifier]) = token_table.get_kind(*cursor..=*cursor + 1) {
        //             *cursor += 1;
        //             let (start, end) = token_table.get_position(*cursor).unwrap();
        //             *cursor += 1;
        //             Ok(Self { prefix: Some((*current_start, *current_end)), value: (*start, *end) })
        //         } else {
        //             Ok(Self { prefix: None, value: (*current_start, *current_end) })
        //         }
        //     }
        //     _ => Err(ParserError::SyntaxError(*cursor, *cursor)),
        // }
    }
}

#[derive(Debug, PartialEq)]
pub struct Star {
    pub prefix: Option<usize>,
}

impl FromToken for Star {
    fn from_token(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError>
    where
        Self: Sized,
    {
        let first = token_table
            .get_kind(*cursor)
            .map(|kind| kind == &TokenKind::Identifier)
            .unwrap_or(false);
        let first_star = token_table
            .get_kind(*cursor)
            .map(|kind| kind == &TokenKind::Multiply)
            .unwrap_or(false);
        let dot = token_table
            .get_kind(*cursor + 1)
            .map(|kind| kind == &TokenKind::Dot)
            .unwrap_or(false);
        let second = token_table
            .get_kind(*cursor + 2)
            .map(|kind| kind == &TokenKind::Multiply)
            .unwrap_or(false);

        let sum = ((first || first_star) as usize) + (dot as usize) + (second as usize);

        let prefix = match (first_star, first, sum) {
            (true, false, 1) => None,
            (false, true, 3) => Some(*cursor),
            _ => return Err(ParserError::SyntaxError(*cursor, *cursor)),
        };

        *cursor += sum;

        Ok(Self { prefix })
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionCall {
    name: usize,
    args: MiniVec<Expr>,
}

impl FromToken for FunctionCall {
    fn from_token(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError>
    where
        Self: Sized,
    {
        let first = token_table
            .get_kind(*cursor)
            .map(|kind| kind == &TokenKind::Identifier)
            .unwrap_or(false);
        let second = token_table
            .get_kind(*cursor + 1)
            .map(|kind| kind == &TokenKind::LeftParen)
            .unwrap_or(false);

        if !(first && second) {
            return Err(ParserError::SyntaxError(*cursor, *cursor));
        }

        let name_pos = *cursor;
        *cursor += 2;

        let mut args = MiniVec::with_capacity(8);
        loop {
            let expr = Expr::build(token_table, cursor)?;
            args.push(expr);

            match token_table.get_kind(*cursor) {
                Some(TokenKind::Comma) => {
                    *cursor += 1;
                    continue;
                }
                Some(TokenKind::RightParen) => {
                    *cursor += 1;
                    break;
                }
                _ => {
                    return Err(ParserError::SyntaxError(*cursor, *cursor));
                }
            }
        }

        Ok(Self {
            name: name_pos,
            args,
        })

        // if let Some((TokenKind::Identifier, (start, end))) = token_table.get_entry(*cursor) {
        //     *cursor += 1;
        //     let name = (*start, *end);

        //     expect_kind(token_table, cursor, &TokenKind::LeftParen)?;
        //     *cursor += 1;

        //     let mut args = MiniVec::with_capacity(8);
        //     loop {
        //         let expr = Expr::build(token_table, cursor)?;
        //         args.push(expr);

        //         if let Some(TokenKind::Comma) = token_table.get_kind(*cursor) {
        //             *cursor += 1;
        //             continue;
        //         } else if let Some(TokenKind::RightParen) = token_table.get_kind(*cursor) {
        //             *cursor += 1;
        //             break;
        //         } else {
        //             return Err(ParserError::SyntaxError(*cursor, *cursor));
        //         }
        //     }

        //     Ok(Self {
        //         name: name,
        //         args,
        //     })
        // } else {
        //     return Err(ParserError::SyntaxError(*cursor, *cursor));
        // }
    }
}

#[derive(Debug, PartialEq)]
pub struct StringLiteral {
    value: usize,
}

impl FromToken for StringLiteral {
    fn from_token(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError>
    where
        Self: Sized,
    {
        if let Some(TokenKind::StringLiteral) = token_table.get_kind(*cursor) {
            let value = *cursor;
            *cursor += 1;
            Ok(Self { value })
        } else {
            Err(ParserError::SyntaxError(*cursor, *cursor))
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct NumbericLiteral {
    value: usize,
}

impl FromToken for NumbericLiteral {
    fn from_token(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError>
    where
        Self: Sized,
    {
        if let Some(TokenKind::Number) = token_table.get_kind(*cursor) {
            let value = *cursor;
            *cursor += 1;
            Ok(Self { value })
        } else {
            Err(ParserError::SyntaxError(*cursor, *cursor))
        }
    }
}

#[cfg(test)]
mod test {
    use minivec::mini_vec;

    use crate::{
        Expr, ParserError,
        common::{Alias, BinaryOp, BinaryOperator, Field, FunctionCall, StringLiteral},
        keyword::Keyword,
        token::{TokenKind, TokenTable},
    };

    #[test]
    fn test_column() {
        let mut token_table = TokenTable::with_capacity(3);
        token_table.push(TokenKind::Identifier, 0, 1); // prefix
        token_table.push(TokenKind::Dot, 2, 2);
        token_table.push(TokenKind::Identifier, 3, 4); // value

        let mut cursor = 0;
        let expr = Expr::class_field(&token_table, &mut cursor).unwrap();
        assert_eq!(
            expr,
            Expr::Field(Field {
                prefix: Some(0),
                value: 2,
            })
        );
        assert_eq!(cursor, 3);

        let mut token_table = TokenTable::with_capacity(3);
        token_table.push(TokenKind::Identifier, 0, 1); // prefix
        token_table.push(TokenKind::Dot, 2, 2);
        token_table.push(TokenKind::Identifier, 3, 4); // value
        token_table.push(TokenKind::Keyword(Keyword::As), 5, 6); // As
        token_table.push(TokenKind::Identifier, 7, 8); // alias

        let mut cursor = 0;
        let expr = Alias::new(&token_table, &mut cursor).unwrap();
        assert_eq!(
            expr,
            Alias {
                name: Some(4),
                value: Expr::Field(Field {
                    prefix: Some(0),
                    value: 2,
                })
            }
        );
        assert_eq!(cursor, 5);

        let mut token_table = TokenTable::with_capacity(3);
        token_table.push(TokenKind::Identifier, 0, 1); // prefix
        token_table.push(TokenKind::Dot, 2, 2);
        token_table.push(TokenKind::Identifier, 3, 4); // value
        token_table.push(TokenKind::Identifier, 5, 6); // alias

        let mut cursor = 0;
        let expr = Alias::new(&token_table, &mut cursor).unwrap();
        assert_eq!(
            expr,
            Alias {
                name: Some(3),
                value: Expr::Field(Field {
                    prefix: Some(0),
                    value: 2,
                })
            }
        );
        assert_eq!(cursor, 4);
    }

    #[test]
    fn test_function() {
        let mut token_table = TokenTable::with_capacity(3);
        token_table.push(TokenKind::Identifier, 0, 1); // name
        token_table.push(TokenKind::LeftParen, 2, 2);
        token_table.push(TokenKind::StringLiteral, 3, 5);
        token_table.push(TokenKind::RightParen, 6, 6); // args

        let mut cursor = 0;
        let expr = Expr::class_function_call(&token_table, &mut cursor).unwrap();
        assert_eq!(
            expr,
            Expr::FunctionCall(FunctionCall {
                    name: 0,
                    args: mini_vec![Expr::StringLiteral(StringLiteral { value: 2 })]
                
            })
        );
        assert_eq!(cursor, 4);
    }

    #[test]
    fn test_function_1() {
        let mut token_table = TokenTable::with_capacity(3);
        token_table.push(TokenKind::Identifier, 0, 1); // name
        token_table.push(TokenKind::LeftParen, 2, 2);
        token_table.push(TokenKind::StringLiteral, 3, 5);
        token_table.push(TokenKind::Comma, 6, 6);
        token_table.push(TokenKind::StringLiteral, 7, 8);
        token_table.push(TokenKind::RightParen, 9, 9); // args

        let mut cursor = 0;
        let expr = Expr::class_function_call(&token_table, &mut cursor).unwrap();
        assert_eq!(
            expr,
            Expr::FunctionCall(FunctionCall {
                    name: 0,
                    args: mini_vec![
                        Expr::StringLiteral(StringLiteral { value: 2 }),
                        Expr::StringLiteral(StringLiteral { value: 4 })
                    ]
            })
        );
        assert_eq!(cursor, 6);
    }


    #[test]
    fn test_function_should_panic_1() {
        let mut token_table = TokenTable::with_capacity(3);
        token_table.push(TokenKind::Identifier, 0, 1); // name
        token_table.push(TokenKind::LeftParen, 2, 2);
        token_table.push(TokenKind::StringLiteral, 3, 5);
        token_table.push(TokenKind::Comma, 6, 6);
        token_table.push(TokenKind::RightParen, 7, 7); // args

        let mut cursor = 0;
        let expr = Expr::class_function_call(&token_table, &mut cursor);
        assert_eq!(expr, Err(ParserError::SyntaxError(4, 4)));
    }

    #[test]
    fn test_binary_operator_1() {
        let mut token_table = TokenTable::with_capacity(7);
        token_table.push(TokenKind::Number, 0, 0); // left
        token_table.push(TokenKind::Plus, 1, 1); // +
        token_table.push(TokenKind::Number, 2, 2); // right
        token_table.push(TokenKind::Multiply, 3, 3); // *
        token_table.push(TokenKind::Number, 4, 4); // right

        let mut cursor = 0;
        let expr = Expr::build(&token_table, &mut cursor).unwrap();
        assert_eq!(
            expr,
            Expr::BinaryOp(Box::new(BinaryOp {
                op: BinaryOperator::Add,
                left: Expr::NumbericLiteral(crate::common::NumbericLiteral { value: 0 }),
                right: Expr::BinaryOp(Box::new(BinaryOp {
                    op: BinaryOperator::Multiply,
                    left: Expr::NumbericLiteral(crate::common::NumbericLiteral { value: 2 }),
                    right: Expr::NumbericLiteral(crate::common::NumbericLiteral { value: 4 }),
                })),
            }))
        );
        assert_eq!(cursor, 5);
    }

    #[test]
    fn test_build_binary_op_2() {
        let mut token_table = TokenTable::with_capacity(7);
        token_table.push(TokenKind::Number, 0, 0); // left
        token_table.push(TokenKind::Multiply, 1, 1); // +
        token_table.push(TokenKind::Number, 2, 2); // right
        token_table.push(TokenKind::Plus, 3, 3); // *
        token_table.push(TokenKind::Number, 4, 4); // right

        let mut cursor = 0;
        let expr = Expr::build(&token_table, &mut cursor).unwrap();
        assert_eq!(
            expr,
            Expr::BinaryOp(Box::new(BinaryOp {
                op: BinaryOperator::Add,
                left: Expr::BinaryOp(Box::new(BinaryOp {
                    op: BinaryOperator::Multiply,
                    left: Expr::NumbericLiteral(crate::common::NumbericLiteral { value: 0 }),
                    right: Expr::NumbericLiteral(crate::common::NumbericLiteral { value: 2 }),
                })),
                right: Expr::NumbericLiteral(crate::common::NumbericLiteral { value: 4 }),
            }))
        );
        assert_eq!(cursor, 5);
    }

    #[test]
    fn test_build_binary_op_3() {
        let mut token_table = TokenTable::with_capacity(7);
        token_table.push(TokenKind::LeftParen, 0, 0); // left
        token_table.push(TokenKind::Number, 1, 1); // left
        token_table.push(TokenKind::Plus, 2, 2); // +
        token_table.push(TokenKind::Number, 3, 3); // right
        token_table.push(TokenKind::RightParen, 4, 4); // right
        token_table.push(TokenKind::Multiply, 5, 5); // *
        token_table.push(TokenKind::Number, 6, 6); // right

        let mut cursor = 0;
        let expr = Expr::build(&token_table, &mut cursor).unwrap();
        assert_eq!(
            expr,
            Expr::BinaryOp(Box::new(BinaryOp {
                op: BinaryOperator::Multiply,
                left: Expr::BinaryOp(Box::new(BinaryOp {
                    op: BinaryOperator::Add,
                    left: Expr::NumbericLiteral(crate::common::NumbericLiteral { value: 1 }),
                    right: Expr::NumbericLiteral(crate::common::NumbericLiteral { value: 3 }),
                })),
                right: Expr::NumbericLiteral(crate::common::NumbericLiteral { value: 6 }),
            }))
        );
        assert_eq!(cursor, 7);
    }
}
