use crate::{
    runtime::Runtime,
    token::{
        TokenLocation,
        base::{BaseToken, BufferToken, StringToken, ValueToken},
        logic::ExpressionToken,
    },
};

use std::{
    io::Read,
    sync::{Arc, LazyLock, RwLock},
};

pub static FUNCTIONS: LazyLock<Vec<&str>> =
    LazyLock::new(|| vec!["fs#readstr", "fs#readbin", "fs#write"]);

pub fn run(
    name: &str,
    args: &[Arc<ExpressionToken>],
    runtime: &mut Runtime,
    location: &TokenLocation,
) -> Option<ExpressionToken> {
    match name {
        "fs#readstr" => {
            if args.len() != 1 {
                panic!("fs#readstr requires 1 argument in {}", location);
            }

            let value = runtime.extract_value(&args[0])?;
            let path = value.value(0).to_string();
            let content = std::fs::read_to_string(path).unwrap();

            Some(ExpressionToken::Value(ValueToken::String(StringToken {
                location: Default::default(),
                value: content,
            })))
        }
        "fs#readbin" => {
            if args.len() != 1 {
                panic!("fs#readbin requires 1 argument in {}", location);
            }

            let value = runtime.extract_value(&args[0])?;
            let path = value.value(0).to_string();
            let mut content = Vec::new();
            std::fs::File::open(path)
                .unwrap()
                .read_to_end(&mut content)
                .unwrap();

            Some(ExpressionToken::Value(ValueToken::Buffer(BufferToken {
                location: Default::default(),
                value: Arc::new(RwLock::new(content)),
            })))
        }
        "fs#write" => {
            if args.len() != 2 {
                panic!("fs#write requires 2 arguments in {}", location);
            }

            let value = runtime.extract_value(&args[0])?;
            let path = value.value(0).to_string();
            let content = runtime.extract_value(&args[1])?;

            std::fs::write(path, content.value(0)).unwrap();

            Some(ExpressionToken::Value(ValueToken::String(StringToken {
                location: Default::default(),
                value: "ok".to_string(),
            })))
        }
        _ => None,
    }
}
