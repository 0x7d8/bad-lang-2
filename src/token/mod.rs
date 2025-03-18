pub mod base;
pub mod logic;
pub mod macros;
pub mod runtime;

use base::{
    ArrayToken, BooleanToken, FunctionToken, NullToken, NumberToken, StringToken, ValueToken,
};
use logic::{
    BreakToken, ExpressionToken, FnCallToken, IfToken, LetAssignNumToken, LetAssignToken, LetToken,
    LoopToken, ReturnToken,
};
use std::{
    collections::HashMap,
    fmt::Display,
    str::FromStr,
    sync::{Arc, Mutex, RwLock},
};

#[derive(Debug, Clone)]
pub struct TokenLocation {
    pub file: String,
    pub line: usize,
}

impl Default for TokenLocation {
    fn default() -> Self {
        Self {
            file: "<internal>".to_string(),
            line: 1,
        }
    }
}

impl Display for TokenLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.file, self.line)
    }
}

#[derive(Debug, Clone)]
pub enum Token {
    Let(LetToken),
    LetAssign(LetAssignToken),
    LetAssignNum(LetAssignNumToken),
    FnCall(FnCallToken),
    Loop(LoopToken),
    Break(BreakToken),
    Return(ReturnToken),
    If(IfToken),
}

pub enum InsideToken {
    Function(FunctionToken),
    Loop(LoopToken),
    If(IfToken),
}

type MacroFn = fn(Vec<ExpressionToken>) -> Option<ExpressionToken>;
pub struct Tokenizer {
    location: TokenLocation,

    input: String,
    default_macros: HashMap<String, MacroFn>,

    pub tokens: Vec<Token>,
    inside: Vec<Arc<Mutex<InsideToken>>>,
}

impl Tokenizer {
    pub fn new(input: &str, file: &str) -> Self {
        Self {
            location: TokenLocation {
                file: file.to_string(),
                line: 0,
            },

            input: input.to_string(),
            default_macros: HashMap::from([
                ("concat!".to_string(), macros::concat as MacroFn),
                ("inline!".to_string(), macros::inline as MacroFn),
                ("add!".to_string(), macros::number::add as MacroFn),
                ("sqrt!".to_string(), macros::number::sqrt as MacroFn),
            ]),
            tokens: Vec::new(),
            inside: Vec::new(),
        }
    }

    pub fn parse(&mut self) {
        self.tokens.clear();

        for line in self.input.clone().lines() {
            self.location.line += 1;

            let token = self.tokenize(line);
            if let Some(token) = token {
                self.push_token(token);
            }
        }
    }

    fn push_token(&mut self, token: Token) {
        if !self.inside.is_empty() {
            match &*self.inside.last().unwrap().lock().unwrap() {
                InsideToken::Function(fn_token) => {
                    fn_token.body.write().unwrap().push(token);
                }
                InsideToken::Loop(loop_token) => {
                    loop_token.body.write().unwrap().push(token);
                }
                InsideToken::If(if_token) => {
                    if_token.body.write().unwrap().push(token);
                }
            }
        } else {
            self.tokens.push(token);
        }
    }

    fn current_tokens_context(&self) -> Vec<Token> {
        let mut tokens = Vec::new();

        for token in &self.tokens {
            tokens.push(token.clone());
            Self::add_nested_tokens(Self::check_if_is_inside(token), &mut tokens);
        }

        for inside in &self.inside {
            match &*inside.lock().unwrap() {
                InsideToken::Function(fn_token) => {
                    for token in fn_token.body.read().unwrap().iter() {
                        tokens.push(token.clone());
                        Self::add_nested_tokens(Self::check_if_is_inside(token), &mut tokens);
                    }
                }
                InsideToken::Loop(loop_token) => {
                    for token in loop_token.body.read().unwrap().iter() {
                        tokens.push(token.clone());
                        Self::add_nested_tokens(Self::check_if_is_inside(token), &mut tokens);
                    }
                }
                InsideToken::If(if_token) => {
                    for token in if_token.body.read().unwrap().iter() {
                        tokens.push(token.clone());
                        Self::add_nested_tokens(Self::check_if_is_inside(token), &mut tokens);
                    }
                }
            }
        }

        tokens
    }

    fn check_if_is_inside(token: &Token) -> Option<InsideToken> {
        match token {
            Token::Loop(loop_token) => {
                return Some(InsideToken::Loop(loop_token.clone()));
            }
            Token::If(if_token) => {
                return Some(InsideToken::If(if_token.clone()));
            }
            Token::Let(let_token) => {
                if let_token.is_function {
                    if let ExpressionToken::Value(ValueToken::Function(fn_token)) =
                        &*let_token.value.read().unwrap()
                    {
                        return Some(InsideToken::Function(fn_token.clone()));
                    }
                }
            }
            _ => {}
        }

        None
    }

    fn add_nested_tokens(token: Option<InsideToken>, tokens: &mut Vec<Token>) {
        if token.is_none() {
            return;
        }

        match token.unwrap() {
            InsideToken::Function(fn_token) => {
                for token in fn_token.body.read().unwrap().iter() {
                    tokens.push(token.clone());
                    Self::add_nested_tokens(Self::check_if_is_inside(token), tokens);
                }
            }
            InsideToken::Loop(loop_token) => {
                for token in loop_token.body.read().unwrap().iter() {
                    tokens.push(token.clone());
                    Self::add_nested_tokens(Self::check_if_is_inside(token), tokens);
                }
            }
            InsideToken::If(if_token) => {
                for token in if_token.body.read().unwrap().iter() {
                    tokens.push(token.clone());
                    Self::add_nested_tokens(Self::check_if_is_inside(token), tokens);
                }
            }
        }
    }

    pub fn tokenize(&mut self, mut segment: &str) -> Option<Token> {
        segment = segment.trim();

        if segment.is_empty() || segment.starts_with("//") || segment.starts_with("#") {
            return None;
        }

        if segment == "}" {
            if !self.inside.is_empty() {
                self.inside.pop().unwrap();
                return None;
            } else {
                panic!("unexpected '}}' in {}", self.location);
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
                panic!("unexpected value in {} (did you typo?)", self.location);
            }

            return Some(Token::Let(LetToken {
                name: name.to_string(),
                is_const: parts[1] == "const",
                is_function: false,
                value: Arc::new(RwLock::new(value.unwrap())),
                location: self.location(),
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
                    is_function: false,
                    value: Arc::new(RwLock::new(ExpressionToken::Value(ValueToken::Null(
                        NullToken {
                            location: self.location(),
                        },
                    )))),
                    location: self.location(),
                }));
            }

            let body = Arc::new(RwLock::new(body));

            let value = ValueToken::Function(FunctionToken {
                name: name.clone(),
                args: args.clone(),
                body: Arc::clone(&body),

                location: self.location(),
            });

            let token = Token::Let(LetToken {
                name: name.clone(),
                is_const: true,
                is_function: true,
                value: Arc::new(RwLock::new(ExpressionToken::Value(value))),
                location: self.location(),
            });

            self.push_token(token);
            self.inside
                .push(Arc::new(Mutex::new(InsideToken::Function(FunctionToken {
                    name,
                    args,
                    body,

                    location: self.location(),
                }))));

            return None;
        } else if segment.starts_with("loop") {
            let body = Arc::new(RwLock::new(Vec::new()));
            let token = Token::Loop(LoopToken {
                body: Arc::clone(&body),
            });

            self.push_token(token);
            self.inside
                .push(Arc::new(Mutex::new(InsideToken::Loop(LoopToken { body }))));

            return None;
        } else if segment.starts_with("return") && !self.inside.is_empty() {
            if segment.len() < 7 {
                return Some(Token::Return(ReturnToken {
                    value: Arc::new(ExpressionToken::Value(ValueToken::Null(NullToken {
                        location: self.location(),
                    }))),
                }));
            }

            let value = self.parse_expression(segment[6..].trim());
            if value.is_none() {
                panic!("unexpected value in {} (did you typo?)", self.location);
            }

            return Some(Token::Return(ReturnToken {
                value: Arc::new(value.unwrap()),
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

            let condition = Arc::new(condition.unwrap_or_else(|| {
                panic!("unexpected condition in {} (did you typo?)", self.location)
            }));

            let body = Arc::new(RwLock::new(Vec::new()));
            let token = Token::If(IfToken {
                reversed,
                condition: Arc::clone(&condition),
                body: Arc::clone(&body),
                location: self.location(),
            });

            self.push_token(token);
            self.inside
                .push(Arc::new(Mutex::new(InsideToken::If(IfToken {
                    reversed,
                    condition,
                    body,
                    location: self.location(),
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
                    args: tokens.into_iter().map(Arc::new).collect(),
                    location: self.location(),
                }));
            }
        }

        for token in self.current_tokens_context().iter().rev() {
            if let Token::Let(let_token) = token {
                if segment.starts_with(&format!("{}(", let_token.name)) {
                    let tokens =
                        self.parse_args(&segment[let_token.name.len() + 1..segment.len() - 1]);

                    return Some(Token::FnCall(FnCallToken {
                        name: let_token.name.clone(),
                        args: tokens.into_iter().map(Arc::new).collect(),
                        location: self.location(),
                    }));
                }

                if let_token.is_const {
                    continue;
                }

                if segment.starts_with(&format!("{} = ", let_token.name)) {
                    let value = self.parse_expression(segment[let_token.name.len() + 3..].trim());
                    if value.is_none() {
                        panic!("unexpected value in {} (did you typo?)", self.location);
                    }

                    return Some(Token::LetAssign(LetAssignToken {
                        name: let_token.name.clone(),
                        value: Arc::new(value.unwrap()),
                    }));
                }

                if segment.starts_with(&format!("{} += ", let_token.name)) {
                    let value = self.parse_expression(segment[let_token.name.len() + 4..].trim());
                    if value.is_none() {
                        panic!("unexpected value in {} (did you typo?)", self.location);
                    }

                    return Some(Token::LetAssignNum(LetAssignNumToken {
                        name: let_token.name.clone(),
                        operation: logic::NumOperation::Add,
                        value: Arc::new(value.unwrap()),
                    }));
                } else if segment == format!("{}++", let_token.name) {
                    return Some(Token::LetAssignNum(LetAssignNumToken {
                        name: let_token.name.clone(),
                        operation: logic::NumOperation::Add,
                        value: Arc::new(ExpressionToken::Value(ValueToken::Number(NumberToken {
                            value: 1.0,
                            location: self.location(),
                        }))),
                    }));
                } else if segment.starts_with(&format!("{} -= ", let_token.name)) {
                    let value = self.parse_expression(segment[let_token.name.len() + 4..].trim());
                    if value.is_none() {
                        panic!("unexpected value in {} (did you typo?)", self.location);
                    }

                    return Some(Token::LetAssignNum(LetAssignNumToken {
                        name: let_token.name.clone(),
                        operation: logic::NumOperation::Sub,
                        value: Arc::new(value.unwrap()),
                    }));
                } else if segment == format!("{}--", let_token.name) {
                    return Some(Token::LetAssignNum(LetAssignNumToken {
                        name: let_token.name.clone(),
                        operation: logic::NumOperation::Sub,
                        value: Arc::new(ExpressionToken::Value(ValueToken::Number(NumberToken {
                            value: 1.0,
                            location: self.location(),
                        }))),
                    }));
                } else if segment.starts_with(&format!("{} *= ", let_token.name)) {
                    let value = self.parse_expression(segment[let_token.name.len() + 4..].trim());
                    if value.is_none() {
                        panic!("unexpected value in {} (did you typo?)", self.location);
                    }

                    return Some(Token::LetAssignNum(LetAssignNumToken {
                        name: let_token.name.clone(),
                        operation: logic::NumOperation::Mul,
                        value: Arc::new(value.unwrap()),
                    }));
                } else if segment.starts_with(&format!("{} /= ", let_token.name)) {
                    let value = self.parse_expression(segment[let_token.name.len() + 4..].trim());
                    if value.is_none() {
                        panic!("unexpected value in {} (did you typo?)", self.location);
                    }

                    return Some(Token::LetAssignNum(LetAssignNumToken {
                        name: let_token.name.clone(),
                        operation: logic::NumOperation::Div,
                        value: Arc::new(value.unwrap()),
                    }));
                }
            }
        }

        panic!("unexpected token in {} (did you typo?)", self.location);
    }

    pub fn parse_expression(&self, segment: &str) -> Option<ExpressionToken> {
        if segment.starts_with("\"") && segment.ends_with("\"") {
            return Some(ExpressionToken::Value(ValueToken::String(StringToken {
                value: segment[1..segment.len() - 1]
                    .to_string()
                    .replace("\\n", "\n")
                    .replace("\\r", "\r")
                    .replace("\\t", "\t")
                    .replace("\\\\", "\\"),
                location: self.location(),
            })));
        } else if segment.starts_with("[") && segment.ends_with("]") {
            let tokens = self.parse_args(&segment[1..segment.len() - 1]);

            return Some(ExpressionToken::Value(ValueToken::Array(ArrayToken {
                location: self.location(),
                value: Arc::new(RwLock::new(tokens)),
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
                    args: tokens.into_iter().map(Arc::new).collect(),
                    location: self.location(),
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
                if segment.starts_with(&format!("{}(", let_token.name)) && let_token.is_function {
                    let tokens =
                        self.parse_args(&segment[let_token.name.len() + 1..segment.len() - 1]);

                    return Some(ExpressionToken::FnCall(FnCallToken {
                        name: let_token.name.clone(),
                        args: tokens.into_iter().map(Arc::new).collect(),
                        location: self.location(),
                    }));
                }

                if segment == let_token.name {
                    return Some(ExpressionToken::Let(LetToken {
                        name: let_token.name.clone(),
                        is_const: let_token.is_const,
                        is_function: matches!(
                            &*let_token.value.read().unwrap(),
                            ExpressionToken::Value(ValueToken::Function(_))
                        ),
                        value: Arc::clone(&let_token.value),
                        location: self.location(),
                    }));
                }
            }
        }

        {
            let mut context = meval::Context::empty();

            for token in self.current_tokens_context().iter().rev() {
                if let Token::Let(let_token) = token {
                    context.var(&let_token.name, 0.0);
                }
            }

            if let Ok(expression) = meval::Expr::from_str(segment) {
                return Some(ExpressionToken::Math(expression));
            }
        }

        panic!("unexpected expression in {} (did you typo?)", self.location);
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
        self.location.clone()
    }
}
