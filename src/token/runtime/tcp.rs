use crate::{
    runtime::Runtime,
    token::{
        TokenLocation,
        base::{BaseToken, BufferToken, NativeMemoryToken, NullToken, StringToken, ValueToken},
        logic::ExpressionToken,
    },
};

use std::{
    io::{Read, Write},
    sync::{Arc, LazyLock, Mutex, RwLock},
};

pub static FUNCTIONS: LazyLock<Vec<&str>> = LazyLock::new(|| {
    vec![
        "tcp#bind",
        "tcp#getconn",
        "tcp#readstr",
        "tcp#readbin",
        "tcp#write",
        "tcp#close",
    ]
});

pub fn run(
    name: &str,
    args: &[Arc<ExpressionToken>],
    runtime: &mut Runtime,
    location: &TokenLocation,
) -> Option<ExpressionToken> {
    match name {
        "tcp#bind" => {
            if args.len() != 2 {
                panic!("tcp#bind requires 2 arguments in {}", location);
            }

            let address = runtime.extract_value(&args[0])?;
            let port = runtime.extract_value(&args[1])?;

            match (address, port) {
                (ValueToken::String(address), ValueToken::Number(port)) => {
                    let listener =
                        std::net::TcpListener::bind(format!("{}:{}", address.value, port.value))
                            .unwrap();

                    Some(ExpressionToken::Value(ValueToken::NativeMemory(
                        NativeMemoryToken {
                            name: "TcpListener".to_string(),
                            memory: Arc::new(Mutex::new(Box::new(listener))),
                        },
                    )))
                }
                _ => {
                    panic!("tcp#bind requires a string and a number in {}", location);
                }
            }
        }
        "tcp#getconn" => {
            if args.len() != 1 {
                panic!("tcp#getconn requires 1 argument in {}", location);
            }

            let listener = runtime.extract_value(&args[0]);
            if let Some(ValueToken::NativeMemory(listener)) = listener {
                let listener = listener.memory.lock().unwrap();
                let listener = listener
                    .as_ref()
                    .downcast_ref::<std::net::TcpListener>()
                    .unwrap();

                let (stream, _) = listener.accept().unwrap();

                Some(ExpressionToken::Value(ValueToken::NativeMemory(
                    NativeMemoryToken {
                        name: "TcpStream".to_string(),
                        memory: Arc::new(Mutex::new(Box::new(stream))),
                    },
                )))
            } else {
                panic!("tcp#getconn requires a TcpListener in {}", location);
            }
        }
        "tcp#readstr" => {
            if args.is_empty() || args.len() > 2 {
                panic!(
                    "tcp#readstr requires at least 1 argument and at most 2 arguments in {}",
                    location
                );
            }

            let stream = runtime.extract_value(&args[0]);
            let length = if args.len() == 2 {
                runtime.extract_value(&args[1])
            } else {
                None
            };

            if let Some(ValueToken::NativeMemory(stream)) = stream {
                let stream = stream.memory.lock().unwrap();
                let mut stream = stream
                    .as_ref()
                    .downcast_ref::<std::net::TcpStream>()
                    .unwrap();

                let length = if let Some(ValueToken::Number(length)) = length {
                    length.value as usize
                } else {
                    1024
                };

                let mut buffer = vec![0; length];
                let read = stream.read(&mut buffer).unwrap();

                let result = String::from_utf8_lossy(&buffer[..read]).to_string();

                Some(ExpressionToken::Value(ValueToken::String(StringToken {
                    location: Default::default(),
                    value: result,
                })))
            } else {
                panic!("tcp#read requires a TcpStream in {}", location);
            }
        }
        "tcp#readbin" => {
            if args.is_empty() || args.len() > 2 {
                panic!(
                    "tcp#readbin requires at least 1 argument and at most 2 arguments in {}",
                    location
                );
            }

            let stream = runtime.extract_value(&args[0]);
            let length = if args.len() == 2 {
                runtime.extract_value(&args[1])
            } else {
                None
            };

            if let Some(ValueToken::NativeMemory(stream)) = stream {
                let stream = stream.memory.lock().unwrap();
                let mut stream = stream
                    .as_ref()
                    .downcast_ref::<std::net::TcpStream>()
                    .unwrap();

                let length = if let Some(ValueToken::Number(length)) = length {
                    length.value as usize
                } else {
                    1024
                };

                let mut buffer = vec![0; length];
                let read = stream.read(&mut buffer).unwrap();

                let result = buffer[..read].to_vec();

                Some(ExpressionToken::Value(ValueToken::Buffer(BufferToken {
                    location: Default::default(),
                    value: Arc::new(RwLock::new(result)),
                })))
            } else {
                panic!("tcp#read requires a TcpStream in {}", location);
            }
        }
        "tcp#write" => {
            if args.len() != 2 {
                panic!("tcp#write requires 2 arguments in {}", location);
            }

            let stream = runtime.extract_value(&args[0]);
            let data = runtime.extract_value(&args[1]);

            if let Some(ValueToken::NativeMemory(stream)) = stream {
                let stream = stream.memory.lock().unwrap();
                let mut stream = stream
                    .as_ref()
                    .downcast_ref::<std::net::TcpStream>()
                    .unwrap();

                let data = match data {
                    Some(data) => data.value(0).to_string(),
                    _ => panic!(
                        "tcp#write requires a value as the second argument in {}",
                        location
                    ),
                };

                stream.write_all(data.as_bytes()).unwrap();

                Some(ExpressionToken::Value(ValueToken::Null(NullToken {
                    location: Default::default(),
                })))
            } else {
                panic!("tcp#write requires a TcpStream in {}", location);
            }
        }
        _ => None,
    }
}
