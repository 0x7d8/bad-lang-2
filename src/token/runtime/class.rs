use crate::{
    runtime::Runtime,
    token::{
        TokenLocation,
        base::{StringToken, ValueToken},
        logic::ExpressionToken,
    },
};

use std::sync::{Arc, LazyLock};

pub static FUNCTIONS: LazyLock<Vec<&str>> = LazyLock::new(|| vec!["class#get"]);

pub fn run(
    name: &str,
    args: &[Arc<ExpressionToken>],
    runtime: &mut Runtime,
    location: &TokenLocation,
) -> Option<ExpressionToken> {
    match name {
        "class#get" => {
            if args.len() != 2 {
                panic!("array#get requires 2 arguments in {location}");
            }

            let value = runtime.extract_value(&args[0])?;
            match value {
                ValueToken::ClassInstance(class_instance) => {
                    let value = runtime.extract_value(&args[1])?;
                    match value {
                        ValueToken::String(StringToken { value, .. }) => {
                            let class_instance = class_instance.scope.read().unwrap();
                            let value = class_instance.get(&value);

                            value.map(|value| value.read().unwrap().clone())
                        }
                        _ => {
                            panic!(
                                "class#get requires a string as the second argument in {location}"
                            );
                        }
                    }
                }
                _ => {
                    panic!(
                        "class#get requires a class instance as the first argument in {location}"
                    );
                }
            }
        }
        _ => None,
    }
}
