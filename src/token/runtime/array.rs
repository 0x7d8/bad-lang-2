use crate::{
    runtime::Runtime,
    token::{
        LINE,
        base::{
            ArrayToken, BaseToken, BooleanToken, NullToken, NumberToken, StringToken, ValueToken,
        },
        logic::ExpressionToken,
    },
};

use std::{cell::RefCell, rc::Rc, sync::LazyLock};

pub static FUNCTIONS: LazyLock<Vec<&str>> = LazyLock::new(|| {
    vec![
        "array#push",
        "array#pop",
        "array#len",
        "array#clone",
        "array#concat",
        "array#contains",
        "array#get",
        "array#set",
    ]
});

pub fn run(
    name: &str,
    args: &[Rc<ExpressionToken>],
    runtime: &mut Runtime,
) -> Option<ExpressionToken> {
    match name {
        "array#push" => {
            if args.len() < 2 {
                panic!(
                    "array#push requires at least 2 arguments on line {}",
                    unsafe { LINE }
                );
            }

            let value = runtime.extract_value(&args[0])?;
            match value {
                ValueToken::Array(array) => {
                    for arg in args.iter().skip(1) {
                        let value = runtime.extract_value(arg)?;
                        array.value.borrow_mut().push(ExpressionToken::Value(value));
                    }

                    Some(ExpressionToken::Value(ValueToken::Array(array.clone())))
                }
                _ => {
                    panic!(
                        "array#push requires an array as the first argument on line {}",
                        unsafe { LINE }
                    );
                }
            }
        }
        "array#pop" => {
            if args.len() != 1 {
                panic!("array#pop requires 1 argument on line {}", unsafe { LINE });
            }

            let value = runtime.extract_value(&args[0])?;
            match value {
                ValueToken::Array(array) => {
                    let value = array
                        .value
                        .borrow_mut()
                        .pop()
                        .unwrap_or(ExpressionToken::Value(ValueToken::Null(NullToken {
                            location: Default::default(),
                        })));

                    Some(value)
                }
                _ => {
                    panic!(
                        "array#pop requires an array as the first argument on line {}",
                        unsafe { LINE }
                    );
                }
            }
        }
        "array#len" => {
            if args.len() != 1 {
                panic!("array#len requires 1 argument on line {}", unsafe { LINE });
            }

            let value = runtime.extract_value(&args[0])?;
            match value {
                ValueToken::Array(array) => {
                    let len = array.value.borrow().len();

                    Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                        location: Default::default(),
                        value: len as f64,
                    })))
                }
                _ => {
                    panic!(
                        "array#len requires an array as the first argument on line {}",
                        unsafe { LINE }
                    );
                }
            }
        }
        "array#clone" => {
            if args.len() != 1 {
                panic!("array#clone requires 1 argument on line {}", unsafe {
                    LINE
                });
            }

            let value = runtime.extract_value(&args[0])?;
            match value {
                ValueToken::Array(array) => {
                    let value = array.value.borrow().clone();

                    Some(ExpressionToken::Value(ValueToken::Array(ArrayToken {
                        location: Default::default(),
                        value: Rc::new(RefCell::new(value)),
                    })))
                }
                _ => {
                    panic!(
                        "array#clone requires an array as the first argument on line {}",
                        unsafe { LINE }
                    );
                }
            }
        }
        "array#concat" => {
            if args.len() < 2 {
                panic!(
                    "array#concat requires at least 2 arguments on line {}",
                    unsafe { LINE }
                );
            }

            let mut result = Vec::new();

            for arg in args {
                let value = runtime.extract_value(arg)?;

                match value {
                    ValueToken::Array(array) => {
                        result.extend(array.value.borrow().iter().cloned());
                    }
                    _ => {
                        panic!(
                            "array#concat requires an array as each argument on line {}",
                            unsafe { LINE }
                        );
                    }
                }
            }

            Some(ExpressionToken::Value(ValueToken::Array(ArrayToken {
                location: Default::default(),
                value: Rc::new(RefCell::new(result)),
            })))
        }
        "array#contains" => {
            if args.len() != 2 {
                panic!("array#contains requires 2 arguments on line {}", unsafe {
                    LINE
                });
            }

            let value = runtime.extract_value(&args[0])?;
            match value {
                ValueToken::Array(array) => {
                    let target = runtime.extract_value(&args[1])?;

                    let contains = array.value.borrow().iter().any(|item| {
                        let item = runtime.extract_value(item).unwrap();
                        item.value() == target.value()
                    });

                    Some(ExpressionToken::Value(ValueToken::Boolean(BooleanToken {
                        location: Default::default(),
                        value: contains,
                    })))
                }
                _ => {
                    panic!(
                        "array#contains requires an array as the first argument on line {}",
                        unsafe { LINE }
                    );
                }
            }
        }
        "array#get" => {
            if args.len() != 2 {
                panic!("array#get requires 2 arguments on line {}", unsafe { LINE });
            }

            let value = runtime.extract_value(&args[0])?;
            match value {
                ValueToken::Array(array) => {
                    let index = runtime.extract_value(&args[1])?;
                    match index {
                        ValueToken::Number(number) => {
                            let index = number.value as usize;
                            let value = array.value.borrow().get(index).cloned().unwrap_or({
                                ExpressionToken::Value(ValueToken::Null(NullToken {
                                    location: Default::default(),
                                }))
                            });

                            Some(value)
                        }
                        _ => {
                            panic!(
                                "array#get requires a number as the second argument on line {}",
                                unsafe { LINE }
                            );
                        }
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
                                        location: Default::default(),
                                        value: c.to_string(),
                                    }))
                                })
                                .unwrap_or_else(|| {
                                    ExpressionToken::Value(ValueToken::Null(NullToken {
                                        location: Default::default(),
                                    }))
                                });

                            Some(value)
                        }
                        _ => {
                            panic!(
                                "array#get requires a number as the second argument on line {}",
                                unsafe { LINE }
                            );
                        }
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
                                location: Default::default(),
                                value: value == 1,
                            })))
                        }
                        _ => {
                            panic!(
                                "array#get requires a number as the second argument on line {}",
                                unsafe { LINE }
                            );
                        }
                    }
                }
                _ => {
                    panic!(
                        "array#get requires an array, string or number as the first argument on line {}",
                        unsafe { LINE }
                    );
                }
            }
        }
        "array#set" => {
            if args.len() != 3 {
                panic!("array#set requires 3 arguments on line {}", unsafe { LINE });
            }

            let value = runtime.extract_value(&args[0])?;
            match value {
                ValueToken::Array(array) => {
                    let index = runtime.extract_value(&args[1])?;
                    let value = runtime.extract_value(&args[2])?;

                    match index {
                        ValueToken::Number(number) => {
                            let index = number.value as usize;
                            let mut arr = array.value.borrow_mut();

                            if index >= arr.len() {
                                arr.resize(
                                    index + 1,
                                    ExpressionToken::Value(ValueToken::Null(NullToken {
                                        location: Default::default(),
                                    })),
                                );
                            }

                            arr[index] = ExpressionToken::Value(value);

                            Some(ExpressionToken::Value(ValueToken::Array(array.clone())))
                        }
                        _ => {
                            panic!(
                                "array#set requires a number as the second argument on line {}",
                                unsafe { LINE }
                            );
                        }
                    }
                }
                _ => {
                    panic!(
                        "array#set requires an array as the first argument on line {}",
                        unsafe { LINE }
                    );
                }
            }
        }
        _ => None,
    }
}
