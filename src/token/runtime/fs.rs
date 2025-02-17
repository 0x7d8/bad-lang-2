use crate::{
    runtime::Runtime,
    token::{
        base::{BaseToken, StringToken, ValueToken},
        logic::ExpressionToken,
    },
};

use std::{rc::Rc, sync::LazyLock};

pub static FUNCTIONS: LazyLock<Vec<&str>> = LazyLock::new(|| vec!["fs#readstr"]);

pub fn run(
    name: &str,
    args: &Vec<Rc<ExpressionToken>>,
    runtime: &mut Runtime,
) -> Option<ExpressionToken> {
    match name {
        "fs#readstr" => {
            if args.len() != 1 {
                return None;
            }

            let value = runtime.extract_value(&args[0])?;
            let path = value.value().to_string();
            let content = std::fs::read_to_string(path).unwrap();

            Some(ExpressionToken::Value(ValueToken::String(StringToken {
                value: content,
            })))
        }
        _ => None,
    }
}
