use crate::{
    runtime::Runtime,
    token::{
        TokenLocation,
        base::{BaseToken, NullToken, ValueToken},
        logic::ExpressionToken,
    },
};

use std::sync::{Arc, LazyLock};

pub static FUNCTIONS: LazyLock<Vec<&str>> = LazyLock::new(|| vec!["io#println", "io#inspect"]);

pub fn run(
    name: &str,
    args: &[Arc<ExpressionToken>],
    runtime: &mut Runtime,
    location: &TokenLocation,
) -> Option<ExpressionToken> {
    match name {
        "io#println" => {
            if args.len() != 1 {
                panic!("io#println requires 1 argument in {}", location);
            }

            let value = runtime.extract_value(&args[0])?;
            println!("{}", value.value());

            Some(ExpressionToken::Value(ValueToken::Null(NullToken {
                location: Default::default(),
            })))
        }
        "io#inspect" => {
            if args.len() != 1 {
                panic!("io#inspect requires 1 argument in {}", location);
            }

            let value = runtime.extract_value(&args[0])?;
            println!("{} {}", value.location(), value.inspect());

            Some(ExpressionToken::Value(ValueToken::Null(NullToken {
                location: Default::default(),
            })))
        }
        _ => None,
    }
}
