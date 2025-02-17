use crate::{
    runtime::Runtime,
    token::{
        base::{BaseToken, BooleanToken, ValueToken},
        logic::ExpressionToken,
    },
};

use std::{rc::Rc, sync::LazyLock};

pub static FUNCTIONS: LazyLock<Vec<&str>> = LazyLock::new(|| vec!["#eq", "#lt", "#gt"]);

pub fn run(
    name: &str,
    args: &Vec<Rc<ExpressionToken>>,
    runtime: &mut Runtime,
) -> Option<ExpressionToken> {
    match name {
        "#eq" => {
            if args.len() != 2 {
                return None;
            }

            let left = runtime.extract_value(&args[0])?;
            let right = runtime.extract_value(&args[1])?;

            Some(ExpressionToken::Value(ValueToken::Boolean(BooleanToken {
                value: left.value() == right.value(),
            })))
        }
        "#lt" => {
            if args.len() != 2 {
                return None;
            }

            let left = runtime.extract_value(&args[0])?;
            let right = runtime.extract_value(&args[1])?;

            match (left, right) {
                (ValueToken::Number(left), ValueToken::Number(right)) => Some(ExpressionToken::Value(
                    ValueToken::Boolean(BooleanToken {
                        value: left.value < right.value,
                    }),
                )),
                _ => None,
            }
        }
        "#gt" => {
            if args.len() != 2 {
                return None;
            }

            let left = runtime.extract_value(&args[0])?;
            let right = runtime.extract_value(&args[1])?;

            match (left, right) {
                (ValueToken::Number(left), ValueToken::Number(right)) => Some(ExpressionToken::Value(
                    ValueToken::Boolean(BooleanToken {
                        value: left.value > right.value,
                    }),
                )),
                _ => None,
            }
        }
        _ => None,
    }
}
