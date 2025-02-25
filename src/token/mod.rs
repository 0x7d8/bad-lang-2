pub mod base;
pub mod logic;
pub mod macros;
pub mod runtime;

use base::{ArrayToken, BooleanToken, NullToken, NumberToken, StringToken, ValueToken};
use logic::{
    BreakToken, ExpressionToken, FnCallToken, FnToken, IfToken, LetAssignNumToken, LetAssignToken,
    LetToken, LoopToken, ReturnToken,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug, Clone, Copy)]
pub struct TokenLocation {
    pub line: usize,
}

impl Default for TokenLocation {
    fn default() -> Self {
        Self { line: usize::MAX }
    }
}

#[derive(Debug, Clone)]
pub enum Token {
    Let(LetToken),
    LetAssign(LetAssignToken),
    LetAssignNum(LetAssignNumToken),
    Fn(FnToken),
    FnCall(FnCallToken),
    Loop(LoopToken),
    Break(BreakToken),
    Return(ReturnToken),
    If(IfToken),
}

pub static mut LINE: usize = 0;

pub struct Tokenizer {
    input: String,
    default_macros: HashMap<String, fn(Vec<ExpressionToken>) -> Option<ExpressionToken>>,

    pub tokens: Vec<Token>,
    inside: Vec<Rc<RefCell<Token>>>,
}

impl Tokenizer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
            default_macros: HashMap::from([
                (
                    "concat!".to_string(),
                    macros::concat as fn(Vec<ExpressionToken>) -> Option<ExpressionToken>,
                ),
                (
                    "inline!".to_string(),
                    macros::inline as fn(Vec<ExpressionToken>) -> Option<ExpressionToken>,
                ),
                (
                    "add!".to_string(),
                    macros::number::add as fn(Vec<ExpressionToken>) -> Option<ExpressionToken>,
                ),
                (
                    "sqrt!".to_string(),
                    macros::number::sqrt as fn(Vec<ExpressionToken>) -> Option<ExpressionToken>,
                ),
            ]),
            tokens: Vec::new(),
            inside: Vec::new(),
        }
    }

    pub fn parse(&mut self) {
        self.tokens.clear();

        for line in self.input.clone().lines() {
            unsafe {
                LINE += 1;
            }

            let token = self.tokenize(line);
            if let Some(token) = token {
                self.push_token(token);
            }
        }
    }

    fn push_token(&mut self, token: Token) {
        if !self.inside.is_empty() {
            match &*self.inside.last().unwrap().borrow() {
                Token::Fn(fn_token) => {
                    fn_token.body.borrow_mut().push(token);
                }
                Token::Loop(loop_token) => {
                    loop_token.body.borrow_mut().push(token);
                }
                Token::If(if_token) => {
                    if_token.body.borrow_mut().push(token);
                }
                _ => unreachable!(),
            }
        } else {
            self.tokens.push(token);
        }
    }

    fn current_tokens_context(&self) -> Vec<Token> {
        let mut tokens = Vec::new();

        for token in &self.tokens {
            tokens.push(token.clone());
            self.add_nested_tokens(token, &mut tokens);
        }

        for inside in &self.inside {
            match &*inside.borrow() {
                Token::Fn(fn_token) => {
                    for token in fn_token.body.borrow().iter() {
                        tokens.push(token.clone());
                        self.add_nested_tokens(token, &mut tokens);
                    }
                }
                Token::Loop(loop_token) => {
                    for token in loop_token.body.borrow().iter() {
                        tokens.push(token.clone());
                        self.add_nested_tokens(token, &mut tokens);
                    }
                }
                Token::If(if_token) => {
                    for token in if_token.body.borrow().iter() {
                        tokens.push(token.clone());
                        self.add_nested_tokens(token, &mut tokens);
                    }
                }
                _ => unreachable!(),
            }
        }

        tokens
    }

    fn add_nested_tokens(&self, token: &Token, tokens: &mut Vec<Token>) {
        match token {
            Token::Fn(fn_token) => {
                for token in fn_token.body.borrow().iter() {
                    tokens.push(token.clone());
                    self.add_nested_tokens(token, tokens);
                }
            }
            Token::Loop(loop_token) => {
                for token in loop_token.body.borrow().iter() {
                    tokens.push(token.clone());
                    self.add_nested_tokens(token, tokens);
                }
            }
            Token::If(if_token) => {
                for token in if_token.body.borrow().iter() {
                    tokens.push(token.clone());
                    self.add_nested_tokens(token, tokens);
                }
            }
            _ => {}
        }
    }

    pub fn tokenize(&mut self, mut segment: &str) -> Option<Token> {
        segment = segment.trim();

        if segment == "}" {
            if !self.inside.is_empty() {
                self.inside.pop().unwrap();
                return None;
            } else {
                panic!("unexpected '}}' at line {}", unsafe { LINE });
            }
        }

        if segment.starts_with("let") {
            let parts: Vec<&str> = segment.split_whitespace().collect();
            if parts.len() < 3 {
                return None;
            }

            let name;
            let value;
            if parts[1] == "const" {
                name = parts[2];
                value = self.parse_expression(parts[4..].join(" ").as_str());
            } else {
                name = parts[1];
                value = self.parse_expression(parts[3..].join(" ").as_str());
            }

            if value.is_none() {
                panic!("unexpected value at line {} (did you typo?)", unsafe {
                    LINE
                });
            }

            return Some(Token::Let(LetToken {
                name: name.to_string(),
                is_const: parts[1] == "const",
                value: Rc::new(RefCell::new(value.unwrap())),
            }));
        } else if segment.starts_with("fn") {
            let parts: Vec<&str> = segment.split("(").collect();
            if parts.len() != 2 {
                return None;
            }

            let name = parts[0][3..].trim().to_string();
            let mut args: Vec<String> = parts[1][0..parts[1].len() - 3]
                .split(",")
                .map(|arg| arg.trim().to_string())
                .collect();

            if args.len() == 1 && args[0].is_empty() {
                args.clear();
            }

            let mut body = Vec::new();

            for arg in &args {
                body.push(Token::Let(LetToken {
                    name: arg.clone(),
                    is_const: false,
                    value: Rc::new(RefCell::new(ExpressionToken::Value(ValueToken::Null(
                        NullToken {
                            location: self.location(),
                        },
                    )))),
                }));
            }

            let body = Rc::new(RefCell::new(body));
            let token = Token::Fn(FnToken {
                name: name.clone(),
                args: args.clone(),
                body: Rc::clone(&body),
            });

            self.push_token(token);
            self.inside.push(Rc::new(RefCell::new(Token::Fn(FnToken {
                name,
                args,
                body,
            }))));

            return None;
        } else if segment.starts_with("loop") {
            let body = Rc::new(RefCell::new(Vec::new()));
            let token = Token::Loop(LoopToken {
                body: Rc::clone(&body),
            });

            self.push_token(token);
            self.inside
                .push(Rc::new(RefCell::new(Token::Loop(LoopToken { body }))));

            return None;
        } else if segment.starts_with("return") && !self.inside.is_empty() {
            if segment.len() < 7 {
                return Some(Token::Return(ReturnToken {
                    value: Rc::new(ExpressionToken::Value(ValueToken::Null(NullToken {
                        location: self.location(),
                    }))),
                }));
            }

            let value = self.parse_expression(segment[6..].trim());
            if value.is_none() {
                panic!("unexpected value at line {} (did you typo?)", unsafe {
                    LINE
                });
            }

            return Some(Token::Return(ReturnToken {
                value: Rc::new(value.unwrap()),
            }));
        } else if segment.starts_with("if") {
            let reversed;
            let condition;
            if segment.starts_with("if not") {
                reversed = true;
                condition = self.parse_expression(segment[8..segment.len() - 3].trim());
            } else {
                reversed = false;
                condition = self.parse_expression(segment[4..segment.len() - 3].trim());
            }

            let condition = Rc::new(condition.unwrap_or_else(|| {
                panic!("unexpected condition at line {} (did you typo?)", unsafe {
                    LINE
                })
            }));

            let body = Rc::new(RefCell::new(Vec::new()));
            let token = Token::If(IfToken {
                reversed,
                condition: Rc::clone(&condition),
                body: Rc::clone(&body),
            });

            self.push_token(token);
            self.inside.push(Rc::new(RefCell::new(Token::If(IfToken {
                reversed,
                condition,
                body,
            }))));

            return None;
        } else if segment == "break" && !self.inside.is_empty() {
            return Some(Token::Break(BreakToken));
        }

        for func in runtime::FUNCTIONS.iter() {
            if segment.starts_with(&format!("{}(", func)) {
                let tokens = self.parse_args(&segment[func.len() + 1..segment.len() - 1]);

                return Some(Token::FnCall(FnCallToken {
                    name: func.to_string(),
                    args: tokens.into_iter().map(Rc::new).collect(),
                }));
            }
        }

        for token in self.current_tokens_context().iter().rev() {
            if let Token::Fn(fn_token) = token {
                if segment.starts_with(&format!("{}(", fn_token.name)) {
                    let tokens =
                        self.parse_args(&segment[fn_token.name.len() + 1..segment.len() - 1]);

                    return Some(Token::FnCall(FnCallToken {
                        name: fn_token.name.clone(),
                        args: tokens.into_iter().map(Rc::new).collect(),
                    }));
                }
            } else if let Token::Let(let_token) = token {
                if let_token.is_const {
                    continue;
                }

                if segment.starts_with(&format!("{} = ", let_token.name)) {
                    let value = self.parse_expression(segment[let_token.name.len() + 3..].trim());
                    if value.is_none() {
                        panic!("unexpected value at line {} (did you typo?)", unsafe {
                            LINE
                        });
                    }

                    return Some(Token::LetAssign(LetAssignToken {
                        name: let_token.name.clone(),
                        value: Rc::new(value.unwrap()),
                    }));
                }

                if segment.starts_with(&format!("{} += ", let_token.name)) {
                    let value = self.parse_expression(segment[let_token.name.len() + 4..].trim());
                    if value.is_none() {
                        panic!("unexpected value at line {} (did you typo?)", unsafe {
                            LINE
                        });
                    }

                    return Some(Token::LetAssignNum(LetAssignNumToken {
                        name: let_token.name.clone(),
                        operation: logic::NumOperation::Add,
                        value: Rc::new(value.unwrap()),
                    }));
                } else if segment == format!("{}++", let_token.name) {
                    return Some(Token::LetAssignNum(LetAssignNumToken {
                        name: let_token.name.clone(),
                        operation: logic::NumOperation::Add,
                        value: Rc::new(ExpressionToken::Value(ValueToken::Number(NumberToken {
                            location: self.location(),
                            value: 1.0,
                        }))),
                    }));
                } else if segment.starts_with(&format!("{} -= ", let_token.name)) {
                    let value = self.parse_expression(segment[let_token.name.len() + 4..].trim());
                    if value.is_none() {
                        panic!("unexpected value at line {} (did you typo?)", unsafe {
                            LINE
                        });
                    }

                    return Some(Token::LetAssignNum(LetAssignNumToken {
                        name: let_token.name.clone(),
                        operation: logic::NumOperation::Sub,
                        value: Rc::new(value.unwrap()),
                    }));
                } else if segment == format!("{}--", let_token.name) {
                    return Some(Token::LetAssignNum(LetAssignNumToken {
                        name: let_token.name.clone(),
                        operation: logic::NumOperation::Sub,
                        value: Rc::new(ExpressionToken::Value(ValueToken::Number(NumberToken {
                            location: self.location(),
                            value: 1.0,
                        }))),
                    }));
                } else if segment.starts_with(&format!("{} *= ", let_token.name)) {
                    let value = self.parse_expression(segment[let_token.name.len() + 4..].trim());
                    if value.is_none() {
                        panic!("unexpected value at line {} (did you typo?)", unsafe {
                            LINE
                        });
                    }

                    return Some(Token::LetAssignNum(LetAssignNumToken {
                        name: let_token.name.clone(),
                        operation: logic::NumOperation::Mul,
                        value: Rc::new(value.unwrap()),
                    }));
                } else if segment.starts_with(&format!("{} /= ", let_token.name)) {
                    let value = self.parse_expression(segment[let_token.name.len() + 4..].trim());
                    if value.is_none() {
                        panic!("unexpected value at line {} (did you typo?)", unsafe {
                            LINE
                        });
                    }

                    return Some(Token::LetAssignNum(LetAssignNumToken {
                        name: let_token.name.clone(),
                        operation: logic::NumOperation::Div,
                        value: Rc::new(value.unwrap()),
                    }));
                }
            }
        }

        None
    }

    pub fn parse_expression(&self, segment: &str) -> Option<ExpressionToken> {
        if segment.starts_with("\"") && segment.ends_with("\"") {
            return Some(ExpressionToken::Value(ValueToken::String(StringToken {
                location: self.location(),
                value: segment[1..segment.len() - 1].to_string(),
            })));
        } else if segment.starts_with("[") && segment.ends_with("]") {
            let tokens = self.parse_args(&segment[1..segment.len() - 1]);

            return Some(ExpressionToken::Value(ValueToken::Array(ArrayToken {
                location: self.location(),
                value: Rc::new(RefCell::new(tokens)),
            })));
        }

        let number = segment.parse::<f64>();
        if let Ok(number) = number {
            return Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                location: self.location(),
                value: number,
            })));
        }

        if let Some(stripped) = segment.strip_prefix("0x") {
            let number = u64::from_str_radix(stripped, 16);
            if let Ok(number) = number {
                return Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                    location: self.location(),
                    value: number as f64,
                })));
            }
        }

        if segment == "true" || segment == "false" {
            return Some(ExpressionToken::Value(ValueToken::Boolean(BooleanToken {
                location: self.location(),
                value: segment == "true",
            })));
        }

        if segment == "null" {
            return Some(ExpressionToken::Value(ValueToken::Null(NullToken {
                location: self.location(),
            })));
        }

        for func in runtime::FUNCTIONS.iter() {
            if segment.starts_with(format!("{}(", func).as_str()) && segment.ends_with(")") {
                let tokens = self.parse_args(&segment[func.len() + 1..segment.len() - 1]);

                return Some(ExpressionToken::FnCall(FnCallToken {
                    name: func.to_string(),
                    args: tokens.into_iter().map(Rc::new).collect(),
                }));
            }
        }

        for (name, func) in &self.default_macros {
            if segment.starts_with(format!("{}(", name).as_str()) && segment.ends_with(")") {
                let tokens = self.parse_args(&segment[name.len() + 1..segment.len() - 1]);

                return func(tokens);
            }
        }

        for token in self.current_tokens_context().iter().rev() {
            if let Token::Let(let_token) = token {
                if segment == let_token.name {
                    return Some(ExpressionToken::Let(LetToken {
                        name: let_token.name.clone(),
                        is_const: let_token.is_const,
                        value: Rc::clone(&let_token.value),
                    }));
                }
            } else if let Token::Fn(fn_token) = token {
                if segment.starts_with(&format!("{}(", fn_token.name)) {
                    let tokens =
                        self.parse_args(&segment[fn_token.name.len() + 1..segment.len() - 1]);

                    return Some(ExpressionToken::FnCall(FnCallToken {
                        name: fn_token.name.clone(),
                        args: tokens.into_iter().map(Rc::new).collect(),
                    }));
                }
            }
        }

        None
    }

    pub fn parse_args(&self, segment: &str) -> Vec<ExpressionToken> {
        let mut tokens = Vec::new();
        let mut expr = String::new();
        let mut depth = 0;

        let mut in_string = false;
        let mut in_array = false;

        for c in segment.chars() {
            if c == '"' {
                in_string = !in_string;
            } else if c == '[' {
                in_array = true;
            } else if c == ']' {
                in_array = false;
            } else if !in_string && !in_array {
                if c == '(' {
                    depth += 1;
                } else if c == ')' {
                    depth -= 1;
                }
            }

            if c == ',' && depth == 0 && !in_string && !in_array {
                if let Some(token) = self.parse_expression(expr.trim()) {
                    tokens.push(token);
                }
                expr.clear();
            } else {
                expr.push(c);
            }
        }

        if !expr.is_empty() {
            if let Some(token) = self.parse_expression(expr.trim()) {
                tokens.push(token);
            }
        }

        tokens
    }

    fn location(&self) -> TokenLocation {
        TokenLocation {
            line: unsafe { LINE },
        }
    }
}
