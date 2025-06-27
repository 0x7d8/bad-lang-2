use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use super::{Token, TokenLocation, logic::ExpressionToken};

pub trait BaseToken: PartialEq<ValueToken> + PartialEq<Self> {
    fn inspect(&self) -> String;
    fn value(&self, spaces: usize) -> String;
    fn truthy(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct StringToken {
    pub value: String,

    pub location: TokenLocation,
}

impl PartialEq<ValueToken> for StringToken {
    fn eq(&self, other: &ValueToken) -> bool {
        if let ValueToken::String(other) = other {
            self.value == other.value
        } else {
            false
        }
    }
}

impl PartialEq<StringToken> for StringToken {
    fn eq(&self, other: &StringToken) -> bool {
        self.value == other.value
    }
}

impl BaseToken for StringToken {
    fn inspect(&self) -> String {
        format!("String({}) {{ {} }}", self.value.len(), self.value)
    }

    fn value(&self, spaces: usize) -> String {
        " ".repeat(spaces) + &self.value
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

impl PartialEq<ValueToken> for NumberToken {
    fn eq(&self, other: &ValueToken) -> bool {
        if let ValueToken::Number(other) = other {
            self.value == other.value
        } else {
            false
        }
    }
}

impl PartialEq<NumberToken> for NumberToken {
    fn eq(&self, other: &NumberToken) -> bool {
        self.value == other.value
    }
}

impl BaseToken for NumberToken {
    fn inspect(&self) -> String {
        format!("Number(f64) {{ {} }}", self.value)
    }

    fn value(&self, spaces: usize) -> String {
        " ".repeat(spaces) + &self.value.to_string()
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

impl PartialEq<ValueToken> for BooleanToken {
    fn eq(&self, other: &ValueToken) -> bool {
        if let ValueToken::Boolean(other) = other {
            self.value == other.value
        } else {
            false
        }
    }
}

impl PartialEq<BooleanToken> for BooleanToken {
    fn eq(&self, other: &BooleanToken) -> bool {
        self.value == other.value
    }
}

impl BaseToken for BooleanToken {
    fn inspect(&self) -> String {
        format!("Boolean(bool) {{ {} }}", self.value)
    }

    fn value(&self, spaces: usize) -> String {
        " ".repeat(spaces) + &self.value.to_string()
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

impl PartialEq<ValueToken> for ArrayToken {
    fn eq(&self, _other: &ValueToken) -> bool {
        false
    }
}

impl PartialEq<ArrayToken> for ArrayToken {
    fn eq(&self, _other: &ArrayToken) -> bool {
        false
    }
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

    fn value(&self, spaces: usize) -> String {
        let mut result = " ".repeat(spaces) + "[\n";

        for token in self.value.read().unwrap().iter() {
            if let ExpressionToken::Value(value_token) = token {
                result.push_str(&format!("{}\n", value_token.value(spaces + 2)));
            }
        }

        result.push_str(" ".repeat(spaces).as_str());
        result.push(']');

        result
    }

    fn truthy(&self) -> bool {
        !self.value.read().unwrap().is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct RangeToken {
    pub start: Arc<RwLock<ExpressionToken>>,
    pub end: Arc<RwLock<ExpressionToken>>,

    pub location: TokenLocation,
}

impl PartialEq<ValueToken> for RangeToken {
    fn eq(&self, _other: &ValueToken) -> bool {
        false
    }
}

impl PartialEq<RangeToken> for RangeToken {
    fn eq(&self, _other: &RangeToken) -> bool {
        false
    }
}

impl BaseToken for RangeToken {
    fn inspect(&self) -> String {
        format!(
            "Range({}..{}) {{ <range> }}",
            if let ExpressionToken::Value(value_token) = &*self.start.read().unwrap() {
                value_token.inspect()
            } else {
                "<start expression>".to_string()
            },
            if let ExpressionToken::Value(value_token) = &*self.end.read().unwrap() {
                value_token.inspect()
            } else {
                "<end expression>".to_string()
            }
        )
    }

    fn value(&self, spaces: usize) -> String {
        format!(
            "{}..{}",
            if let ExpressionToken::Value(value_token) = &*self.start.read().unwrap() {
                value_token.value(spaces)
            } else {
                "<start expression>".to_string()
            },
            if let ExpressionToken::Value(value_token) = &*self.end.read().unwrap() {
                value_token.value(spaces)
            } else {
                "<end expression>".to_string()
            }
        )
    }

    fn truthy(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub struct BufferToken {
    pub value: Arc<RwLock<Vec<u8>>>,

    pub location: TokenLocation,
}

impl PartialEq<ValueToken> for BufferToken {
    fn eq(&self, _other: &ValueToken) -> bool {
        if let ValueToken::Buffer(other) = _other {
            for (left, right) in self
                .value
                .read()
                .unwrap()
                .iter()
                .zip(other.value.read().unwrap().iter())
            {
                if left != right {
                    return false;
                }
            }

            true
        } else {
            false
        }
    }
}

impl PartialEq<BufferToken> for BufferToken {
    fn eq(&self, other: &BufferToken) -> bool {
        for (left, right) in self
            .value
            .read()
            .unwrap()
            .iter()
            .zip(other.value.read().unwrap().iter())
        {
            if left != right {
                return false;
            }
        }

        true
    }
}

impl BaseToken for BufferToken {
    fn inspect(&self) -> String {
        let mut result = format!("Buffer({}) {{ ", self.value.read().unwrap().len());

        for byte in self.value.read().unwrap().iter().take(100) {
            result.push_str(&format!("{byte:02x} "));
        }

        if self.value.read().unwrap().len() > 100 {
            result
                .push_str(format!("... {} more ", self.value.read().unwrap().len() - 100).as_str());
        }

        result + "}"
    }

    fn value(&self, spaces: usize) -> String {
        let length = self.value.read().unwrap().len();

        " ".repeat(spaces)
            + &self
                .value
                .read()
                .unwrap()
                .iter()
                .fold(String::with_capacity(length * 3), |acc, byte| {
                    acc + &format!("{byte:02x} ")
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

impl PartialEq<ValueToken> for NullToken {
    fn eq(&self, other: &ValueToken) -> bool {
        matches!(other, ValueToken::Null(_))
    }
}

impl PartialEq<NullToken> for NullToken {
    fn eq(&self, _other: &NullToken) -> bool {
        true
    }
}

impl BaseToken for NullToken {
    fn inspect(&self) -> String {
        "Null".to_string()
    }

    fn value(&self, spaces: usize) -> String {
        " ".repeat(spaces) + "null"
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

impl PartialEq<ValueToken> for NativeMemoryToken {
    fn eq(&self, _other: &ValueToken) -> bool {
        false
    }
}

impl PartialEq<NativeMemoryToken> for NativeMemoryToken {
    fn eq(&self, _other: &NativeMemoryToken) -> bool {
        false
    }
}

impl BaseToken for NativeMemoryToken {
    fn inspect(&self) -> String {
        format!("NativeMemory({}) {{ <native> }}", self.name)
    }

    fn value(&self, _: usize) -> String {
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

impl PartialEq<ValueToken> for FunctionToken {
    fn eq(&self, _other: &ValueToken) -> bool {
        false
    }
}

impl PartialEq<FunctionToken> for FunctionToken {
    fn eq(&self, _other: &FunctionToken) -> bool {
        false
    }
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

    fn value(&self, _: usize) -> String {
        self.inspect()
    }

    fn truthy(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub struct ClassToken {
    pub name: String,
    pub args: Vec<String>,
    pub body: Arc<RwLock<Vec<Token>>>,

    pub location: TokenLocation,
}

impl PartialEq<ValueToken> for ClassToken {
    fn eq(&self, _other: &ValueToken) -> bool {
        false
    }
}

impl PartialEq<ClassToken> for ClassToken {
    fn eq(&self, _other: &ClassToken) -> bool {
        false
    }
}

impl BaseToken for ClassToken {
    fn inspect(&self) -> String {
        format!("Class({}) {{ }}", self.name)
    }

    fn value(&self, _: usize) -> String {
        self.inspect()
    }

    fn truthy(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub struct ClassInstanceToken {
    pub class: Arc<RwLock<ClassToken>>,
    pub scope: Arc<RwLock<HashMap<String, Arc<RwLock<ExpressionToken>>>>>,

    #[allow(dead_code)]
    pub location: TokenLocation,
}

impl PartialEq<ValueToken> for ClassInstanceToken {
    fn eq(&self, _other: &ValueToken) -> bool {
        false
    }
}

impl PartialEq<ClassInstanceToken> for ClassInstanceToken {
    fn eq(&self, _other: &ClassInstanceToken) -> bool {
        false
    }
}

impl BaseToken for ClassInstanceToken {
    fn inspect(&self) -> String {
        format!(
            "ClassInstance({}) {{ {} variables }}",
            self.class.read().unwrap().name,
            self.scope.read().unwrap().len()
        )
    }

    fn value(&self, _: usize) -> String {
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
    Range(RangeToken),
    Buffer(BufferToken),
    NativeMemory(NativeMemoryToken),
    Function(FunctionToken),
    Class(ClassToken),
    ClassInstance(ClassInstanceToken),
}

impl PartialEq<ValueToken> for ValueToken {
    fn eq(&self, other: &ValueToken) -> bool {
        match (self, other) {
            (ValueToken::String(left), ValueToken::String(right)) => left == right,
            (ValueToken::Number(left), ValueToken::Number(right)) => left == right,
            (ValueToken::Boolean(left), ValueToken::Boolean(right)) => left == right,
            (ValueToken::Null(left), ValueToken::Null(right)) => left == right,
            (ValueToken::Array(left), ValueToken::Array(right)) => left == right,
            (ValueToken::Range(left), ValueToken::Range(right)) => left == right,
            (ValueToken::Buffer(left), ValueToken::Buffer(right)) => left == right,
            (ValueToken::NativeMemory(left), ValueToken::NativeMemory(right)) => left == right,
            (ValueToken::Function(left), ValueToken::Function(right)) => left == right,
            (ValueToken::Class(left), ValueToken::Class(right)) => left == right,
            (ValueToken::ClassInstance(left), ValueToken::ClassInstance(right)) => left == right,
            _ => false,
        }
    }
}

impl BaseToken for ValueToken {
    fn inspect(&self) -> String {
        match self {
            ValueToken::String(string_token) => string_token.inspect(),
            ValueToken::Number(number_token) => number_token.inspect(),
            ValueToken::Boolean(boolean_token) => boolean_token.inspect(),
            ValueToken::Null(null_token) => null_token.inspect(),
            ValueToken::Array(array_token) => array_token.inspect(),
            ValueToken::Range(range_token) => range_token.inspect(),
            ValueToken::Buffer(buffer_token) => buffer_token.inspect(),
            ValueToken::NativeMemory(native_memory_token) => native_memory_token.inspect(),
            ValueToken::Function(function_token) => function_token.inspect(),
            ValueToken::Class(class_token) => class_token.inspect(),
            ValueToken::ClassInstance(class_instance_token) => class_instance_token.inspect(),
        }
    }

    fn value(&self, spaces: usize) -> String {
        match self {
            ValueToken::String(string_token) => string_token.value(spaces),
            ValueToken::Number(number_token) => number_token.value(spaces),
            ValueToken::Boolean(boolean_token) => boolean_token.value(spaces),
            ValueToken::Null(null_token) => null_token.value(spaces),
            ValueToken::Array(array_token) => array_token.value(spaces),
            ValueToken::Range(range_token) => range_token.value(spaces),
            ValueToken::Buffer(buffer_token) => buffer_token.value(spaces),
            ValueToken::NativeMemory(native_memory_token) => native_memory_token.value(spaces),
            ValueToken::Function(function_token) => function_token.value(spaces),
            ValueToken::Class(class_token) => class_token.value(spaces),
            ValueToken::ClassInstance(class_instance_token) => class_instance_token.value(spaces),
        }
    }

    fn truthy(&self) -> bool {
        match self {
            ValueToken::String(string_token) => string_token.truthy(),
            ValueToken::Number(number_token) => number_token.truthy(),
            ValueToken::Boolean(boolean_token) => boolean_token.truthy(),
            ValueToken::Null(null_token) => null_token.truthy(),
            ValueToken::Array(array_token) => array_token.truthy(),
            ValueToken::Range(range_token) => range_token.truthy(),
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
            ValueToken::Range(token) => token.location.clone(),
            ValueToken::Buffer(token) => token.location.clone(),
            ValueToken::NativeMemory(_) => TokenLocation::default(),
            ValueToken::Function(token) => token.location.clone(),
            ValueToken::Class(token) => token.location.clone(),
            ValueToken::ClassInstance(_) => TokenLocation::default(),
        }
    }
}
