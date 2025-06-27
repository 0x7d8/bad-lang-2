use crate::{
    runtime::Runtime,
    token::{
        TokenLocation,
        base::{NumberToken, ValueToken},
        logic::ExpressionToken,
    },
};

use std::sync::{Arc, LazyLock};

pub static FUNCTIONS: LazyLock<Vec<&str>> = LazyLock::new(|| vec!["rng#rand", "rng#rand_range"]);

pub fn run(
    name: &str,
    args: &[Arc<ExpressionToken>],
    runtime: &mut Runtime,
    location: &TokenLocation,
) -> Option<ExpressionToken> {
    match name {
        "rng#rand" => {
            if !args.is_empty() {
                panic!("rng#rand requires 0 arguments in {location}");
            }

            let result = rand::random::<f64>();

            Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                location: Default::default(),
                value: result,
            })))
        }
        "rng#rand_range" => {
            if args.len() != 2 {
                panic!("rng#rand_range requires 2 arguments in {location}");
            }

            let min = runtime.extract_value(&args[0])?;
            let max = runtime.extract_value(&args[1])?;

            match (min, max) {
                (ValueToken::Number(min), ValueToken::Number(max)) => {
                    let result = rand::random::<f64>() * (max.value - min.value) + min.value;

                    Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                        location: Default::default(),
                        value: result,
                    })))
                }
                _ => {
                    panic!("rng#rand_range requires 2 numbers in {location}");
                }
            }
        }
        _ => None,
    }
}
