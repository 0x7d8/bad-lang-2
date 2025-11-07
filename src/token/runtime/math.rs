use crate::{
    runtime::Runtime,
    token::{
        TokenLocation,
        base::{BaseToken, NumberToken, ValueToken},
        logic::ExpressionToken,
    },
};

use super::string;

use std::sync::{Arc, LazyLock};

pub static FUNCTIONS: LazyLock<Vec<&str>> = LazyLock::new(|| {
    vec![
        "math#eval",
        "#=",
        "math#floor",
        "math#ceil",
        "math#round",
        "math#sqrt",
        "math#mod",
    ]
});

pub fn run(
    name: &str,
    args: &[Arc<ExpressionToken>],
    runtime: &mut Runtime,
    location: &TokenLocation,
) -> Option<ExpressionToken> {
    match name {
        "math#eval" => {
            if args.len() != 1 {
                panic!("math#eval requires 1 argument in {location}");
            }

            let value = runtime.extract_value(&args[0])?;
            let path = value.value(0).to_string();
            let result = meval::eval_str(&path).unwrap();

            Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                location: Default::default(),
                value: result,
            })))
        }
        "#=" => {
            if args.is_empty() {
                panic!("#= (math#eval) requires at least 1 argument in {location}");
            }

            let expression = string::run("string#format", args, runtime, location)?;
            let result = meval::eval_str(runtime.extract_value(&expression)?.value(0)).unwrap();

            Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                location: Default::default(),
                value: result,
            })))
        }
        "math#floor" => {
            if args.len() != 1 {
                panic!("math#floor requires 1 argument in {location}");
            }

            let value = runtime.extract_value(&args[0])?;
            let value = match value {
                ValueToken::Number(value) => value.value,
                _ => panic!("math#floor requires a number in {location}"),
            };

            Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                location: Default::default(),
                value: value.floor(),
            })))
        }
        "math#ceil" => {
            if args.len() != 1 {
                panic!("math#ceil requires 1 argument in {location}");
            }

            let value = runtime.extract_value(&args[0])?;
            let value = match value {
                ValueToken::Number(value) => value.value,
                _ => panic!("math#ceil requires a number in {location}"),
            };

            Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                location: Default::default(),
                value: value.ceil(),
            })))
        }
        "math#round" => {
            if args.len() != 1 {
                panic!("math#round requires 1 argument in {location}");
            }

            let value = runtime.extract_value(&args[0])?;
            let value = match value {
                ValueToken::Number(value) => value.value,
                _ => panic!("math#round requires a number in {location}"),
            };

            Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                location: Default::default(),
                value: value.round(),
            })))
        }
        "math#sqrt" => {
            if args.len() != 1 {
                panic!("math#sqrt requires 1 argument in {location}");
            }

            let value = runtime.extract_value(&args[0])?;
            let value = match value {
                ValueToken::Number(value) => value.value,
                _ => panic!("math#sqrt requires a number in {location}"),
            };

            Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                location: Default::default(),
                value: value.sqrt(),
            })))
        }
        "math#mod" => {
            if args.len() != 2 {
                panic!("math#mod requires 2 arguments in {location}");
            }

            let value = runtime.extract_value(&args[0])?;
            let value = match value {
                ValueToken::Number(value) => value.value,
                _ => panic!("math#mod requires 2 numbers in {location}"),
            };

            let value_divisor = runtime.extract_value(&args[1])?;
            let value_divisor = match value_divisor {
                ValueToken::Number(value) => value.value,
                _ => panic!("math#mod requires 2 numbers in {location}"),
            };

            Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                location: Default::default(),
                value: value.rem_euclid(value_divisor),
            })))
        }
        _ => None,
    }
}
