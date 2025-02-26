use super::{
    ExpressionToken, LetToken,
    base::{ArrayToken, BooleanToken, NullToken, StringToken},
};
use crate::token::base::{NumberToken, ValueToken};

pub mod number;

pub fn extract_number(token: &ExpressionToken) -> Option<f64> {
    match token {
        ExpressionToken::Value(ValueToken::Number(NumberToken { value, .. })) => Some(*value),
        ExpressionToken::Let(LetToken {
            value, is_const, ..
        }) => {
            if !*is_const {
                return None;
            }

            if let ExpressionToken::Value(ValueToken::Number(NumberToken { value, .. })) =
                &*value.lock().unwrap()
            {
                Some(*value)
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn extract_string(token: &ExpressionToken) -> Option<String> {
    match token {
        ExpressionToken::Value(ValueToken::String(StringToken { value, .. })) => {
            Some(value.clone())
        }
        ExpressionToken::Let(LetToken {
            value, is_const, ..
        }) => {
            if !*is_const {
                return None;
            }

            if let ExpressionToken::Value(ValueToken::String(StringToken { value, .. })) =
                &*value.lock().unwrap()
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
        location: Default::default(),
        value: result,
    })))
}

pub fn inline(args: Vec<ExpressionToken>) -> Option<ExpressionToken> {
    if args.len() != 1 {
        return None;
    }

    match &args[0] {
        ExpressionToken::Value(ValueToken::String(StringToken { value, location })) => {
            Some(ExpressionToken::Value(ValueToken::String(StringToken {
                location: *location,
                value: value.clone(),
            })))
        }
        ExpressionToken::Value(ValueToken::Number(NumberToken { value, location })) => {
            Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                location: *location,
                value: *value,
            })))
        }
        ExpressionToken::Value(ValueToken::Boolean(BooleanToken { value, location })) => {
            Some(ExpressionToken::Value(ValueToken::Boolean(BooleanToken {
                location: *location,
                value: *value,
            })))
        }
        ExpressionToken::Value(ValueToken::Array(ArrayToken { value, location })) => {
            Some(ExpressionToken::Value(ValueToken::Array(ArrayToken {
                location: *location,
                value: value.clone(),
            })))
        }
        ExpressionToken::Value(ValueToken::Null(NullToken { location })) => {
            Some(ExpressionToken::Value(ValueToken::Null(NullToken {
                location: *location,
            })))
        }
        ExpressionToken::Let(LetToken {
            value, is_const, ..
        }) => {
            if !*is_const {
                return None;
            }

            match &*value.as_ref().lock().unwrap() {
                ExpressionToken::Value(ValueToken::String(StringToken { value, location })) => {
                    Some(ExpressionToken::Value(ValueToken::String(StringToken {
                        location: *location,
                        value: value.clone(),
                    })))
                }
                ExpressionToken::Value(ValueToken::Number(NumberToken { value, location })) => {
                    Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                        location: *location,
                        value: *value,
                    })))
                }
                ExpressionToken::Value(ValueToken::Boolean(BooleanToken { value, location })) => {
                    Some(ExpressionToken::Value(ValueToken::Boolean(BooleanToken {
                        location: *location,
                        value: *value,
                    })))
                }
                ExpressionToken::Value(ValueToken::Array(ArrayToken { value, location })) => {
                    Some(ExpressionToken::Value(ValueToken::Array(ArrayToken {
                        location: *location,
                        value: value.clone(),
                    })))
                }
                ExpressionToken::Value(ValueToken::Null(NullToken { location })) => {
                    Some(ExpressionToken::Value(ValueToken::Null(NullToken {
                        location: *location,
                    })))
                }
                _ => None,
            }
        }
        _ => None,
    }
}
