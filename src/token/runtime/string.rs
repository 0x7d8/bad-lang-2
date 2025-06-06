use crate::{
    runtime::Runtime,
    token::{
        TokenLocation,
        base::{ArrayToken, BaseToken, BooleanToken, NumberToken, StringToken, ValueToken},
        logic::ExpressionToken,
    },
};

use std::sync::{Arc, LazyLock, RwLock};

pub static FUNCTIONS: LazyLock<Vec<&str>> = LazyLock::new(|| {
    vec![
        "string#concat",
        "string#format",
        "string#len",
        "string#split",
        "string#trim",
        "string#to_number",
        "string#replace",
        "string#replacen",
        "string#index_of",
        "string#slice",
        "string#to_upper",
        "string#to_lower",
        "string#starts_with",
        "string#ends_with",
        "string#contains",
    ]
});

pub fn run(
    name: &str,
    args: &[Arc<ExpressionToken>],
    runtime: &mut Runtime,
    location: &TokenLocation,
) -> Option<ExpressionToken> {
    match name {
        "string#concat" => {
            if args.len() < 2 {
                panic!("string#concat requires 2 arguments in {}", location);
            }

            let mut result = String::new();

            for arg in args {
                let value = runtime.extract_value(arg)?;

                result.push_str(&value.value(0));
            }

            Some(ExpressionToken::Value(ValueToken::String(StringToken {
                location: Default::default(),
                value: result,
            })))
        }
        "string#format" => {
            if args.is_empty() {
                panic!("string#format requires at least 1 argument in {}", location);
            }

            let format = runtime.extract_value(&args[0])?.value(0).to_string();
            let mut values = Vec::new();

            for arg in args.iter().skip(1) {
                let value = runtime.extract_value(arg)?;

                values.push(value.value(0));
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
                panic!("string#len requires 1 argument in {}", location);
            }

            let value = runtime.extract_value(&args[0])?;
            let len = value.value(0).len();

            Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                location: Default::default(),
                value: len as f64,
            })))
        }
        "string#split" => {
            if args.len() != 2 {
                panic!("string#split requires 2 arguments in {}", location);
            }

            let value = runtime.extract_value(&args[0])?;
            let separator = runtime.extract_value(&args[1])?;

            let value = value.value(0);
            let separator = separator.value(0);

            Some(ExpressionToken::Value(ValueToken::Array(ArrayToken {
                location: Default::default(),
                value: Arc::new(RwLock::new(
                    value
                        .split(&separator)
                        .map(|s| {
                            ExpressionToken::Value(ValueToken::String(StringToken {
                                location: Default::default(),
                                value: s.to_string(),
                            }))
                        })
                        .collect(),
                )),
            })))
        }
        "string#trim" => {
            if args.len() != 1 {
                panic!("string#trim requires 1 argument in {}", location);
            }

            let value = runtime.extract_value(&args[0])?;
            let value = value.value(0);

            Some(ExpressionToken::Value(ValueToken::String(StringToken {
                location: Default::default(),
                value: value.trim().to_string(),
            })))
        }
        "string#to_number" => {
            if args.len() != 1 {
                panic!("string#to_number requires 1 argument in {}", location);
            }

            let value = runtime.extract_value(&args[0])?;
            let value = value.value(0);

            let value = value.parse::<f64>().unwrap();

            Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                location: Default::default(),
                value,
            })))
        }
        "string#replace" => {
            if args.len() != 3 {
                panic!("string#replace requires 3 arguments in {}", location);
            }

            let value = runtime.extract_value(&args[0])?;
            let search = runtime.extract_value(&args[1])?;
            let replace = runtime.extract_value(&args[2])?;

            let value = value.value(0);
            let search = search.value(0);
            let replace = replace.value(0);

            Some(ExpressionToken::Value(ValueToken::String(StringToken {
                location: Default::default(),
                value: value.replace(&search, &replace),
            })))
        }
        "string#replacen" => {
            if args.len() != 4 {
                panic!("string#replacen requires 4 arguments in {}", location);
            }

            let value = runtime.extract_value(&args[0])?;
            let search = runtime.extract_value(&args[1])?;
            let replace = runtime.extract_value(&args[2])?;
            let n = runtime.extract_value(&args[3])?;

            let value = value.value(0);
            let search = search.value(0);
            let replace = replace.value(0);
            let n = match n {
                ValueToken::Number(n) => n.value as usize,
                _ => panic!(
                    "string#replacen requires a number as the last argument in {}",
                    location
                ),
            };

            Some(ExpressionToken::Value(ValueToken::String(StringToken {
                location: Default::default(),
                value: value.replacen(&search, &replace, n),
            })))
        }
        "string#index_of" => {
            if args.len() != 2 {
                panic!("string#index_of requires 2 arguments in {}", location);
            }

            let value = runtime.extract_value(&args[0])?;
            let search = runtime.extract_value(&args[1])?;

            let value = value.value(0);
            let search = search.value(0);

            let index = value.find(&search).unwrap_or(value.len());

            Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                location: Default::default(),
                value: index as f64,
            })))
        }
        "string#slice" => {
            if args.len() != 3 {
                panic!("string#slice requires 3 arguments in {}", location);
            }

            let value = runtime.extract_value(&args[0])?;
            let start = runtime.extract_value(&args[1])?;
            let end = runtime.extract_value(&args[2])?;

            let value = value.value(0);
            match (start, end) {
                (ValueToken::Number(start), ValueToken::Number(end)) => {
                    let start = start.value as usize;
                    let end = end.value as usize;

                    Some(ExpressionToken::Value(ValueToken::String(StringToken {
                        location: Default::default(),
                        value: value[start..end].to_string(),
                    })))
                }
                _ => panic!(
                    "string#slice requires 2 numbers as the last 2 arguments in {}",
                    location
                ),
            }
        }
        "string#to_upper" => {
            if args.len() != 1 {
                panic!("string#to_upper requires 1 argument in {}", location);
            }

            let value = runtime.extract_value(&args[0])?;
            let value = value.value(0);

            Some(ExpressionToken::Value(ValueToken::String(StringToken {
                location: Default::default(),
                value: value.to_uppercase(),
            })))
        }
        "string#to_lower" => {
            if args.len() != 1 {
                panic!("string#to_lower requires 1 argument in {}", location);
            }

            let value = runtime.extract_value(&args[0])?;
            let value = value.value(0);

            Some(ExpressionToken::Value(ValueToken::String(StringToken {
                location: Default::default(),
                value: value.to_lowercase(),
            })))
        }
        "string#starts_with" => {
            if args.len() != 2 {
                panic!("string#starts_with requires 2 arguments in {}", location);
            }

            let value = runtime.extract_value(&args[0])?;
            let search = runtime.extract_value(&args[1])?;

            let value = value.value(0);
            let search = search.value(0);

            Some(ExpressionToken::Value(ValueToken::Boolean(BooleanToken {
                location: Default::default(),
                value: value.starts_with(&search),
            })))
        }
        "string#ends_with" => {
            if args.len() != 2 {
                panic!("string#ends_with requires 2 arguments in {}", location);
            }

            let value = runtime.extract_value(&args[0])?;
            let search = runtime.extract_value(&args[1])?;

            let value = value.value(0);
            let search = search.value(0);

            Some(ExpressionToken::Value(ValueToken::Boolean(BooleanToken {
                location: Default::default(),
                value: value.ends_with(&search),
            })))
        }
        "string#contains" => {
            if args.len() != 2 {
                panic!("string#contains requires 2 arguments in {}", location);
            }

            let value = runtime.extract_value(&args[0])?;
            let search = runtime.extract_value(&args[1])?;

            let value = value.value(0);
            let search = search.value(0);

            Some(ExpressionToken::Value(ValueToken::Boolean(BooleanToken {
                location: Default::default(),
                value: value.contains(&search),
            })))
        }
        _ => None,
    }
}
