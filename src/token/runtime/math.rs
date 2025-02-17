use crate::{
    runtime::Runtime,
    token::{
        base::{BaseToken, NumberToken, ValueToken},
        logic::ExpressionToken,
    },
};

use std::{rc::Rc, sync::LazyLock};

pub static FUNCTIONS: LazyLock<Vec<&str>> = LazyLock::new(|| vec!["math#eval"]);

pub fn run(
    name: &str,
    args: &Vec<Rc<ExpressionToken>>,
    runtime: &mut Runtime,
) -> Option<ExpressionToken> {
    match name {
        "math#eval" => {
            if args.len() != 1 {
                return None;
            }

            let value = runtime.extract_value(&args[0])?;
            let path = value.value().to_string();
            let result = meval::eval_str(&path).unwrap();

            Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                value: result,
            })))
        }
        _ => None,
    }
}
