use minivec::MiniVec;

use crate::{
    common::{
        alias::Aliasable,
        pratt_parser::{PrattOutput, PrattParser, PrattParserTrait, PrecedenceTrait},
        utils::{expect_kind, maybe_kind},
    },
    keyword::Keyword,
    token::{TokenKind, TokenTable},
    ParserError,
};

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
}

impl PrecedenceTrait for BinaryOperator {
    /// 获取运算符的优先级，数值越大优先级越高
    fn precedence(&self) -> usize {
        match self {
            BinaryOperator::Or => 1,
            BinaryOperator::And => 2,
            BinaryOperator::Equal | BinaryOperator::NotEqual => 3,
            BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual => 4,
            BinaryOperator::Add | BinaryOperator::Subtract => 5,
            BinaryOperator::Multiply | BinaryOperator::Divide | BinaryOperator::Mod => 6,
        }
    }

    /// 判断是否是左结合的运算符
    fn is_left_associative(&self) -> bool {
        true // 所有二元运算符都是左结合的
    }

    fn min_precedence() -> usize {
        0
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
    Between(Between),
    In(In),
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

    pub(crate) fn class_between(
        field: Box<Expr>,
        token_table: &TokenTable,
        cursor: &mut usize,
    ) -> Result<Self, ParserError> {
        Between::build(field, token_table, cursor).map(Expr::Between)
    }

    pub(crate) fn class_in(
        field: Box<Expr>,
        token_table: &TokenTable,
        cursor: &mut usize,
    ) -> Result<Self, ParserError> {
        In::build(field, token_table, cursor).map(Expr::In)
    }

    /// 使用 Pratt Parser 解析表达式
    pub(crate) fn parse_expression(
        token_table: &TokenTable,
        cursor: &mut usize,
    ) -> Result<Self, ParserError> {
        PrattParser::parse_expression::<Self>(token_table, cursor)
    }

    pub(crate) fn build(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> {
        Self::parse_expression(token_table, cursor)
    }
}

impl Aliasable for Expr {
    fn aliasable(token_table: &TokenTable, cursor: &mut usize) -> Result<Self, ParserError> {
        Self::build(token_table, cursor)
    }
}

impl PrattOutput<BinaryOperator> for Expr {
    fn apply(op: BinaryOperator, left: Self, right: Self) -> Self {
        Expr::BinaryOp(Box::new(BinaryOp { op, left, right }))
    }
}

impl PrattParserTrait for Expr {
    type Item = BinaryOperator;
    type Output = Self;

    fn parse_primary(
        token_table: &TokenTable,
        cursor: &mut usize,
    ) -> Result<Self::Output, ParserError> {
        match token_table.get_kind(*cursor) {
            Some(TokenKind::Number) => Self::class_number_literal(token_table, cursor),
            Some(TokenKind::StringLiteral) => Self::class_string_literal(token_table, cursor),
            Some(TokenKind::Identifier) => {
                // 检查是否是函数调用
                if let Some(TokenKind::LeftParen) = token_table.get_kind(*cursor + 1) {
                    Self::class_function_call(token_table, cursor)
                } else if let Ok(star) = Self::class_star(token_table, cursor) {
                    Ok(star)
                } else {
                    let field = Self::class_field(token_table, cursor)?;

                    if maybe_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Between)) {
                        Self::class_between(Box::new(field), token_table, cursor)
                    } else if maybe_kind(token_table, cursor, &TokenKind::Keyword(Keyword::In)) {
                        Self::class_in(Box::new(field), token_table, cursor)
                    } else {
                        Ok(field)
                    }
                }
            }
            Some(TokenKind::Multiply) => Self::class_star(token_table, cursor),
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

    fn match_item(token_kind: &TokenKind) -> Option<Self::Item> {
        match token_kind {
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
    pub(crate) prefix: Option<usize>,
    pub(crate) value: usize,
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

        let (prefix, value, sum) = match (first, dot, second) {
            (true, false, _) => (None, *cursor, 1),
            (true, true, true) => (Some(*cursor), *cursor + 2, 3),
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

        if first_star {
            *cursor += 1;
            Ok(Self { prefix: None })
        } else if first && dot && second {
            let prefix = *cursor;
            *cursor += 3;
            Ok(Self {
                prefix: Some(prefix),
            })
        } else {
            Err(ParserError::SyntaxError(*cursor, *cursor))
        }
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

#[derive(Debug, PartialEq)]
pub struct Between {
    field: Box<Expr>,
    lower: Box<Expr>,
    upper: Box<Expr>,
}

impl Between {
    pub(crate) fn build(
        field: Box<Expr>,
        token_table: &TokenTable,
        cursor: &mut usize,
    ) -> Result<Self, ParserError> {
        expect_kind(token_table, cursor, &TokenKind::Keyword(Keyword::Between))?;
        *cursor += 1;

        let lower = Box::new(Expr::build(token_table, cursor)?);

        expect_kind(token_table, cursor, &TokenKind::Keyword(Keyword::And))?;
        *cursor += 1;

        let upper = Box::new(Expr::build(token_table, cursor)?);

        Ok(Self {
            field,
            lower,
            upper,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct In {
    field: Box<Expr>,
    values: MiniVec<Expr>,
}

impl In {
    pub(crate) fn build(
        field: Box<Expr>,
        token_table: &TokenTable,
        cursor: &mut usize,
    ) -> Result<Self, ParserError> {

        expect_kind(token_table, cursor, &TokenKind::Keyword(Keyword::In))?;
        *cursor += 1;

        expect_kind(token_table, cursor, &TokenKind::LeftParen)?;
        *cursor += 1;

        let mut values = MiniVec::with_capacity(8);
        loop {
            token_table.get_kind(*cursor);
            let expr = Expr::build(token_table, cursor)?;
            values.push(expr);

            match token_table.get_kind(*cursor) {
                Some(TokenKind::Comma) => {
                    *cursor += 1;
                    continue;
                }
                Some(TokenKind::RightParen) => {
                    *cursor += 1;
                    break;
                }
                Some(TokenKind::Keyword(_)) => {
                    break;
                }
                Some(_) => {
                    continue;
                }
                None => {
                    return Err(ParserError::SyntaxError(*cursor, *cursor));
                }
            }
        }

        Ok(Self { field, values })
    }
}

#[cfg(test)]
mod test {
    use minivec::mini_vec;

    use crate::{
        common::{
            alias::Alias,
            expr::{
                Between, BinaryOp, BinaryOperator, Expr, Field, FunctionCall, In, NumbericLiteral,
                Star, StringLiteral,
            },
        },
        keyword::Keyword,
        token::{TokenKind, TokenTable},
        ParserError,
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
                left: Expr::NumbericLiteral(NumbericLiteral { value: 0 }),
                right: Expr::BinaryOp(Box::new(BinaryOp {
                    op: BinaryOperator::Multiply,
                    left: Expr::NumbericLiteral(NumbericLiteral { value: 2 }),
                    right: Expr::NumbericLiteral(NumbericLiteral { value: 4 }),
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
                    left: Expr::NumbericLiteral(NumbericLiteral { value: 0 }),
                    right: Expr::NumbericLiteral(NumbericLiteral { value: 2 }),
                })),
                right: Expr::NumbericLiteral(NumbericLiteral { value: 4 }),
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
                    left: Expr::NumbericLiteral(NumbericLiteral { value: 1 }),
                    right: Expr::NumbericLiteral(NumbericLiteral { value: 3 }),
                })),
                right: Expr::NumbericLiteral(NumbericLiteral { value: 6 }),
            }))
        );
        assert_eq!(cursor, 7);
    }

    #[test]
    fn test_binary_operator_all_arithmetic() {
        let mut token_table = TokenTable::with_capacity(3);
        token_table.push(TokenKind::Number, 0, 0);
        token_table.push(TokenKind::Divide, 1, 1);
        token_table.push(TokenKind::Number, 2, 2);

        let mut cursor = 0;
        let expr = Expr::build(&token_table, &mut cursor).unwrap();
        assert_eq!(
            expr,
            Expr::BinaryOp(Box::new(BinaryOp {
                op: BinaryOperator::Divide,
                left: Expr::NumbericLiteral(NumbericLiteral { value: 0 }),
                right: Expr::NumbericLiteral(NumbericLiteral { value: 2 }),
            }))
        );
        assert_eq!(cursor, 3);

        let mut token_table = TokenTable::with_capacity(3);
        token_table.push(TokenKind::Number, 0, 0);
        token_table.push(TokenKind::Mod, 1, 1);
        token_table.push(TokenKind::Number, 2, 2);

        let mut cursor = 0;
        let expr = Expr::build(&token_table, &mut cursor).unwrap();
        assert_eq!(
            expr,
            Expr::BinaryOp(Box::new(BinaryOp {
                op: BinaryOperator::Mod,
                left: Expr::NumbericLiteral(NumbericLiteral { value: 0 }),
                right: Expr::NumbericLiteral(NumbericLiteral { value: 2 }),
            }))
        );
    }

    #[test]
    fn test_binary_operator_comparison() {
        let mut token_table = TokenTable::with_capacity(3);
        token_table.push(TokenKind::Identifier, 0, 1);
        token_table.push(TokenKind::Equal, 2, 2);
        token_table.push(TokenKind::Number, 3, 3);

        let mut cursor = 0;
        let expr = Expr::build(&token_table, &mut cursor).unwrap();
        assert_eq!(
            expr,
            Expr::BinaryOp(Box::new(BinaryOp {
                op: BinaryOperator::Equal,
                left: Expr::Field(Field {
                    prefix: None,
                    value: 0
                }),
                right: Expr::NumbericLiteral(NumbericLiteral { value: 2 }),
            }))
        );
        assert_eq!(cursor, 3);

        let mut token_table = TokenTable::with_capacity(3);
        token_table.push(TokenKind::Identifier, 0, 1);
        token_table.push(TokenKind::NotEqual, 2, 2);
        token_table.push(TokenKind::StringLiteral, 3, 5);

        let mut cursor = 0;
        let expr = Expr::build(&token_table, &mut cursor).unwrap();
        assert_eq!(
            expr,
            Expr::BinaryOp(Box::new(BinaryOp {
                op: BinaryOperator::NotEqual,
                left: Expr::Field(Field {
                    prefix: None,
                    value: 0
                }),
                right: Expr::StringLiteral(StringLiteral { value: 2 }),
            }))
        );

        let mut token_table = TokenTable::with_capacity(3);
        token_table.push(TokenKind::Identifier, 0, 1);
        token_table.push(TokenKind::Greater, 2, 2);
        token_table.push(TokenKind::Number, 3, 3);

        let mut cursor = 0;
        let expr = Expr::build(&token_table, &mut cursor).unwrap();
        assert_eq!(
            expr,
            Expr::BinaryOp(Box::new(BinaryOp {
                op: BinaryOperator::Greater,
                left: Expr::Field(Field {
                    prefix: None,
                    value: 0
                }),
                right: Expr::NumbericLiteral(NumbericLiteral { value: 2 }),
            }))
        );

        let mut token_table = TokenTable::with_capacity(3);
        token_table.push(TokenKind::Identifier, 0, 1);
        token_table.push(TokenKind::LessEqual, 2, 2);
        token_table.push(TokenKind::Number, 3, 3);

        let mut cursor = 0;
        let expr = Expr::build(&token_table, &mut cursor).unwrap();
        assert_eq!(
            expr,
            Expr::BinaryOp(Box::new(BinaryOp {
                op: BinaryOperator::LessEqual,
                left: Expr::Field(Field {
                    prefix: None,
                    value: 0
                }),
                right: Expr::NumbericLiteral(NumbericLiteral { value: 2 }),
            }))
        );

        let mut token_table = TokenTable::with_capacity(3);
        token_table.push(TokenKind::Identifier, 0, 1);
        token_table.push(TokenKind::GreaterEqual, 2, 2);
        token_table.push(TokenKind::Number, 3, 3);

        let mut cursor = 0;
        let expr = Expr::build(&token_table, &mut cursor).unwrap();
        assert_eq!(
            expr,
            Expr::BinaryOp(Box::new(BinaryOp {
                op: BinaryOperator::GreaterEqual,
                left: Expr::Field(Field {
                    prefix: None,
                    value: 0
                }),
                right: Expr::NumbericLiteral(NumbericLiteral { value: 2 }),
            }))
        );
    }

    #[test]
    fn test_binary_operator_logical() {
        let mut token_table = TokenTable::with_capacity(5);
        token_table.push(TokenKind::Identifier, 0, 1);
        token_table.push(TokenKind::Keyword(Keyword::And), 2, 4);
        token_table.push(TokenKind::Identifier, 5, 6);
        token_table.push(TokenKind::Keyword(Keyword::Or), 7, 8);
        token_table.push(TokenKind::Identifier, 9, 10);

        let mut cursor = 0;
        let expr = Expr::build(&token_table, &mut cursor).unwrap();
        assert_eq!(
            expr,
            Expr::BinaryOp(Box::new(BinaryOp {
                op: BinaryOperator::Or,
                left: Expr::BinaryOp(Box::new(BinaryOp {
                    op: BinaryOperator::And,
                    left: Expr::Field(Field {
                        prefix: None,
                        value: 0
                    }),
                    right: Expr::Field(Field {
                        prefix: None,
                        value: 2
                    }),
                })),
                right: Expr::Field(Field {
                    prefix: None,
                    value: 4
                }),
            }))
        );
        assert_eq!(cursor, 5);
    }

    #[test]
    fn test_string_literal() {
        let mut token_table = TokenTable::with_capacity(1);
        token_table.push(TokenKind::StringLiteral, 0, 10);

        let mut cursor = 0;
        let expr = Expr::class_string_literal(&token_table, &mut cursor).unwrap();
        assert_eq!(expr, Expr::StringLiteral(StringLiteral { value: 0 }));
        assert_eq!(cursor, 1);
    }

    #[test]
    fn test_number_literal() {
        let mut token_table = TokenTable::with_capacity(1);
        token_table.push(TokenKind::Number, 0, 5);

        let mut cursor = 0;
        let expr = Expr::class_number_literal(&token_table, &mut cursor).unwrap();
        assert_eq!(expr, Expr::NumbericLiteral(NumbericLiteral { value: 0 }));
        assert_eq!(cursor, 1);
    }

    #[test]
    fn test_star() {
        let mut token_table = TokenTable::with_capacity(1);
        token_table.push(TokenKind::Multiply, 0, 0);

        let mut cursor = 0;
        let expr = Expr::class_star(&token_table, &mut cursor).unwrap();
        assert_eq!(expr, Expr::Star(Star { prefix: None }));
        assert_eq!(cursor, 1);

        let mut token_table = TokenTable::with_capacity(3);
        token_table.push(TokenKind::Identifier, 0, 1);
        token_table.push(TokenKind::Dot, 2, 2);
        token_table.push(TokenKind::Multiply, 3, 3);

        let mut cursor = 0;
        let expr = Expr::class_star(&token_table, &mut cursor).unwrap();
        assert_eq!(expr, Expr::Star(Star { prefix: Some(0) }));
        assert_eq!(cursor, 3);
    }

    #[test]
    fn test_field_simple() {
        let mut token_table = TokenTable::with_capacity(1);
        token_table.push(TokenKind::Identifier, 0, 5);

        let mut cursor = 0;
        let expr = Expr::class_field(&token_table, &mut cursor).unwrap();
        assert_eq!(
            expr,
            Expr::Field(Field {
                prefix: None,
                value: 0
            })
        );
        assert_eq!(cursor, 1);
    }

    #[test]
    fn test_field_with_prefix() {
        let mut token_table = TokenTable::with_capacity(3);
        token_table.push(TokenKind::Identifier, 0, 3);
        token_table.push(TokenKind::Dot, 4, 4);
        token_table.push(TokenKind::Identifier, 5, 8);

        let mut cursor = 0;
        let expr = Expr::class_field(&token_table, &mut cursor).unwrap();
        assert_eq!(
            expr,
            Expr::Field(Field {
                prefix: Some(0),
                value: 2
            })
        );
        assert_eq!(cursor, 3);
    }

    #[test]
    fn test_field_invalid() {
        let mut token_table = TokenTable::with_capacity(2);
        token_table.push(TokenKind::Number, 0, 1);
        token_table.push(TokenKind::Identifier, 2, 3);

        let mut cursor = 0;
        let result = Expr::class_field(&token_table, &mut cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_function_multiple_args() {
        let mut token_table = TokenTable::with_capacity(9);
        token_table.push(TokenKind::Identifier, 0, 3);
        token_table.push(TokenKind::LeftParen, 4, 4);
        token_table.push(TokenKind::Number, 5, 5);
        token_table.push(TokenKind::Comma, 6, 6);
        token_table.push(TokenKind::Identifier, 7, 8);
        token_table.push(TokenKind::Comma, 9, 9);
        token_table.push(TokenKind::StringLiteral, 10, 15);
        token_table.push(TokenKind::Comma, 16, 16);
        token_table.push(TokenKind::Number, 17, 20);
        token_table.push(TokenKind::RightParen, 21, 21);

        let mut cursor = 0;
        let expr = Expr::class_function_call(&token_table, &mut cursor).unwrap();
        assert_eq!(
            expr,
            Expr::FunctionCall(FunctionCall {
                name: 0,
                args: mini_vec![
                    Expr::NumbericLiteral(NumbericLiteral { value: 2 }),
                    Expr::Field(Field {
                        prefix: None,
                        value: 4
                    }),
                    Expr::StringLiteral(StringLiteral { value: 6 }),
                    Expr::NumbericLiteral(NumbericLiteral { value: 8 }),
                ]
            })
        );
        assert_eq!(cursor, 10);
    }

    #[test]
    fn test_parenthesized_expression() {
        let mut token_table = TokenTable::with_capacity(7);
        token_table.push(TokenKind::LeftParen, 0, 0);
        token_table.push(TokenKind::Number, 1, 1);
        token_table.push(TokenKind::Plus, 2, 2);
        token_table.push(TokenKind::Number, 3, 3);
        token_table.push(TokenKind::RightParen, 4, 4);
        token_table.push(TokenKind::Multiply, 5, 5);
        token_table.push(TokenKind::Number, 6, 6);

        let mut cursor = 0;
        let expr = Expr::build(&token_table, &mut cursor).unwrap();
        assert_eq!(
            expr,
            Expr::BinaryOp(Box::new(BinaryOp {
                op: BinaryOperator::Multiply,
                left: Expr::BinaryOp(Box::new(BinaryOp {
                    op: BinaryOperator::Add,
                    left: Expr::NumbericLiteral(NumbericLiteral { value: 1 }),
                    right: Expr::NumbericLiteral(NumbericLiteral { value: 3 }),
                })),
                right: Expr::NumbericLiteral(NumbericLiteral { value: 6 }),
            }))
        );
    }

    #[test]
    fn test_complex_expression() {
        let mut token_table = TokenTable::with_capacity(15);
        token_table.push(TokenKind::Identifier, 0, 1);
        token_table.push(TokenKind::Multiply, 2, 2);
        token_table.push(TokenKind::LeftParen, 3, 3);
        token_table.push(TokenKind::Number, 4, 4);
        token_table.push(TokenKind::Plus, 5, 5);
        token_table.push(TokenKind::Number, 6, 6);
        token_table.push(TokenKind::RightParen, 7, 7);
        token_table.push(TokenKind::Keyword(Keyword::And), 8, 10);
        token_table.push(TokenKind::Identifier, 11, 12);
        token_table.push(TokenKind::Greater, 13, 13);
        token_table.push(TokenKind::Number, 14, 14);

        let mut cursor = 0;
        let expr = Expr::build(&token_table, &mut cursor).unwrap();
        assert_eq!(
            expr,
            Expr::BinaryOp(Box::new(BinaryOp {
                op: BinaryOperator::And,
                left: Expr::BinaryOp(Box::new(BinaryOp {
                    op: BinaryOperator::Multiply,
                    left: Expr::Field(Field {
                        prefix: None,
                        value: 0
                    }),
                    right: Expr::BinaryOp(Box::new(BinaryOp {
                        op: BinaryOperator::Add,
                        left: Expr::NumbericLiteral(NumbericLiteral { value: 3 }),
                        right: Expr::NumbericLiteral(NumbericLiteral { value: 5 }),
                    })),
                })),
                right: Expr::BinaryOp(Box::new(BinaryOp {
                    op: BinaryOperator::Greater,
                    left: Expr::Field(Field {
                        prefix: None,
                        value: 8
                    }),
                    right: Expr::NumbericLiteral(NumbericLiteral { value: 10 }),
                })),
            }))
        );
    }

    #[test]
    fn test_precedence_add_subtract_vs_multiply_divide() {
        let mut token_table = TokenTable::with_capacity(9);
        token_table.push(TokenKind::Number, 0, 0);
        token_table.push(TokenKind::Plus, 1, 1);
        token_table.push(TokenKind::Number, 2, 2);
        token_table.push(TokenKind::Multiply, 3, 3);
        token_table.push(TokenKind::Number, 4, 4);
        token_table.push(TokenKind::Divide, 5, 5);
        token_table.push(TokenKind::Number, 6, 6);
        token_table.push(TokenKind::Subtract, 7, 7);
        token_table.push(TokenKind::Number, 8, 8);

        let mut cursor = 0;
        let expr = Expr::build(&token_table, &mut cursor).unwrap();
        let inner = match &expr {
            Expr::BinaryOp(bop) => &bop.right,
            _ => panic!("Expected BinaryOp"),
        };
        assert_eq!(
            expr,
            Expr::BinaryOp(Box::new(BinaryOp {
                op: BinaryOperator::Subtract,
                left: Expr::BinaryOp(Box::new(BinaryOp {
                    op: BinaryOperator::Add,
                    left: Expr::NumbericLiteral(NumbericLiteral { value: 0 }),
                    right: Expr::BinaryOp(Box::new(BinaryOp {
                        op: BinaryOperator::Divide,
                        left: Expr::BinaryOp(Box::new(BinaryOp {
                            op: BinaryOperator::Multiply,
                            left: Expr::NumbericLiteral(NumbericLiteral { value: 2 }),
                            right: Expr::NumbericLiteral(NumbericLiteral { value: 4 }),
                        })),
                        right: Expr::NumbericLiteral(NumbericLiteral { value: 6 }),
                    })),
                })),
                right: Expr::NumbericLiteral(NumbericLiteral { value: 8 }),
            }))
        );
    }

    #[test]
    fn test_alias_without_as() {
        let mut token_table = TokenTable::with_capacity(3);
        token_table.push(TokenKind::Identifier, 0, 3);
        token_table.push(TokenKind::Identifier, 4, 8);

        let mut cursor = 0;
        let alias = Alias::new(&token_table, &mut cursor).unwrap();
        assert_eq!(
            alias,
            Alias {
                name: Some(1),
                value: Expr::Field(Field {
                    prefix: None,
                    value: 0
                }),
            }
        );
        assert_eq!(cursor, 2);
    }

    #[test]
    fn test_alias_with_as() {
        let mut token_table = TokenTable::with_capacity(3);
        token_table.push(TokenKind::Identifier, 0, 3);
        token_table.push(TokenKind::Keyword(Keyword::As), 4, 5);
        token_table.push(TokenKind::Identifier, 6, 10);

        let mut cursor = 0;
        let alias = Alias::new(&token_table, &mut cursor).unwrap();
        assert_eq!(
            alias,
            Alias {
                name: Some(2),
                value: Expr::Field(Field {
                    prefix: None,
                    value: 0
                }),
            }
        );
        assert_eq!(cursor, 3);
    }

    #[test]
    fn test_alias_function_call() {
        let mut token_table = TokenTable::with_capacity(5);
        token_table.push(TokenKind::Identifier, 0, 3);
        token_table.push(TokenKind::LeftParen, 4, 4);
        token_table.push(TokenKind::Number, 5, 5);
        token_table.push(TokenKind::RightParen, 6, 6);
        token_table.push(TokenKind::Identifier, 7, 10);

        let mut cursor = 0;
        let alias = Alias::new(&token_table, &mut cursor).unwrap();
        assert_eq!(
            alias,
            Alias {
                name: Some(4),
                value: Expr::FunctionCall(FunctionCall {
                    name: 0,
                    args: mini_vec![Expr::NumbericLiteral(NumbericLiteral { value: 2 })],
                }),
            }
        );
        assert_eq!(cursor, 5);
    }

    #[test]
    fn test_function_call_with_nested_expression() {
        let mut token_table = TokenTable::with_capacity(11);
        token_table.push(TokenKind::Identifier, 0, 3);
        token_table.push(TokenKind::LeftParen, 4, 4);
        token_table.push(TokenKind::Identifier, 5, 6);
        token_table.push(TokenKind::Plus, 7, 7);
        token_table.push(TokenKind::Number, 8, 8);
        token_table.push(TokenKind::Comma, 9, 9);
        token_table.push(TokenKind::Identifier, 10, 13);
        token_table.push(TokenKind::Multiply, 14, 14);
        token_table.push(TokenKind::Number, 15, 15);
        token_table.push(TokenKind::RightParen, 16, 16);

        let mut cursor = 0;
        let expr = Expr::class_function_call(&token_table, &mut cursor).unwrap();
        assert_eq!(
            expr,
            Expr::FunctionCall(FunctionCall {
                name: 0,
                args: mini_vec![
                    Expr::BinaryOp(Box::new(BinaryOp {
                        op: BinaryOperator::Add,
                        left: Expr::Field(Field {
                            prefix: None,
                            value: 2
                        }),
                        right: Expr::NumbericLiteral(NumbericLiteral { value: 4 }),
                    })),
                    Expr::BinaryOp(Box::new(BinaryOp {
                        op: BinaryOperator::Multiply,
                        left: Expr::Field(Field {
                            prefix: None,
                            value: 6
                        }),
                        right: Expr::NumbericLiteral(NumbericLiteral { value: 8 }),
                    })),
                ]
            })
        );
    }

    #[test]
    fn test_longer_arithmetic_chain() {
        let mut token_table = TokenTable::with_capacity(13);
        token_table.push(TokenKind::Number, 0, 0);
        token_table.push(TokenKind::Plus, 1, 1);
        token_table.push(TokenKind::Number, 2, 2);
        token_table.push(TokenKind::Plus, 3, 3);
        token_table.push(TokenKind::Number, 4, 4);
        token_table.push(TokenKind::Plus, 5, 5);
        token_table.push(TokenKind::Number, 6, 6);
        token_table.push(TokenKind::Multiply, 7, 7);
        token_table.push(TokenKind::Number, 8, 8);
        token_table.push(TokenKind::Multiply, 9, 9);
        token_table.push(TokenKind::Number, 10, 10);
        token_table.push(TokenKind::Plus, 11, 11);
        token_table.push(TokenKind::Number, 12, 12);

        let mut cursor = 0;
        let expr = Expr::build(&token_table, &mut cursor).unwrap();
        assert_eq!(cursor, 13);
    }

    #[test]
    fn test_comparison_chain_with_and_or() {
        let mut token_table = TokenTable::with_capacity(11);
        token_table.push(TokenKind::Identifier, 0, 0);
        token_table.push(TokenKind::Greater, 1, 1);
        token_table.push(TokenKind::Number, 2, 2);
        token_table.push(TokenKind::Keyword(Keyword::And), 3, 5);
        token_table.push(TokenKind::Identifier, 6, 6);
        token_table.push(TokenKind::Less, 7, 7);
        token_table.push(TokenKind::Number, 8, 8);
        token_table.push(TokenKind::Keyword(Keyword::Or), 9, 10);
        token_table.push(TokenKind::Identifier, 11, 11);
        token_table.push(TokenKind::Equal, 12, 12);
        token_table.push(TokenKind::Number, 13, 13);

        let mut cursor = 0;
        let expr = Expr::build(&token_table, &mut cursor).unwrap();
        assert_eq!(cursor, 11);
    }

    #[test]
    fn test_between_invalid_missing_and() {
        let mut token_table = TokenTable::with_capacity(5);
        token_table.push(TokenKind::Identifier, 0, 3);
        token_table.push(TokenKind::Keyword(Keyword::Between), 4, 10);
        token_table.push(TokenKind::Number, 11, 12);
        token_table.push(TokenKind::Number, 13, 14);
        token_table.push(TokenKind::Number, 15, 16);

        let mut cursor = 0;
        let result = Expr::build(&token_table, &mut cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_in_basic() {
        let mut token_table = TokenTable::with_capacity(9);
        token_table.push(TokenKind::Identifier, 0, 3);
        token_table.push(TokenKind::Keyword(Keyword::In), 4, 5);
        token_table.push(TokenKind::LeftParen, 6, 6);
        token_table.push(TokenKind::Number, 7, 7);
        token_table.push(TokenKind::Comma, 8, 8);
        token_table.push(TokenKind::Number, 9, 9);
        token_table.push(TokenKind::Comma, 10, 10);
        token_table.push(TokenKind::StringLiteral, 11, 15);
        token_table.push(TokenKind::RightParen, 16, 16);

        let mut cursor = 0;
        let result = Expr::build(&token_table, &mut cursor);
        assert!(result.is_ok());
        let expr = result.unwrap();
        assert_eq!(cursor, 9);

        let expected = Expr::In(In {
            field: Box::new(Expr::Field(Field {
                prefix: None,
                value: 0,
            })),
            values: mini_vec![
                Expr::NumbericLiteral(NumbericLiteral { value: 3 }),
                Expr::NumbericLiteral(NumbericLiteral { value: 5 }),
                Expr::StringLiteral(StringLiteral { value: 7 }),
            ],
        });
        assert_eq!(expr, expected);
    }

    #[test]
    fn test_in_with_single_value() {
        let mut token_table = TokenTable::with_capacity(5);
        token_table.push(TokenKind::Identifier, 0, 3);
        token_table.push(TokenKind::Keyword(Keyword::In), 4, 5);
        token_table.push(TokenKind::LeftParen, 6, 6);
        token_table.push(TokenKind::Number, 7, 8);
        token_table.push(TokenKind::RightParen, 9, 9);

        let mut cursor = 0;
        let result = Expr::build(&token_table, &mut cursor);
        assert!(result.is_ok());
        assert_eq!(cursor, 5);

        let expr = result.unwrap();
        if let Expr::In(in_expr) = expr {
            assert_eq!(in_expr.values.len(), 1);
        } else {
            panic!("Expected In expression");
        }
    }

    #[test]
    fn test_in_with_field_values() {
        let mut token_table = TokenTable::with_capacity(9);
        token_table.push(TokenKind::Identifier, 0, 3);
        token_table.push(TokenKind::Keyword(Keyword::In), 4, 5);
        token_table.push(TokenKind::LeftParen, 6, 6);
        token_table.push(TokenKind::Identifier, 7, 10);
        token_table.push(TokenKind::Comma, 11, 11);
        token_table.push(TokenKind::Identifier, 12, 15);
        token_table.push(TokenKind::Comma, 16, 16);
        token_table.push(TokenKind::Identifier, 17, 20);
        token_table.push(TokenKind::RightParen, 21, 21);

        let mut cursor = 0;
        let result = Expr::build(&token_table, &mut cursor);
        assert!(result.is_ok());
        let expr = result.unwrap();
        assert_eq!(cursor, 9);

        let expected = Expr::In(In {
            field: Box::new(Expr::Field(Field {
                prefix: None,
                value: 0,
            })),
            values: mini_vec![
                Expr::Field(Field {
                    prefix: None,
                    value: 3
                }),
                Expr::Field(Field {
                    prefix: None,
                    value: 5
                }),
                Expr::Field(Field {
                    prefix: None,
                    value: 7
                }),
            ],
        });
        assert_eq!(expr, expected);
    }

    #[test]
    fn test_in_invalid_missing_paren() {
        let mut token_table = TokenTable::with_capacity(5);
        token_table.push(TokenKind::Identifier, 0, 3);
        token_table.push(TokenKind::Keyword(Keyword::In), 4, 5);
        token_table.push(TokenKind::Number, 6, 7);
        token_table.push(TokenKind::Comma, 8, 8);
        token_table.push(TokenKind::Number, 9, 10);

        let mut cursor = 0;
        let result = Expr::build(&token_table, &mut cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_in_invalid_missing_value() {
        let mut token_table = TokenTable::with_capacity(4);
        token_table.push(TokenKind::Identifier, 0, 3);
        token_table.push(TokenKind::Keyword(Keyword::In), 4, 5);
        token_table.push(TokenKind::LeftParen, 6, 6);
        token_table.push(TokenKind::RightParen, 7, 7);

        let mut cursor = 0;
        let result = Expr::build(&token_table, &mut cursor);
        assert!(result.is_err());
    }
}
