use crate::{
    runtime::Runtime,
    token::{
        base::{BaseToken, NumberToken, StringToken, ValueToken},
        logic::ExpressionToken,
    },
};

use std::{rc::Rc, sync::LazyLock};

pub static FUNCTIONS: LazyLock<Vec<&str>> =
    LazyLock::new(|| vec!["string#concat", "string#format", "string#len"]);

pub fn run(
    name: &str,
    args: &Vec<Rc<ExpressionToken>>,
    runtime: &mut Runtime,
) -> Option<ExpressionToken> {
    match name {
        "string#concat" => {
            if args.len() < 2 {
                return None;
            }

            let mut result = String::new();

            for arg in args {
                let value = runtime.extract_value(&arg)?;

                result.push_str(&value.value());
            }

            Some(ExpressionToken::Value(ValueToken::String(StringToken {
                value: result,
            })))
        }
        "string#format" => {
            if args.len() < 2 {
                return None;
            }

            let format = runtime.extract_value(&args[0])?.value().to_string();
            let mut values = Vec::new();

            for arg in args.iter().skip(1) {
                let value = runtime.extract_value(&arg)?;

                values.push(value.value());
            }

            let mut result = format;
            for value in values {
                result = result.replacen("{}", &value, 1);
            }

            Some(ExpressionToken::Value(ValueToken::String(StringToken {
                value: result,
            })))
        }
        "string#len" => {
            if args.len() != 1 {
                return None;
            }

            let value = runtime.extract_value(&args[0])?;
            let len = value.value().len();

            Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                value: len as f64,
            })))
        }
        _ => None,
    }
}
