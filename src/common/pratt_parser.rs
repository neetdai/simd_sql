use std::fmt::Debug;

use crate::{ParserError, token::TokenTable};

pub(crate) trait PrattOutput<I>
where
    I: PrecedenceTrait,
{
    fn apply(op: I, left: Self, right: Self) -> Self;
}

pub(crate) trait PrecedenceTrait {
    fn precedence(&self) -> usize;
    fn is_left_associative(&self) -> bool;
    fn min_precedence() -> usize;
}

pub(crate) trait PrattParserTrait<'a> {
    type Item: PrecedenceTrait;
    type Output: PrattOutput<Self::Item> + Debug + 'a;

    fn parse_primary(
        token_table: &TokenTable<'a>,
        cursor: &mut usize,
    ) -> Result<Self::Output, ParserError>;

    fn match_item(token_table: &TokenTable, cursor: &mut usize) -> Option<Self::Item>;

    fn parse_postfix(
        left: Self::Output,
        token_table: &TokenTable<'a>,
        cursor: &mut usize,
    ) -> Result<(Self::Output, Flow), ParserError>;
}

#[derive(Debug, PartialEq)]
pub(crate) enum Flow {
    Continue,
    Run,
}

#[derive(Debug)]
pub(crate) struct PrattParser;

impl PrattParser {
    pub(crate) fn parse_expression<'a, P: PrattParserTrait<'a> + Debug>(
        token_table: &TokenTable<'a>,
        cursor: &mut usize,
    ) -> Result<P::Output, ParserError> {
        Self::parse_expression_with_min_precedence::<P>(
            token_table,
            cursor,
            P::Item::min_precedence(),
        )
    }

    fn parse_expression_with_min_precedence<'a, P: PrattParserTrait<'a> + Debug>(
        token_table: &TokenTable<'a>,
        cursor: &mut usize,
        _initial_min_precedence: usize,
    ) -> Result<P::Output, ParserError> {
        let mut current_left = P::parse_primary(token_table, cursor)?;

        let mut stack: Vec<(P::Item, P::Output)> = Vec::new();

        loop {
            let (new_left, flow) = P::parse_postfix(current_left, token_table, cursor)?;
            current_left = new_left;
            match flow {
                Flow::Continue => continue,
                Flow::Run => {}
            }

            let op = match P::match_item(token_table, cursor) {
                Some(op) => op,
                None => break,
            };

            let op_precedence = op.precedence();

            while let Some((stack_op, _)) = stack.last() {
                let stack_prec = stack_op.precedence();

                let should_reduce = if op.is_left_associative() {
                    stack_prec >= op_precedence
                } else {
                    stack_prec > op_precedence
                };

                if should_reduce {
                    let (top_op, top_left) = stack.pop().unwrap();
                    current_left = P::Output::apply(top_op, top_left, current_left);
                } else {
                    break;
                }
            }

            stack.push((op, current_left));
            current_left = P::parse_primary(token_table, cursor)?;
        }

        while let Some((op, left)) = stack.pop() {
            current_left = P::Output::apply(op, left, current_left);
        }

        Ok(current_left)
    }
}
