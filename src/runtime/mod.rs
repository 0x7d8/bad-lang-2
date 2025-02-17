use crate::token::{
    base::{BaseToken, NullToken, ValueToken},
    logic::{ExpressionToken, LetToken},
    runtime, Token,
};

use std::rc::Rc;

pub struct Runtime {
    tokens: Vec<Token>,
    inside: Vec<Token>,
}

impl Runtime {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            inside: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        for token in self.tokens.clone() {
            self.execute(&token);
        }
    }

    fn current_tokens_context(&self) -> Vec<Token> {
        let mut tokens = Vec::with_capacity(self.tokens.len());

        for token in &self.tokens {
            tokens.push(token.clone());
            self.add_nested_tokens(token, &mut tokens);
        }

        for inside in &self.inside {
            match inside {
                Token::Fn(fn_token) => {
                    for token in fn_token.body.borrow().iter().cloned() {
                        tokens.push(token.clone());
                        self.add_nested_tokens(&token, &mut tokens);
                    }
                }
                Token::Loop(loop_token) => {
                    for token in loop_token.body.borrow().iter().cloned() {
                        tokens.push(token.clone());
                        self.add_nested_tokens(&token, &mut tokens);
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
                for token in fn_token.body.borrow().iter().cloned() {
                    tokens.push(token.clone());
                    self.add_nested_tokens(&token, tokens);
                }
            }
            Token::Loop(loop_token) => {
                for token in loop_token.body.borrow().iter().cloned() {
                    tokens.push(token.clone());
                    self.add_nested_tokens(&token, tokens);
                }
            }
            _ => {}
        }
    }

    fn execute(&mut self, token: &Token) -> Option<ExpressionToken> {
        match token {
            Token::Loop(token) => {
                self.inside.push(Token::Loop(token.clone()));

                let body = token.body.borrow();

                loop {
                    let mut break_loop = false;

                    for token in body.iter() {
                        if self.execute(token).is_none() {
                            break_loop = true;
                            break;
                        }
                    }

                    if break_loop {
                        break;
                    }
                }

                self.inside.pop();
            }
            Token::If(token) => {
                self.inside.push(Token::If(token.clone()));

                let condition = self.extract_value(&*token.condition).unwrap();

                if (token.reversed && !condition.truthy())
                    || (!token.reversed && condition.truthy())
                {
                    let body = token.body.borrow();

                    for token in body.iter() {
                        if self.execute(token).is_none() {
                            self.inside.pop();
                            return None;
                        }
                    }
                }

                self.inside.pop();
            }
            Token::Break(_) => {
                self.inside.pop();
                return None;
            }
            Token::Return(token) => {
                self.inside.pop();
                return Some((*token.value).clone());
            }
            Token::FnCall(call_token) => {
                if runtime::FUNCTIONS.contains(&call_token.name.as_str()) {
                    return runtime::run(&call_token.name.as_str(), &call_token.args, self);
                }

                for token in self.current_tokens_context().iter().rev() {
                    if let Token::Fn(fn_token) = token {
                        if call_token.name != fn_token.name {
                            continue;
                        }

                        self.inside.push(Token::Fn(fn_token.clone()));

                        let mut index = 0;
                        while index < fn_token.args.len() {
                            let value = self
                                .extract_value(&*call_token.args.get(index).unwrap_or(&Rc::new(
                                    ExpressionToken::Value(ValueToken::Null(NullToken)),
                                )))
                                .unwrap();
                            match fn_token.body.borrow().get(index).unwrap() {
                                Token::Let(let_token) => {
                                    *let_token.value.borrow_mut() = ExpressionToken::Value(value);
                                }
                                _ => {}
                            };

                            index += 1;
                        }

                        for token in fn_token.body.borrow().iter() {
                            match token {
                                Token::Return(token) => {
                                    return Some((*token.value).clone());
                                }
                                _ => {
                                    if self.execute(token).is_none() {
                                        break;
                                    }
                                }
                            }
                        }

                        self.inside.pop();
                    }
                }
            }
            Token::LetAssign(assign_token) => {
                for token in self.current_tokens_context().iter() {
                    if let Token::Let(let_token) = token {
                        if assign_token.name != let_token.name {
                            continue;
                        }

                        let value = self.extract_value(&*assign_token.value).unwrap();
                        *let_token.value.borrow_mut() = ExpressionToken::Value(value);

                        break;
                    }
                }
            }
            _ => {}
        }

        Some(ExpressionToken::Value(ValueToken::Null(NullToken)))
    }

    pub fn extract_value(&mut self, token: &ExpressionToken) -> Option<ValueToken> {
        match token {
            ExpressionToken::Value(value) => Some(value.clone()),
            ExpressionToken::Let(LetToken { value, .. }) => {
                if let ExpressionToken::Value(value) = &*value.borrow() {
                    Some(value.clone())
                } else {
                    self.extract_value(&*value.borrow())
                }
            }
            ExpressionToken::FnCall(value) => {
                let value = self.execute(&Token::FnCall(value.clone())).unwrap();
                self.extract_value(&value)
            }
        }
    }
}
