use crate::{
    runtime::Runtime,
    token::{
        Token, TokenLocation,
        base::{NativeMemoryToken, NullToken, ValueToken},
        logic::{ExpressionToken, FnCallToken, LetToken},
    },
};

use std::sync::{Arc, LazyLock, Mutex, RwLock};

pub static FUNCTIONS: LazyLock<Vec<&str>> = LazyLock::new(|| vec!["thread#launch", "thread#join"]);

pub fn run(
    name: &str,
    args: &[Arc<ExpressionToken>],
    runtime: &mut Runtime,
    location: &TokenLocation,
) -> Option<ExpressionToken> {
    match name {
        "thread#launch" => {
            if args.is_empty() {
                panic!("thread#launch requires at least 1 argument in {}", location);
            }

            let function = runtime.extract_value(&args[0])?;
            match function {
                ValueToken::Function(fn_token) => {
                    let function = fn_token.clone();
                    let args: Vec<_> = args[1..]
                        .iter()
                        .map(|arg| runtime.extract_value(arg).unwrap())
                        .collect();

                    let scope = runtime.scope_aggregate();
                    let mut var_tokens = Vec::new();

                    for variable in scope.iter() {
                        let value = variable.1.read().unwrap();

                        var_tokens.push(Token::Let(LetToken {
                            name: variable.0.clone(),
                            is_const: false,
                            is_function: matches!(
                                runtime.extract_value(&value).unwrap(),
                                ValueToken::Function(_)
                            ),
                            is_class: matches!(
                                runtime.extract_value(&value).unwrap(),
                                ValueToken::Class(_)
                            ),
                            value: Arc::clone(variable.1),
                            location: Default::default(),
                        }));
                    }

                    let thread = std::thread::spawn(move || {
                        let mut tokens = Vec::new();

                        for variable in var_tokens {
                            tokens.push(variable);
                        }

                        tokens.push(Token::Let(LetToken {
                            name: "main".to_string(),
                            is_const: true,
                            is_function: true,
                            is_class: false,
                            value: Arc::new(RwLock::new(ExpressionToken::Value(
                                ValueToken::Function(function),
                            ))),
                            location: Default::default(),
                        }));

                        tokens.push(Token::FnCall(FnCallToken {
                            name: "main".to_string(),
                            class: None,
                            class_instance: None,
                            args: args
                                .iter()
                                .map(|arg| Arc::new(ExpressionToken::Value(arg.clone())))
                                .collect(),
                            location: Default::default(),
                        }));

                        let mut runtime = Runtime::new(tokens);
                        runtime.run();
                    });

                    Some(ExpressionToken::Value(ValueToken::NativeMemory(
                        NativeMemoryToken {
                            name: "Thread".to_string(),
                            memory: Arc::new(Mutex::new(Box::new(thread))),
                        },
                    )))
                }
                _ => {
                    panic!("thread#launch requires a function in {}", location);
                }
            }
        }
        "thread#join" => {
            if args.len() != 1 {
                panic!("thread#join requires 1 argument in {}", location);
            }

            let thread = runtime.extract_value(&args[0]);
            if let Some(ValueToken::NativeMemory(thread)) = thread {
                let mut thread_guard = thread.memory.lock().unwrap();

                let thread_box = std::mem::replace(
                    &mut *thread_guard,
                    Box::new(()) as Box<dyn std::any::Any + Send + Sync>,
                );

                let thread = thread_box
                    .downcast::<std::thread::JoinHandle<()>>()
                    .unwrap();
                thread.join().unwrap();

                Some(ExpressionToken::Value(ValueToken::Null(NullToken {
                    location: Default::default(),
                })))
            } else {
                panic!("thread#kill requires a Thread in {}", location);
            }
        }
        _ => None,
    }
}
