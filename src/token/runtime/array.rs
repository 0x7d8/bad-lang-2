use crate::{
    runtime::Runtime,
    token::{
        base::{BooleanToken, NullToken, NumberToken, StringToken, ValueToken},
        logic::ExpressionToken,
    },
};

use std::{rc::Rc, sync::LazyLock};

pub static FUNCTIONS: LazyLock<Vec<&str>> = LazyLock::new(|| {
    vec![
        "array#push",
        "array#pop",
        "array#len",
        "array#get",
        "array#set",
    ]
});

pub fn run(
    name: &str,
    args: &Vec<Rc<ExpressionToken>>,
    runtime: &mut Runtime,
) -> Option<ExpressionToken> {
    match name {
        "array#push" => {
            if args.len() < 2 {
                return None;
            }

            let value = runtime.extract_value(&args[0])?;
            match value {
                ValueToken::Array(array) => {
                    for arg in args.iter().skip(1) {
                        let value = runtime.extract_value(&arg)?;
                        array.value.borrow_mut().push(ExpressionToken::Value(value));
                    }

                    Some(ExpressionToken::Value(ValueToken::Array(array)))
                }
                _ => None,
            }
        }
        "array#pop" => {
            if args.len() != 1 {
                return None;
            }

            let value = runtime.extract_value(&args[0])?;
            match value {
                ValueToken::Array(array) => {
                    let value = array
                        .value
                        .borrow_mut()
                        .pop()
                        .unwrap_or_else(|| ExpressionToken::Value(ValueToken::Null(NullToken)));
                    Some(value)
                }
                _ => None,
            }
        }
        "array#len" => {
            if args.len() != 1 {
                return None;
            }

            let value = runtime.extract_value(&args[0])?;
            match value {
                ValueToken::Array(array) => {
                    let len = array.value.borrow().len();
                    Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                        value: len as f64,
                    })))
                }
                _ => None,
            }
        }
        "array#get" => {
            if args.len() != 2 {
                return None;
            }

            let value = runtime.extract_value(&args[0])?;
            match value {
                ValueToken::Array(array) => {
                    let index = runtime.extract_value(&args[1])?;
                    match index {
                        ValueToken::Number(number) => {
                            let index = number.value as usize;
                            let value =
                                array.value.borrow().get(index).cloned().unwrap_or_else(|| {
                                    ExpressionToken::Value(ValueToken::Null(NullToken))
                                });

                            Some(value)
                        }
                        _ => None,
                    }
                }
                ValueToken::String(string) => {
                    let index = runtime.extract_value(&args[1])?;
                    match index {
                        ValueToken::Number(number) => {
                            let index = number.value as usize;
                            let value = string
                                .value
                                .chars()
                                .nth(index)
                                .map(|c| {
                                    ExpressionToken::Value(ValueToken::String(StringToken {
                                        value: c.to_string(),
                                    }))
                                })
                                .unwrap_or_else(|| {
                                    ExpressionToken::Value(ValueToken::Null(NullToken))
                                });

                            Some(value)
                        }
                        _ => None,
                    }
                }
                ValueToken::Number(num) => {
                    let index = runtime.extract_value(&args[1])?;
                    match index {
                        ValueToken::Number(number) => {
                            let index = number.value as usize;

                            let integer = num.value as u64;
                            let value = (integer >> index) & 1;

                            Some(ExpressionToken::Value(ValueToken::Boolean(BooleanToken {
                                value: value == 1,
                            })))
                        }
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        "array#set" => {
            if args.len() != 3 {
                return None;
            }

            let value = runtime.extract_value(&args[0])?;
            match value {
                ValueToken::Array(array) => {
                    let index = runtime.extract_value(&args[1])?;
                    let value = runtime.extract_value(&args[2])?;

                    match index {
                        ValueToken::Number(number) => {
                            let index = number.value as usize;
                            array.value.borrow_mut().resize(
                                index + 1,
                                ExpressionToken::Value(ValueToken::Null(NullToken)),
                            );
                            array.value.borrow_mut()[index] = ExpressionToken::Value(value);

                            Some(ExpressionToken::Value(ValueToken::Array(array)))
                        }
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}
