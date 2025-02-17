use super::{
    base::{ArrayToken, BooleanToken, NullToken, StringToken},
    ExpressionToken, LetToken,
};
use crate::token::base::{NumberToken, ValueToken};

pub mod number;

pub fn extract_number(token: &ExpressionToken) -> Option<f64> {
    match token {
        ExpressionToken::Value(ValueToken::Number(NumberToken { value })) => Some(*value),
        ExpressionToken::Let(LetToken {
            value, is_const, ..
        }) => {
            if !*is_const {
                return None;
            }

            if let ExpressionToken::Value(ValueToken::Number(NumberToken { value })) =
                &*value.borrow()
            {
                Some(value.clone())
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn extract_string(token: &ExpressionToken) -> Option<String> {
    match token {
        ExpressionToken::Value(ValueToken::String(StringToken { value })) => Some(value.clone()),
        ExpressionToken::Let(LetToken {
            value, is_const, ..
        }) => {
            if !*is_const {
                return None;
            }

            if let ExpressionToken::Value(ValueToken::String(StringToken { value })) =
                &*value.borrow()
            {
                Some(value.clone())
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn concat(args: Vec<ExpressionToken>) -> Option<ExpressionToken> {
    let mut result = String::new();
    for arg in args {
        if let Some(value) = extract_string(&arg) {
            result.push_str(&value);
        } else {
            return None;
        }
    }

    Some(ExpressionToken::Value(ValueToken::String(StringToken {
        value: result,
    })))
}

pub fn inline(args: Vec<ExpressionToken>) -> Option<ExpressionToken> {
    if args.len() != 1 {
        return None;
    }

    match &args[0] {
        ExpressionToken::Value(ValueToken::String(StringToken { value })) => {
            Some(ExpressionToken::Value(ValueToken::String(StringToken {
                value: value.clone(),
            })))
        }
        ExpressionToken::Value(ValueToken::Number(NumberToken { value })) => {
            Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                value: *value,
            })))
        }
        ExpressionToken::Value(ValueToken::Boolean(BooleanToken { value })) => {
            Some(ExpressionToken::Value(ValueToken::Boolean(BooleanToken {
                value: *value,
            })))
        }
        ExpressionToken::Value(ValueToken::Array(ArrayToken { value })) => {
            Some(ExpressionToken::Value(ValueToken::Array(ArrayToken {
                value: value.clone(),
            })))
        }
        ExpressionToken::Value(ValueToken::Null(_)) => {
            Some(ExpressionToken::Value(ValueToken::Null(NullToken)))
        }
        ExpressionToken::Let(LetToken {
            value, is_const, ..
        }) => {
            if !*is_const {
                return None;
            }

            match &*value.as_ref().borrow() {
                ExpressionToken::Value(ValueToken::String(StringToken { value })) => {
                    Some(ExpressionToken::Value(ValueToken::String(StringToken {
                        value: value.clone(),
                    })))
                }
                ExpressionToken::Value(ValueToken::Number(NumberToken { value })) => {
                    Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                        value: *value,
                    })))
                }
                ExpressionToken::Value(ValueToken::Boolean(BooleanToken { value })) => {
                    Some(ExpressionToken::Value(ValueToken::Boolean(BooleanToken {
                        value: *value,
                    })))
                }
                ExpressionToken::Value(ValueToken::Array(ArrayToken { value })) => {
                    Some(ExpressionToken::Value(ValueToken::Array(ArrayToken {
                        value: value.clone(),
                    })))
                }
                ExpressionToken::Value(ValueToken::Null(_)) => {
                    Some(ExpressionToken::Value(ValueToken::Null(NullToken)))
                }
                _ => None,
            }
        }
        _ => None,
    }
}
