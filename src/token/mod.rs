pub mod base;
pub mod comparison;
pub mod logic;
pub mod macros;
pub mod runtime;

use base::{
    ArrayToken, BooleanToken, ClassToken, FunctionToken, NullToken, NumberToken, RangeToken,
    StringToken, ValueToken,
};
use comparison::{COMPARISON_OPERATORS, ComparisonToken};
use logic::{
    BreakToken, ClassFnCallToken, ClassInstantiationToken, ExpressionToken, FnCallToken, IfToken,
    LetAssignNumToken, LetAssignToken, LetToken, LoopToken, ReturnToken, StaticClassFnCallToken,
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
    StaticClassFnCall(StaticClassFnCallToken),
    ClassFnCall(ClassFnCallToken),
    Loop(LoopToken),
    Break(BreakToken),
    Return(ReturnToken),
    If(IfToken),
}

pub enum InsideToken {
    Function(FunctionToken),
    Loop(LoopToken),
    If(IfToken),
    Class(ClassToken),
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

            if let Some(token) = self.tokenize(line) {
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
                InsideToken::Class(class_token) => {
                    class_token.body.write().unwrap().push(token);
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
                InsideToken::Class(class_token) => {
                    for token in class_token.body.read().unwrap().iter() {
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
                } else if let_token.is_class {
                    if let ExpressionToken::Value(ValueToken::Class(class_token)) =
                        &*let_token.value.read().unwrap()
                    {
                        return Some(InsideToken::Class(class_token.clone()));
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
            InsideToken::Class(class_token) => {
                for token in class_token.body.read().unwrap().iter() {
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
            if self.inside.pop().is_some() {
                return None;
            } else {
                panic!("unexpected '}}' in {}", self.location);
            }
        }

        if segment.starts_with("include") {
            let parts: Vec<&str> = segment.splitn(2, " ").collect();
            if parts.len() != 2 {
                panic!("invalid include in {}", self.location);
            }

            let file = self.parse_expression(parts[1]);
            if let Some(file) = file {
                if let ExpressionToken::Value(ValueToken::String(string_token)) = file {
                    let file = std::fs::read_to_string(&string_token.value);
                    if let Ok(file) = file {
                        let mut tokenizer = Tokenizer::new(&file, &string_token.value);
                        tokenizer.parse();

                        for token in tokenizer.tokens {
                            self.push_token(token);
                        }
                    } else {
                        panic!(
                            "unable to read file \"{}\" in {}",
                            string_token.value, self.location
                        );
                    }
                } else {
                    panic!("unexpected value in {} (did you typo?)", self.location);
                }
            } else {
                panic!("unexpected file in {} (did you typo?)", self.location);
            }

            return None;
        } else if segment.starts_with("import") {
            let parts: Vec<&str> = segment.splitn(4, " ").collect();
            if parts.len() != 4 || parts[2] != "as" {
                panic!("invalid import in {}", self.location);
            }

            let name = parts[3];
            let file = self.parse_expression(parts[1]);
            if let Some(file) = file {
                if let ExpressionToken::Value(ValueToken::String(string_token)) = file {
                    let class = ClassToken {
                        name: name.to_string(),
                        args: Vec::new(),
                        body: Arc::new(RwLock::new(Vec::new())),

                        location: self.location(),
                    };

                    let file = std::fs::read_to_string(&string_token.value);
                    if let Ok(file) = file {
                        let mut tokenizer = Tokenizer::new(&file, &string_token.value);
                        tokenizer.parse();

                        for token in tokenizer.tokens {
                            class.body.write().unwrap().push(token);
                        }
                    } else {
                        panic!(
                            "unable to read file \"{}\" in {}",
                            string_token.value, self.location
                        );
                    }

                    let token = Token::Let(LetToken {
                        name: name.to_string(),
                        is_const: true,
                        is_function: false,
                        is_class: true,
                        value: Arc::new(RwLock::new(ExpressionToken::Value(ValueToken::Class(
                            class,
                        )))),
                    });

                    return Some(token);
                } else {
                    panic!("unexpected value in {} (did you typo?)", self.location);
                }
            } else {
                panic!("unexpected file in {} (did you typo?)", self.location);
            }
        } else if segment.starts_with("let") {
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
                is_class: false,
                value: Arc::new(RwLock::new(value.unwrap())),
            }));
        } else if segment.starts_with("class") {
            let parts: Vec<&str> = segment.split("(").collect();
            if parts.len() != 2 {
                return None;
            }

            let name = parts[0][6..].trim();
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
                    is_class: false,
                    value: Arc::new(RwLock::new(ExpressionToken::Value(ValueToken::Null(
                        NullToken {
                            location: self.location(),
                        },
                    )))),
                }));
            }

            let body = Arc::new(RwLock::new(body));

            let token = Token::Let(LetToken {
                name: name.to_string(),
                is_const: true,
                is_function: false,
                is_class: true,
                value: Arc::new(RwLock::new(ExpressionToken::Value(ValueToken::Class(
                    ClassToken {
                        name: name.to_string(),
                        args: args.clone(),
                        body: Arc::clone(&body),

                        location: self.location(),
                    },
                )))),
            });

            self.push_token(token);
            self.inside
                .push(Arc::new(Mutex::new(InsideToken::Class(ClassToken {
                    name: name.to_string(),
                    args,
                    body,

                    location: self.location(),
                }))));

            return None;
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
                    is_class: false,
                    value: Arc::new(RwLock::new(ExpressionToken::Value(ValueToken::Null(
                        NullToken {
                            location: self.location(),
                        },
                    )))),
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
                is_class: false,
                value: Arc::new(RwLock::new(ExpressionToken::Value(value))),
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
            });

            self.push_token(token);
            self.inside
                .push(Arc::new(Mutex::new(InsideToken::If(IfToken {
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
                    args: tokens.into_iter().map(Arc::new).collect(),
                    location: self.location(),
                }));
            }
        }

        let parts = segment.splitn(2, "(").collect::<Vec<&str>>()[0]
            .splitn(3, ".")
            .collect::<Vec<&str>>();

        if parts.len() > 1 && parts[0] == "self" {
            match parts.len() {
                2 => {
                    if let Some(token) = self.parse_expression(parts[1]) {
                        return Some(Token::LetAssign(LetAssignToken {
                            name: parts[1].to_string(),
                            value: Arc::new(token),
                        }));
                    }
                }
                3 => {
                    if parts[1] == "#" {
                        let value = parts[2].splitn(2, "=").collect::<Vec<&str>>();
                        let name = value[0].trim();
                        let value = value[1].trim();

                        if let Some(token) = self.parse_expression(value) {
                            return Some(Token::LetAssign(LetAssignToken {
                                name: name.to_string(),
                                value: Arc::new(token),
                            }));
                        }
                    }
                }
                _ => {}
            }
        }

        for token in self.current_tokens_context().iter().rev() {
            if let Token::Let(let_token) = token {
                match parts.len() {
                    // regular function call
                    1 => {
                        if segment.starts_with(&format!("{}::", let_token.name)) {
                            let fn_name = segment[let_token.name.len() + 2..]
                                .split("(")
                                .collect::<Vec<&str>>()[0];

                            return Some(Token::StaticClassFnCall(StaticClassFnCallToken {
                                name: fn_name.to_string(),
                                class: let_token.name.clone(),
                                args: Vec::new(),
                            }));
                        } else if segment.starts_with(&format!("{}(", let_token.name)) {
                            let tokens = self
                                .parse_args(&segment[let_token.name.len() + 1..segment.len() - 1]);

                            return Some(Token::FnCall(FnCallToken {
                                name: let_token.name.clone(),
                                args: tokens.into_iter().map(Arc::new).collect(),
                                location: self.location(),
                            }));
                        }
                    }
                    // function call on a class
                    2 => {
                        if segment.starts_with(&format!("{}.{}(", let_token.name, parts[1])) {
                            let tokens = self.parse_args(
                                &segment[parts[0].len() + parts[1].len() + 2..segment.len() - 1],
                            );

                            return Some(Token::ClassFnCall(ClassFnCallToken {
                                name: parts[1].to_string(),
                                instance: parts[0].to_string(),
                                args: tokens.into_iter().map(Arc::new).collect(),
                            }));
                        }
                    }
                    // set a class property
                    3 => {
                        panic!("unable to use class property in {}", self.location);
                    }
                    _ => {}
                };

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

        if segment.starts_with("new") {
            let parts: Vec<&str> = segment.split("(").collect();
            if parts.len() != 2 {
                return None;
            }

            let class = parts[0][4..].trim();

            for token in self.current_tokens_context().iter().rev() {
                if let Token::Let(let_token) = token {
                    if segment.starts_with(&format!("new {}", let_token.name)) {
                        let args = self.parse_args(parts[1][0..parts[1].len() - 1].trim());

                        return Some(ExpressionToken::ClassInstantiation(
                            ClassInstantiationToken {
                                class: class.to_string(),
                                args: args.into_iter().map(Arc::new).collect(),
                            },
                        ));
                    }
                }
            }
        }

        let number = segment.parse::<f64>();
        if let Ok(number) = number {
            return Some(ExpressionToken::Value(ValueToken::Number(NumberToken {
                location: self.location(),
                value: number,
            })));
        }

        // check for ranges (x..x)
        {
            let parts = segment.splitn(2, "..").collect::<Vec<&str>>();
            if let (Some(left), Some(right)) = (parts.first(), parts.get(1)) {
                if !left.is_empty() && !right.is_empty() {
                    let left = self.parse_expression(left);
                    let right = self.parse_expression(right);

                    if let (Some(start), Some(end)) = (left, right) {
                        return Some(ExpressionToken::Value(ValueToken::Range(RangeToken {
                            location: self.location(),
                            start: Arc::new(RwLock::new(start)),
                            end: Arc::new(RwLock::new(end)),
                        })));
                    }
                }
            }
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

        let parts = segment.splitn(2, "(").collect::<Vec<&str>>()[0]
            .splitn(3, ".")
            .collect::<Vec<&str>>();

        if parts.len() > 1 && parts[0] == "self" {
            match parts.len() {
                2 => {
                    if let Some(token) = self.parse_expression(parts[1]) {
                        return Some(ExpressionToken::Let(LetToken {
                            name: parts[1].to_string(),
                            is_const: false,
                            is_function: false,
                            is_class: false,
                            value: Arc::new(RwLock::new(token)),
                        }));
                    }
                }
                3 => {
                    if parts[1] == "#" {
                        if let Some(token) = self.parse_expression(parts[2]) {
                            return Some(ExpressionToken::Let(LetToken {
                                name: parts[2].to_string(),
                                is_const: false,
                                is_function: false,
                                is_class: false,
                                value: Arc::new(RwLock::new(token)),
                            }));
                        }
                    }
                }
                _ => {}
            }
        }

        for token in self.current_tokens_context().iter().rev() {
            if let Token::Let(let_token) = token {
                match parts.len() {
                    // regular function call
                    1 => {
                        if segment.starts_with(&format!("{}::", let_token.name)) {
                            let fn_name = segment[let_token.name.len() + 2..]
                                .split("(")
                                .collect::<Vec<&str>>()[0];

                            return Some(ExpressionToken::StaticClassFnCall(
                                StaticClassFnCallToken {
                                    name: fn_name.to_string(),
                                    class: let_token.name.clone(),
                                    args: Vec::new(),
                                },
                            ));
                        } else if segment.starts_with(&format!("{}(", let_token.name)) {
                            let tokens = self
                                .parse_args(&segment[let_token.name.len() + 1..segment.len() - 1]);

                            return Some(ExpressionToken::FnCall(FnCallToken {
                                name: let_token.name.clone(),
                                args: tokens.into_iter().map(Arc::new).collect(),
                                location: self.location(),
                            }));
                        }
                    }
                    // function call on a class
                    2 => {
                        if segment.starts_with(&format!("{}.{}(", let_token.name, parts[1])) {
                            let tokens = self.parse_args(
                                &segment[parts[0].len() + parts[1].len() + 2..segment.len() - 1],
                            );

                            return Some(ExpressionToken::ClassFnCall(ClassFnCallToken {
                                name: parts[1].to_string(),
                                instance: parts[0].to_string(),
                                args: tokens.into_iter().map(Arc::new).collect(),
                            }));
                        }
                    }
                    // get a class property
                    3 => {
                        if parts[1] != "#" {
                            panic!("unexpected expression in {} (did you typo?)", self.location);
                        }

                        let property = parts[2];

                        if let ExpressionToken::Value(ValueToken::Class(class_token)) =
                            &*let_token.value.read().unwrap()
                        {
                            if class_token.name == parts[0] {
                                for token in class_token.body.read().unwrap().iter() {
                                    if let Token::Let(let_token) = token {
                                        if let_token.name == property {
                                            return Some(ExpressionToken::Let(LetToken {
                                                name: property.to_string(),
                                                is_const: let_token.is_const,
                                                is_function: let_token.is_function,
                                                is_class: let_token.is_class,
                                                value: Arc::clone(&let_token.value),
                                            }));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                };

                if segment == let_token.name {
                    return Some(ExpressionToken::Let(LetToken {
                        name: let_token.name.clone(),
                        is_const: let_token.is_const,
                        is_function: matches!(
                            &*let_token.value.read().unwrap(),
                            ExpressionToken::Value(ValueToken::Function(_))
                        ),
                        is_class: matches!(
                            &*let_token.value.read().unwrap(),
                            ExpressionToken::Value(ValueToken::Class(_))
                        ),
                        value: Arc::clone(&let_token.value),
                    }));
                }
            }
        }

        // comparison parsing
        {
            let mut left = String::new();
            let mut right = String::new();
            let mut operator = None;

            let mut on_left = true;
            for c in segment.chars() {
                if on_left && left.ends_with(" ") {
                    left.pop();

                    for o in COMPARISON_OPERATORS {
                        if left.ends_with(o) {
                            operator = ComparisonToken::parse_operator(o);
                            on_left = false;

                            for _ in 0..o.len() {
                                left.pop();
                            }

                            continue;
                        }
                    }
                }

                if on_left {
                    left.push(c);
                } else {
                    right.push(c);
                }
            }

            if let Some(operator) = operator {
                let left = self.parse_expression(left.trim());
                let right = self.parse_expression(right.trim());

                if left.is_none() || right.is_none() {
                    panic!("unexpected value in {} (did you typo?)", self.location);
                }

                return Some(ExpressionToken::Comparison(ComparisonToken {
                    left: Arc::new(left.unwrap()),
                    right: Arc::new(right.unwrap()),
                    operator,
                }));
            }
        }

        // math parsing attempt
        {
            let mut context = meval::Context::empty();

            for token in self.current_tokens_context().iter().rev() {
                if let Token::Let(let_token) = token {
                    context.var(&let_token.name, 1.0);
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
