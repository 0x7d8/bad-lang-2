use crate::{
    runtime::Runtime,
    token::LINE,
    token::{
        base::{BaseToken, NumberToken, ValueToken},
        logic::ExpressionToken,
    },
};

use super::string;

use std::sync::{Arc, LazyLock};

pub static FUNCTIONS: LazyLock<Vec<&str>> =
    LazyLock::new(|| vec!["math#eval", "#=", "math#floor", "math#ceil", "math#round"]);

pub fn run(
    name: &str,
    args: &[Arc<ExpressionToken>],
    runtime: &mut Runtime,
) -> Option<ExpressionToken> {
    match name {
        "math#eval" => {
            if args.len() != 1 {
                panic!("math#eval requires 1 argument on line {}", unsafe { LINE });
            }

            let value = runtime.extract_value(&args[0])?;
            let path = value.value().to_string();
            let result = meval::eval_str(&path).unwrap();

            Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                location: Default::default(),
                value: result,
            })))
        }
        "#=" => {
            if args.is_empty() {
                panic!(
                    "#= (math#eval) requires at least 1 argument on line {}",
                    unsafe { LINE }
                );
            }

            let expression = string::run("string#format", args, runtime)?;
            let result = meval::eval_str(runtime.extract_value(&expression)?.value()).unwrap();

            Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                location: Default::default(),
                value: result,
            })))
        }
        "math#floor" => {
            if args.len() != 1 {
                panic!("math#floor requires 1 argument on line {}", unsafe { LINE });
            }

            let value = runtime.extract_value(&args[0])?;
            let value = match value {
                ValueToken::Number(value) => value.value,
                _ => panic!("math#floor requires a number on line {}", unsafe { LINE }),
            };

            Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                location: Default::default(),
                value: value.floor(),
            })))
        }
        "math#ceil" => {
            if args.len() != 1 {
                panic!("math#ceil requires 1 argument on line {}", unsafe { LINE });
            }

            let value = runtime.extract_value(&args[0])?;
            let value = match value {
                ValueToken::Number(value) => value.value,
                _ => panic!("math#ceil requires a number on line {}", unsafe { LINE }),
            };

            Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                location: Default::default(),
                value: value.ceil(),
            })))
        }
        "math#round" => {
            if args.len() != 1 {
                panic!("math#round requires 1 argument on line {}", unsafe { LINE });
            }

            let value = runtime.extract_value(&args[0])?;
            let value = match value {
                ValueToken::Number(value) => value.value,
                _ => panic!("math#round requires a number on line {}", unsafe { LINE }),
            };

            Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                location: Default::default(),
                value: value.round(),
            })))
        }
        _ => None,
    }
}
