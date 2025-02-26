use std::sync::{Arc, Mutex};

use super::{Token, TokenLocation, logic::ExpressionToken};

pub trait BaseToken {
    fn inspect(&self) -> String;
    fn value(&self) -> String;
    fn truthy(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct StringToken {
    pub location: TokenLocation,
    pub value: String,
}

impl BaseToken for StringToken {
    fn inspect(&self) -> String {
        format!("String({}) {{ {} }}", self.value.len(), self.value)
    }

    fn value(&self) -> String {
        self.value.clone()
    }

    fn truthy(&self) -> bool {
        !self.value.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct NumberToken {
    pub location: TokenLocation,
    pub value: f64,
}

impl BaseToken for NumberToken {
    fn inspect(&self) -> String {
        format!("Number(f64) {{ {} }}", self.value)
    }

    fn value(&self) -> String {
        self.value.to_string()
    }

    fn truthy(&self) -> bool {
        self.value != 0.0
    }
}

#[derive(Debug, Clone)]
pub struct BooleanToken {
    pub location: TokenLocation,
    pub value: bool,
}

impl BaseToken for BooleanToken {
    fn inspect(&self) -> String {
        format!("Boolean(bool) {{ {} }}", self.value)
    }

    fn value(&self) -> String {
        self.value.to_string()
    }

    fn truthy(&self) -> bool {
        self.value
    }
}

#[derive(Debug, Clone)]
pub struct ArrayToken {
    pub location: TokenLocation,
    pub value: Arc<Mutex<Vec<ExpressionToken>>>,
}

impl BaseToken for ArrayToken {
    fn inspect(&self) -> String {
        let mut result = format!("Array({}) {{\n", self.value.lock().unwrap().len());

        for token in self.value.lock().unwrap().iter() {
            if let ExpressionToken::Value(value_token) = token {
                result.push_str(&format!("{}\n", value_token.inspect()));
            }
        }

        result + "}"
    }

    fn value(&self) -> String {
        let mut result = "[\n".to_string();

        for token in self.value.lock().unwrap().iter() {
            if let ExpressionToken::Value(value_token) = token {
                result.push_str(&format!("{}\n", value_token.value()));
            }
        }

        result + "]"
    }

    fn truthy(&self) -> bool {
        !self.value.lock().unwrap().is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct BufferToken {
    pub location: TokenLocation,
    pub value: Arc<Mutex<Vec<u8>>>,
}

impl BaseToken for BufferToken {
    fn inspect(&self) -> String {
        let mut result = format!("Buffer({}) {{ ", self.value.lock().unwrap().len());

        for byte in self.value.lock().unwrap().iter().take(100) {
            result.push_str(&format!("{:02x} ", byte));
        }

        if self.value.lock().unwrap().len() > 100 {
            result
                .push_str(format!("... {} more ", self.value.lock().unwrap().len() - 100).as_str());
        }

        result + "}"
    }

    fn value(&self) -> String {
        let length = self.value.lock().unwrap().len();

        self.value
            .lock()
            .unwrap()
            .iter()
            .fold(String::with_capacity(length * 3), |acc, byte| {
                acc + &format!("{:02x} ", byte)
            })
    }

    fn truthy(&self) -> bool {
        !self.value.lock().unwrap().is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct NullToken {
    pub location: TokenLocation,
}

impl BaseToken for NullToken {
    fn inspect(&self) -> String {
        "Null".to_string()
    }

    fn value(&self) -> String {
        "null".to_string()
    }

    fn truthy(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
pub struct NativeMemoryToken {
    pub name: String,
    pub memory: Arc<Mutex<Box<dyn std::any::Any>>>,
}

impl BaseToken for NativeMemoryToken {
    fn inspect(&self) -> String {
        format!("NativeMemory({}) {{ <native> }}", self.name)
    }

    fn value(&self) -> String {
        self.inspect()
    }

    fn truthy(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub struct FunctionToken {
    pub name: String,
    pub args: Vec<String>,
    pub body: Arc<Mutex<Vec<Token>>>,
}

impl BaseToken for FunctionToken {
    fn inspect(&self) -> String {
        format!(
            "Function({}, {}) {{ <{} tokens> }}",
            self.name,
            self.args.join(", "),
            self.body.lock().unwrap().len()
        )
    }

    fn value(&self) -> String {
        self.inspect()
    }

    fn truthy(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub enum ValueToken {
    String(StringToken),
    Number(NumberToken),
    Boolean(BooleanToken),
    Null(NullToken),
    Array(ArrayToken),
    Buffer(BufferToken),
    NativeMemory(NativeMemoryToken),
    Function(FunctionToken),
}

impl BaseToken for ValueToken {
    fn inspect(&self) -> String {
        match self {
            ValueToken::String(string_token) => string_token.inspect(),
            ValueToken::Number(number_token) => number_token.inspect(),
            ValueToken::Boolean(boolean_token) => boolean_token.inspect(),
            ValueToken::Null(null_token) => null_token.inspect(),
            ValueToken::Array(array_token) => array_token.inspect(),
            ValueToken::Buffer(buffer_token) => buffer_token.inspect(),
            ValueToken::NativeMemory(native_memory_token) => native_memory_token.inspect(),
            ValueToken::Function(function_token) => function_token.inspect(),
        }
    }

    fn value(&self) -> String {
        match self {
            ValueToken::String(string_token) => string_token.value(),
            ValueToken::Number(number_token) => number_token.value(),
            ValueToken::Boolean(boolean_token) => boolean_token.value(),
            ValueToken::Null(null_token) => null_token.value(),
            ValueToken::Array(array_token) => array_token.value(),
            ValueToken::Buffer(buffer_token) => buffer_token.value(),
            ValueToken::NativeMemory(native_memory_token) => native_memory_token.value(),
            ValueToken::Function(function_token) => function_token.value(),
        }
    }

    fn truthy(&self) -> bool {
        match self {
            ValueToken::String(string_token) => string_token.truthy(),
            ValueToken::Number(number_token) => number_token.truthy(),
            ValueToken::Boolean(boolean_token) => boolean_token.truthy(),
            ValueToken::Null(null_token) => null_token.truthy(),
            ValueToken::Array(array_token) => array_token.truthy(),
            ValueToken::Buffer(buffer_token) => buffer_token.truthy(),
            ValueToken::NativeMemory(native_memory_token) => native_memory_token.truthy(),
            ValueToken::Function(function_token) => function_token.truthy(),
        }
    }
}
