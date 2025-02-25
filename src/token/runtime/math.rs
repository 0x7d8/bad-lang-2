use crate::{
    runtime::Runtime,
    token::LINE,
    token::{
        base::{BaseToken, NumberToken, ValueToken},
        logic::ExpressionToken,
    },
};

use std::{rc::Rc, sync::LazyLock};

use super::string;

pub static FUNCTIONS: LazyLock<Vec<&str>> = LazyLock::new(|| vec!["math#eval", "#="]);

pub fn run(
    name: &str,
    args: &[Rc<ExpressionToken>],
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
                value: result,
            })))
        }
        "#=" => {
            if args.len() < 1 {
                panic!("#= (math#eval) requires at least 1 argument on line {}", unsafe { LINE });
            }

            let expression = string::run("string#format", args, runtime)?;
            let result = meval::eval_str(&runtime.extract_value(&expression)?.value()).unwrap();

            Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                value: result,
            })))
        }
        _ => None,
    }
}
