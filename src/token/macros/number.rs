use super::extract_number;
use crate::token::{
    base::{NumberToken, ValueToken},
    logic::ExpressionToken,
};

pub fn add(args: Vec<ExpressionToken>) -> Option<ExpressionToken> {
    let mut sum = 0.0;
    for arg in args {
        if let Some(value) = extract_number(&arg) {
            sum += value;
        } else {
            return None;
        }
    }

    Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
        value: sum,
    })))
}

pub fn sqrt(args: Vec<ExpressionToken>) -> Option<ExpressionToken> {
    if args.len() != 1 {
        return None;
    }

    let value = extract_number(&args[0])?;
    if value < 0.0 {
        None
    } else {
        Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
            value: value.sqrt(),
        })))
    }
}
