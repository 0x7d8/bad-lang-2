use crate::{
    runtime::Runtime,
    token::{
        TokenLocation,
        base::{BaseToken, BooleanToken, ValueToken},
        logic::ExpressionToken,
    },
};

use std::sync::{Arc, LazyLock};

pub static FUNCTIONS: LazyLock<Vec<&str>> =
    LazyLock::new(|| vec!["#eq", "#lt", "#gt", "#and", "#or"]);

pub fn run(
    name: &str,
    args: &[Arc<ExpressionToken>],
    runtime: &mut Runtime,
    location: &TokenLocation,
) -> Option<ExpressionToken> {
    match name {
        "#eq" => {
            if args.len() != 2 {
                panic!("#eq requires 2 arguments on line {}", location);
            }

            let left = runtime.extract_value(&args[0])?;
            let right = runtime.extract_value(&args[1])?;

            Some(ExpressionToken::Value(ValueToken::Boolean(BooleanToken {
                location: Default::default(),
                value: left.value(0) == right.value(0),
            })))
        }
        "#lt" => {
            if args.len() != 2 {
                panic!("#lt requires 2 arguments on line {}", location);
            }

            let left = runtime.extract_value(&args[0])?;
            let right = runtime.extract_value(&args[1])?;

            match (left, right) {
                (ValueToken::Number(left), ValueToken::Number(right)) => {
                    Some(ExpressionToken::Value(ValueToken::Boolean(BooleanToken {
                        location: Default::default(),
                        value: left.value < right.value,
                    })))
                }
                _ => {
                    panic!("#lt requires 2 numbers on line {}", location);
                }
            }
        }
        "#gt" => {
            if args.len() != 2 {
                panic!("#gt requires 2 arguments on line {}", location);
            }

            let left = runtime.extract_value(&args[0])?;
            let right = runtime.extract_value(&args[1])?;

            match (left, right) {
                (ValueToken::Number(left), ValueToken::Number(right)) => {
                    Some(ExpressionToken::Value(ValueToken::Boolean(BooleanToken {
                        location: Default::default(),
                        value: left.value > right.value,
                    })))
                }
                _ => {
                    panic!("#gt requires 2 numbers on line {}", location);
                }
            }
        }
        "#and" => {
            if args.len() < 2 {
                panic!("#and requires 2 arguments on line {}", location);
            }

            for arg in args {
                let value = runtime.extract_value(arg)?;

                if !value.truthy() {
                    return Some(ExpressionToken::Value(ValueToken::Boolean(BooleanToken {
                        location: Default::default(),
                        value: false,
                    })));
                }
            }

            Some(ExpressionToken::Value(ValueToken::Boolean(BooleanToken {
                location: Default::default(),
                value: true,
            })))
        }
        "#or" => {
            if args.len() < 2 {
                panic!("#or requires 2 arguments on line {}", location);
            }

            for arg in args {
                let value = runtime.extract_value(arg)?;

                if value.truthy() {
                    return Some(ExpressionToken::Value(ValueToken::Boolean(BooleanToken {
                        location: Default::default(),
                        value: true,
                    })));
                }
            }

            Some(ExpressionToken::Value(ValueToken::Boolean(BooleanToken {
                location: Default::default(),
                value: false,
            })))
        }
        _ => None,
    }
}
