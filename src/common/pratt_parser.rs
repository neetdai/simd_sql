use crate::{
    ParserError,
    token::{TokenKind, TokenTable},
};

pub(crate) trait PrattOutput<I>
where
    I: PrecedenceTrait,
{
    fn apply(op: I, left: Self, right: Self) -> Self;
}

pub(crate) trait PrecedenceTrait {
    /// 获取运算符的优先级，数值越大优先级越高
    fn precedence(&self) -> usize;

    /// 判断是否是左结合的运算符
    fn is_left_associative(&self) -> bool;

    fn min_precedence() -> usize;
}

pub(crate) trait PrattParserTrait {
    type Item: PrecedenceTrait;
    type Output: PrattOutput<Self::Item>;

    fn parse_primary(
        token_table: &TokenTable,
        cursor: &mut usize,
    ) -> Result<Self::Output, ParserError>;

    fn match_item(token_kind: &TokenKind) -> Option<Self::Item>;
}

#[derive(Debug)]
pub(crate) struct PrattParser;

impl PrattParser {
    /// 使用 Pratt Parser 解析表达式
    pub(crate) fn parse_expression<P>(
        token_table: &TokenTable,
        cursor: &mut usize,
    ) -> Result<P::Output, ParserError>
    where
        P: PrattParserTrait,
    {
        Self::parse_expression_with_min_precedence::<P>(
            token_table,
            cursor,
            P::Item::min_precedence(),
        )
    }

    /// 使用 Pratt Parser 解析表达式，支持指定最小优先级
    fn parse_expression_with_min_precedence<P>(
        token_table: &TokenTable,
        cursor: &mut usize,
        min_precedence: usize,
    ) -> Result<P::Output, ParserError>
    where
        P: PrattParserTrait,
    {
        // 解析左侧表达式（原子表达式）
        let mut left = P::parse_primary(token_table, cursor)?;

        // 循环处理二元运算符
        loop {
            // 检查当前 token 是否是二元运算符
            let op = match token_table.get_kind(*cursor).and_then(P::match_item) {
                Some(op) => op,
                None => break, // 如果不是二元运算符，则退出循环
            };

            if op.precedence() < min_precedence {
                break; // 如果运算符优先级低于最小优先级，则退出循环
            }

            *cursor += 1; // 消耗运算符 token

            let next_min_precedence = if op.is_left_associative() {
                op.precedence() + 1
            } else {
                op.precedence()
            };

            let right = Self::parse_expression_with_min_precedence::<P>(
                token_table,
                cursor,
                next_min_precedence,
            )?;

            left = P::Output::apply(op, left, right)
        }

        Ok(left)
    }
}
