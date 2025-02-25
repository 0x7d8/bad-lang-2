use crate::{
    runtime::Runtime,
    token::LINE,
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
    args: &[Rc<ExpressionToken>],
    runtime: &mut Runtime,
) -> Option<ExpressionToken> {
    match name {
        "string#concat" => {
            if args.len() < 2 {
                panic!("string#concat requires 2 arguments on line {}", unsafe {
                    LINE
                });
            }

            let mut result = String::new();

            for arg in args {
                let value = runtime.extract_value(arg)?;

                result.push_str(&value.value());
            }

            Some(ExpressionToken::Value(ValueToken::String(StringToken {
                location: Default::default(),
                value: result,
            })))
        }
        "string#format" => {
            if args.is_empty() {
                panic!(
                    "string#format requires at least 1 argument on line {}",
                    unsafe { LINE }
                );
            }

            let format = runtime.extract_value(&args[0])?.value().to_string();
            let mut values = Vec::new();

            for arg in args.iter().skip(1) {
                let value = runtime.extract_value(arg)?;

                values.push(value.value());
            }

            let mut result = format;
            for value in values {
                result = result.replacen("{}", &value, 1);
            }

            Some(ExpressionToken::Value(ValueToken::String(StringToken {
                location: Default::default(),
                value: result,
            })))
        }
        "string#len" => {
            if args.len() != 1 {
                panic!("string#len requires 1 argument on line {}", unsafe { LINE });
            }

            let value = runtime.extract_value(&args[0])?;
            let len = value.value().len();

            Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                location: Default::default(),
                value: len as f64,
            })))
        }
        _ => None,
    }
}
