use crate::{
    runtime::Runtime,
    token::{
        LINE,
        base::{NullToken, NumberToken, ValueToken},
        logic::ExpressionToken,
    },
};

use std::{rc::Rc, sync::LazyLock};

pub static FUNCTIONS: LazyLock<Vec<&str>> = LazyLock::new(|| vec!["time#sleep", "time#now"]);

pub fn run(
    name: &str,
    args: &[Rc<ExpressionToken>],
    runtime: &mut Runtime,
) -> Option<ExpressionToken> {
    match name {
        "time#sleep" => {
            if args.len() != 1 {
                panic!("time#sleep requires 1 argument on line {}", unsafe { LINE });
            }

            let value = runtime.extract_value(&args[0])?;
            let seconds = match value {
                ValueToken::Number(value) => value.value,
                _ => panic!("time#sleep requires a number on line {}", unsafe { LINE }),
            };

            std::thread::sleep(std::time::Duration::from_millis((seconds * 1000.0) as u64));	
            Some(ExpressionToken::Value(ValueToken::Null(NullToken)))
        }
        "time#now" => {
            if !args.is_empty() {
                panic!("time#now requires no arguments on line {}", unsafe { LINE });
            }

            let unix_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                value: unix_time as f64,
            })))
        }
        _ => None,
    }
}
