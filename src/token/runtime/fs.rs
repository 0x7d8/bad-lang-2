use crate::{
    runtime::Runtime,
    token::LINE,
    token::{
        base::{BaseToken, StringToken, ValueToken},
        logic::ExpressionToken,
    },
};

use std::{
    io::Read,
    sync::{Arc, LazyLock},
};

pub static FUNCTIONS: LazyLock<Vec<&str>> =
    LazyLock::new(|| vec!["fs#readstr", "fs#readstr_until"]);

pub fn run(
    name: &str,
    args: &[Arc<ExpressionToken>],
    runtime: &mut Runtime,
) -> Option<ExpressionToken> {
    match name {
        "fs#readstr" => {
            if args.len() != 1 {
                panic!("fs#readstr requires 1 argument on line {}", unsafe { LINE });
            }

            let value = runtime.extract_value(&args[0])?;
            let path = value.value().to_string();
            let content = std::fs::read_to_string(path).unwrap();

            Some(ExpressionToken::Value(ValueToken::String(StringToken {
                location: Default::default(),
                value: content,
            })))
        }
        "fs#readstr_until" => {
            if args.len() != 2 {
                panic!("fs#readstr_until requires 2 arguments on line {}", unsafe {
                    LINE
                });
            }

            let value = runtime.extract_value(&args[0])?;
            let bytes = runtime.extract_value(&args[1])?;

            match (value, bytes) {
                (ValueToken::String(value), ValueToken::Number(bytes)) => {
                    let path = value.value;
                    let bytes = bytes.value;

                    let mut file = std::fs::File::open(path).unwrap();
                    let mut content = String::new();

                    loop {
                        let mut buffer = vec![0; 1024];
                        let read = file.read(&mut buffer).unwrap();

                        if read == 0 {
                            break;
                        }

                        if read > bytes as usize {
                            content.push_str(&String::from_utf8_lossy(&buffer[..bytes as usize]));
                            break;
                        }

                        content.push_str(&String::from_utf8_lossy(&buffer[..read]));
                    }

                    Some(ExpressionToken::Value(ValueToken::String(StringToken {
                        location: Default::default(),
                        value: content,
                    })))
                }
                _ => {
                    panic!(
                        "fs#readstr_until requires a string and a number on line {}",
                        unsafe { LINE }
                    );
                }
            }
        }
        _ => None,
    }
}
