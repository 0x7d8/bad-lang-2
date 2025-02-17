use crate::{
    runtime::Runtime,
    token::{
        base::{BaseToken, NullToken, ValueToken},
        logic::ExpressionToken,
    },
};

use std::{rc::Rc, sync::LazyLock};

pub static FUNCTIONS: LazyLock<Vec<&str>> = LazyLock::new(|| vec!["io#println", "io#inspect"]);

pub fn run(
    name: &str,
    args: &Vec<Rc<ExpressionToken>>,
    runtime: &mut Runtime,
) -> Option<ExpressionToken> {
    match name {
        "io#println" => {
            if args.len() != 1 {
                return None;
            }

            let value = runtime.extract_value(&args[0])?;
            println!("{}", value.value());

            Some(ExpressionToken::Value(ValueToken::Null(NullToken)))
        }
        "io#inspect" => {
            if args.len() != 1 {
                return None;
            }

            let value = runtime.extract_value(&args[0])?;
            println!("{}", value.inspect());

            Some(ExpressionToken::Value(ValueToken::Null(NullToken)))
        }
        _ => None,
    }
}
