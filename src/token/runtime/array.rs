use crate::{
    runtime::Runtime,
    token::{
        TokenLocation,
        base::{
            ArrayToken, BaseToken, BooleanToken, NullToken, NumberToken, StringToken, ValueToken,
        },
        logic::ExpressionToken,
    },
};

use std::sync::{Arc, LazyLock, RwLock};

pub static FUNCTIONS: LazyLock<Vec<&str>> = LazyLock::new(|| {
    vec![
        "array#push",
        "array#pop",
        "array#len",
        "array#clone",
        "array#concat",
        "array#contains",
        "array#from",
        "array#get",
        "array#set",
    ]
});

pub fn run(
    name: &str,
    args: &[Arc<ExpressionToken>],
    runtime: &mut Runtime,
    location: &TokenLocation,
) -> Option<ExpressionToken> {
    match name {
        "array#push" => {
            if args.len() < 2 {
                panic!("array#push requires at least 2 arguments in {location}");
            }

            let value = runtime.extract_value(&args[0])?;
            match value {
                ValueToken::Array(array) => {
                    for arg in args.iter().skip(1) {
                        let value = runtime.extract_value(arg)?;
                        array
                            .value
                            .write()
                            .unwrap()
                            .push(ExpressionToken::Value(value));
                    }

                    Some(ExpressionToken::Value(ValueToken::Array(array.clone())))
                }
                _ => {
                    panic!("array#push requires an array as the first argument in {location}");
                }
            }
        }
        "array#pop" => {
            if args.len() != 1 {
                panic!("array#pop requires 1 argument in {location}");
            }

            let value = runtime.extract_value(&args[0])?;
            match value {
                ValueToken::Array(array) => {
                    let value =
                        array
                            .value
                            .write()
                            .unwrap()
                            .pop()
                            .unwrap_or(ExpressionToken::Value(ValueToken::Null(NullToken {
                                location: Default::default(),
                            })));

                    Some(value)
                }
                _ => {
                    panic!("array#pop requires an array as the first argument in {location}");
                }
            }
        }
        "array#len" => {
            if args.len() != 1 {
                panic!("array#len requires 1 argument in {location}");
            }

            let value = runtime.extract_value(&args[0])?;
            match value {
                ValueToken::Array(array) => {
                    let len = array.value.read().unwrap().len();

                    Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                        location: Default::default(),
                        value: len as f64,
                    })))
                }
                ValueToken::Range(range) => {
                    let start = runtime.extract_value(&range.start.read().unwrap())?;
                    let end = runtime.extract_value(&range.end.read().unwrap())?;

                    if let (ValueToken::Number(start), ValueToken::Number(end)) = (start, end) {
                        let len = (end.value - start.value).abs() as usize;

                        Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                            location: Default::default(),
                            value: len as f64,
                        })))
                    } else {
                        panic!(
                            "array#len requires a range with a set start & end as the first argument in {location}"
                        );
                    }
                }
                _ => {
                    panic!("array#len requires an array as the first argument in {location}");
                }
            }
        }
        "array#clone" => {
            if args.len() != 1 {
                panic!("array#clone requires 1 argument in {location}");
            }

            let value = runtime.extract_value(&args[0])?;
            match value {
                ValueToken::Array(array) => {
                    let value = array.value.read().unwrap().clone();
                    let mut new_value = Vec::new();

                    for item in value.iter() {
                        new_value
                            .push(ExpressionToken::Value(runtime.extract_value(item).unwrap()));
                    }

                    Some(ExpressionToken::Value(ValueToken::Array(ArrayToken {
                        location: Default::default(),
                        value: Arc::new(RwLock::new(new_value)),
                    })))
                }
                _ => {
                    panic!("array#clone requires an array as the first argument in {location}");
                }
            }
        }
        "array#concat" => {
            if args.len() < 2 {
                panic!("array#concat requires at least 2 arguments in {location}");
            }

            let mut result = Vec::new();

            for arg in args {
                let value = runtime.extract_value(arg)?;

                match value {
                    ValueToken::Array(array) => {
                        result.extend(array.value.read().unwrap().iter().cloned());
                    }
                    _ => {
                        panic!("array#concat requires an array as each argument in {location}");
                    }
                }
            }

            Some(ExpressionToken::Value(ValueToken::Array(ArrayToken {
                location: Default::default(),
                value: Arc::new(RwLock::new(result)),
            })))
        }
        "array#contains" => {
            if args.len() != 2 {
                panic!("array#contains requires 2 arguments in {location}");
            }

            let value = runtime.extract_value(&args[0])?;
            match value {
                ValueToken::Array(array) => {
                    let target = runtime.extract_value(&args[1])?;

                    let contains = array.value.read().unwrap().iter().any(|item| {
                        let item = runtime.extract_value(item).unwrap();
                        item.value(0) == target.value(0)
                    });

                    Some(ExpressionToken::Value(ValueToken::Boolean(BooleanToken {
                        location: Default::default(),
                        value: contains,
                    })))
                }
                _ => {
                    panic!("array#contains requires an array as the first argument in {location}");
                }
            }
        }
        "array#from" => {
            if args.len() != 1 {
                panic!("array#from requires 1 argument in {location}");
            }

            let value = runtime.extract_value(&args[0])?;
            match value {
                ValueToken::Array(array) => {
                    Some(ExpressionToken::Value(ValueToken::Array(array.clone())))
                }
                ValueToken::Range(range) => {
                    let start = runtime.extract_value(&range.start.read().unwrap())?;
                    let end = runtime.extract_value(&range.end.read().unwrap())?;

                    if let (ValueToken::Number(start), ValueToken::Number(end)) = (start, end) {
                        let start = start.value as usize;
                        let end = end.value as usize;

                        let mut new_value = Vec::new();
                        for i in start..end {
                            new_value.push(ExpressionToken::Value(ValueToken::Number(
                                NumberToken {
                                    location: Default::default(),
                                    value: i as f64,
                                },
                            )));
                        }

                        Some(ExpressionToken::Value(ValueToken::Array(ArrayToken {
                            location: Default::default(),
                            value: Arc::new(RwLock::new(new_value)),
                        })))
                    } else {
                        panic!(
                            "array#from requires a range with a set start & end as the first argument in {location}"
                        );
                    }
                }
                ValueToken::String(string) => {
                    let mut new_value = Vec::new();
                    for c in string.value.chars() {
                        new_value.push(ExpressionToken::Value(ValueToken::String(StringToken {
                            location: Default::default(),
                            value: c.to_string(),
                        })));
                    }

                    Some(ExpressionToken::Value(ValueToken::Array(ArrayToken {
                        location: Default::default(),
                        value: Arc::new(RwLock::new(new_value)),
                    })))
                }
                ValueToken::Number(num) => {
                    let mut new_value = Vec::new();
                    let integer = num.value as u64;

                    for i in 0..64 {
                        let value = (integer >> i) & 1;
                        new_value.push(ExpressionToken::Value(ValueToken::Boolean(BooleanToken {
                            location: Default::default(),
                            value: value == 1,
                        })));
                    }

                    Some(ExpressionToken::Value(ValueToken::Array(ArrayToken {
                        location: Default::default(),
                        value: Arc::new(RwLock::new(new_value)),
                    })))
                }
                ValueToken::Null(_) => {
                    Some(ExpressionToken::Value(ValueToken::Array(ArrayToken {
                        location: Default::default(),
                        value: Arc::new(RwLock::new(Vec::new())),
                    })))
                }
                _ => {
                    panic!(
                        "array#from requires an array, range, number or string as the first argument in {location}"
                    );
                }
            }
        }
        "array#get" => {
            if args.len() != 2 {
                panic!("array#get requires 2 arguments in {location}");
            }

            let value = runtime.extract_value(&args[0])?;
            match value {
                ValueToken::Array(array) => {
                    let index = runtime.extract_value(&args[1])?;
                    match index {
                        ValueToken::Number(number) => {
                            let index = number.value as usize;
                            let value =
                                array.value.read().unwrap().get(index).cloned().unwrap_or({
                                    ExpressionToken::Value(ValueToken::Null(NullToken {
                                        location: Default::default(),
                                    }))
                                });

                            Some(value)
                        }
                        _ => {
                            panic!(
                                "array#get requires a number as the second argument in {location}"
                            );
                        }
                    }
                }
                ValueToken::Range(range) => {
                    let start = runtime.extract_value(&range.start.read().unwrap())?;
                    let end = runtime.extract_value(&range.end.read().unwrap())?;

                    if let (ValueToken::Number(start), ValueToken::Number(end)) = (start, end) {
                        let start = start.value as usize;
                        let end = end.value as usize;

                        // get number from (start..end) at provided index
                        let index = runtime.extract_value(&args[1])?;
                        match index {
                            ValueToken::Number(number) => {
                                let index = number.value as usize;
                                let value = start + index;

                                if value > end {
                                    Some(ExpressionToken::Value(ValueToken::Null(NullToken {
                                        location: Default::default(),
                                    })))
                                } else {
                                    Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                                        location: Default::default(),
                                        value: value as f64,
                                    })))
                                }
                            }
                            _ => {
                                panic!(
                                    "array#get requires a number as the second argument in {location}"
                                );
                            }
                        }
                    } else {
                        panic!(
                            "array#get requires a range with a set start & end as the first argument in {location}"
                        );
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
                                "array#get requires a number as the second argument in {location}"
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
                                "array#get requires a number as the second argument in {location}"
                            );
                        }
                    }
                }
                _ => {
                    panic!(
                        "array#get requires an array, string or number as the first argument in {location}"
                    );
                }
            }
        }
        "array#set" => {
            if args.len() != 3 {
                panic!("array#set requires 3 arguments in {location}");
            }

            let value = runtime.extract_value(&args[0])?;
            match value {
                ValueToken::Array(array) => {
                    let index = runtime.extract_value(&args[1])?;
                    let value = runtime.extract_value(&args[2])?;

                    match index {
                        ValueToken::Number(number) => {
                            let index = number.value as usize;
                            let mut arr = array.value.write().unwrap();

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
                                "array#set requires a number as the second argument in {location}"
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
                            let bit = runtime.extract_value(&args[2])?;
                            let bit = match bit {
                                ValueToken::Boolean(boolean) => {
                                    if boolean.value {
                                        1
                                    } else {
                                        0
                                    }
                                }
                                _ => {
                                    panic!(
                                        "array#set requires a boolean as the third argument in {location}"
                                    );
                                }
                            };

                            let mask = 1 << index;
                            let value = (integer & !mask) | (bit << index);

                            Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                                location: Default::default(),
                                value: value as f64,
                            })))
                        }
                        _ => {
                            panic!(
                                "array#set requires a number as the second argument in {location}"
                            );
                        }
                    }
                }
                _ => {
                    panic!("array#set requires an array as the first argument in {location}");
                }
            }
        }
        _ => None,
    }
}
