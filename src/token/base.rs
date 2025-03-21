use std::sync::{Arc, Mutex, RwLock};

use super::{Token, TokenLocation, logic::ExpressionToken};

pub trait BaseToken {
    fn inspect(&self) -> String;
    fn value(&self) -> String;
    fn truthy(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct StringToken {
    pub value: String,

    pub location: TokenLocation,
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
    pub value: f64,

    pub location: TokenLocation,
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
    pub value: bool,

    pub location: TokenLocation,
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
    pub value: Arc<RwLock<Vec<ExpressionToken>>>,

    pub location: TokenLocation,
}

impl BaseToken for ArrayToken {
    fn inspect(&self) -> String {
        let mut result = format!("Array({}) {{\n", self.value.read().unwrap().len());

        for token in self.value.read().unwrap().iter() {
            if let ExpressionToken::Value(value_token) = token {
                result.push_str(&format!("{}\n", value_token.inspect()));
            }
        }

        result + "}"
    }

    fn value(&self) -> String {
        let mut result = "[\n".to_string();

        for token in self.value.read().unwrap().iter() {
            if let ExpressionToken::Value(value_token) = token {
                result.push_str(&format!("{}\n", value_token.value()));
            }
        }

        result + "]"
    }

    fn truthy(&self) -> bool {
        !self.value.read().unwrap().is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct BufferToken {
    pub value: Arc<RwLock<Vec<u8>>>,

    pub location: TokenLocation,
}

impl BaseToken for BufferToken {
    fn inspect(&self) -> String {
        let mut result = format!("Buffer({}) {{ ", self.value.read().unwrap().len());

        for byte in self.value.read().unwrap().iter().take(100) {
            result.push_str(&format!("{:02x} ", byte));
        }

        if self.value.read().unwrap().len() > 100 {
            result
                .push_str(format!("... {} more ", self.value.read().unwrap().len() - 100).as_str());
        }

        result + "}"
    }

    fn value(&self) -> String {
        let length = self.value.read().unwrap().len();

        self.value
            .read()
            .unwrap()
            .iter()
            .fold(String::with_capacity(length * 3), |acc, byte| {
                acc + &format!("{:02x} ", byte)
            })
    }

    fn truthy(&self) -> bool {
        !self.value.read().unwrap().is_empty()
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
    pub memory: Arc<Mutex<Box<dyn std::any::Any + Send>>>,
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
    pub body: Arc<RwLock<Vec<Token>>>,

    pub location: TokenLocation,
}

impl BaseToken for FunctionToken {
    fn inspect(&self) -> String {
        format!(
            "Function({}: {}) {{ <{} tokens> }}",
            self.name,
            self.args.join(", "),
            self.body.read().unwrap().len()
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
pub struct ClassToken {
    pub name: String,
    pub body: Arc<RwLock<Vec<Token>>>,

    pub location: TokenLocation,
}

impl BaseToken for ClassToken {
    fn inspect(&self) -> String {
        format!("Class({}) {{ }}", self.name)
    }

    fn value(&self) -> String {
        self.inspect()
    }

    fn truthy(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub struct ClassInstanceToken {
    pub class: Arc<RwLock<ClassToken>>,
}

impl BaseToken for ClassInstanceToken {
    fn inspect(&self) -> String {
        format!("ClassInstance({}) {{ }}", self.class.read().unwrap().name)
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
    Class(ClassToken),
    ClassInstance(ClassInstanceToken),
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
            ValueToken::Class(class_token) => class_token.inspect(),
            ValueToken::ClassInstance(class_instance_token) => class_instance_token.inspect(),
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
            ValueToken::Class(class_token) => class_token.value(),
            ValueToken::ClassInstance(class_instance_token) => class_instance_token.value(),
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
            ValueToken::Class(class_token) => class_token.truthy(),
            ValueToken::ClassInstance(class_instance_token) => class_instance_token.truthy(),
        }
    }
}

impl ValueToken {
    pub fn location(&self) -> TokenLocation {
        match self {
            ValueToken::String(token) => token.location.clone(),
            ValueToken::Number(token) => token.location.clone(),
            ValueToken::Boolean(token) => token.location.clone(),
            ValueToken::Null(token) => token.location.clone(),
            ValueToken::Array(token) => token.location.clone(),
            ValueToken::Buffer(token) => token.location.clone(),
            ValueToken::NativeMemory(_) => TokenLocation::default(),
            ValueToken::Function(token) => token.location.clone(),
            ValueToken::Class(token) => token.location.clone(),
            ValueToken::ClassInstance(_) => TokenLocation::default(),
        }
    }
}
