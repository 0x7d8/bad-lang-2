use crate::{
    runtime::Runtime,
    token::{
        LINE, Token,
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
) -> Option<ExpressionToken> {
    match name {
        "thread#launch" => {
            if args.is_empty() {
                panic!(
                    "thread#launch requires at least 1 argument on line {}",
                    unsafe { LINE }
                );
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
                            is_function: match runtime.extract_value(&value).unwrap() {
                                ValueToken::Function(_) => true,
                                _ => false,
                            },
                            value: Arc::clone(variable.1),
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
                            value: Arc::new(RwLock::new(ExpressionToken::Value(
                                ValueToken::Function(function),
                            ))),
                        }));

                        tokens.push(Token::FnCall(FnCallToken {
                            name: "main".to_string(),
                            args: args
                                .iter()
                                .map(|arg| Arc::new(ExpressionToken::Value(arg.clone())))
                                .collect(),
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
                    panic!("thread#launch requires a function on line {}", unsafe {
                        LINE
                    });
                }
            }
        }
        "thread#join" => {
            if args.len() != 1 {
                panic!("thread#join requires 1 argument on line {}", unsafe {
                    LINE
                });
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
                panic!("thread#kill requires a Thread on line {}", unsafe { LINE });
            }
        }
        _ => None,
    }
}
