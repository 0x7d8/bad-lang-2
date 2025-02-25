use std::{cell::RefCell, rc::Rc};

use super::logic::ExpressionToken;

pub trait BaseToken {
    fn inspect(&self) -> String;
    fn value(&self) -> String;
    fn truthy(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct StringToken {
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
    pub value: Rc<RefCell<Vec<ExpressionToken>>>,
}

impl BaseToken for ArrayToken {
    fn inspect(&self) -> String {
        let mut result = format!("Array({}) {{\n", self.value.borrow().len());

        for token in self.value.borrow().iter() {
            if let ExpressionToken::Value(value_token) = token {
                result.push_str(&format!("{}\n", value_token.inspect()));
            }
        }

        result + "}"
    }

    fn value(&self) -> String {
        let mut result = "[\n".to_string();

        for token in self.value.borrow().iter() {
            if let ExpressionToken::Value(value_token) = token {
                result.push_str(&format!("{}\n", value_token.value()));
            }
        }

        result + "]"
    }

    fn truthy(&self) -> bool {
        !self.value.borrow().is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct NullToken;

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
pub enum ValueToken {
    String(StringToken),
    Number(NumberToken),
    Boolean(BooleanToken),
    Null(NullToken),
    Array(ArrayToken),
}

impl BaseToken for ValueToken {
    fn inspect(&self) -> String {
        match self {
            ValueToken::String(string_token) => string_token.inspect(),
            ValueToken::Number(number_token) => number_token.inspect(),
            ValueToken::Boolean(boolean_token) => boolean_token.inspect(),
            ValueToken::Null(null_token) => null_token.inspect(),
            ValueToken::Array(array_token) => array_token.inspect(),
        }
    }

    fn value(&self) -> String {
        match self {
            ValueToken::String(string_token) => string_token.value(),
            ValueToken::Number(number_token) => number_token.value(),
            ValueToken::Boolean(boolean_token) => boolean_token.value(),
            ValueToken::Null(null_token) => null_token.value(),
            ValueToken::Array(array_token) => array_token.value(),
        }
    }

    fn truthy(&self) -> bool {
        match self {
            ValueToken::String(string_token) => string_token.truthy(),
            ValueToken::Number(number_token) => number_token.truthy(),
            ValueToken::Boolean(boolean_token) => boolean_token.truthy(),
            ValueToken::Null(null_token) => null_token.truthy(),
            ValueToken::Array(array_token) => array_token.truthy(),
        }
    }
}
