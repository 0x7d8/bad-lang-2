use crate::token::{
    Token,
    base::{BaseToken, NullToken, ValueToken},
    logic::{ExpressionToken, LetToken},
    runtime,
};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

struct Scope {
    variables: HashMap<String, Rc<RefCell<ExpressionToken>>>,
    parent: Option<Rc<RefCell<Scope>>>,
}

impl Scope {
    fn new(parent: Option<Rc<RefCell<Scope>>>) -> Self {
        Self {
            variables: HashMap::new(),
            parent,
        }
    }

    fn get(&self, name: &str) -> Option<Rc<RefCell<ExpressionToken>>> {
        if let Some(value) = self.variables.get(name) {
            Some(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
    }

    fn set(&mut self, name: &str, value: Rc<RefCell<ExpressionToken>>) {
        self.variables.insert(name.to_string(), value);
    }
}

pub struct Runtime {
    tokens: Vec<Token>,
    current_scope: Rc<RefCell<Scope>>,
    call_stack: Vec<Token>,
}

impl Runtime {
    pub fn new(tokens: Vec<Token>) -> Self {
        let global_scope = Rc::new(RefCell::new(Scope::new(None)));

        for token in &tokens {
            if let Token::Let(let_token) = token {
                global_scope
                    .borrow_mut()
                    .set(&let_token.name, let_token.value.clone());
            }
        }

        Self {
            tokens,
            current_scope: global_scope,
            call_stack: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        for token in self.tokens.clone() {
            self.execute(&token);
        }
    }

    fn execute(&mut self, token: &Token) -> Option<ExpressionToken> {
        match token {
            Token::Let(let_token) => {
                let value = self.extract_value(&let_token.value.borrow()).unwrap();
                *let_token.value.borrow_mut() = ExpressionToken::Value(value);
            }
            Token::Loop(loop_token) => {
                self.call_stack.push(Token::Loop(loop_token.clone()));

                let loop_scope =
                    Rc::new(RefCell::new(Scope::new(Some(self.current_scope.clone()))));
                let old_scope = std::mem::replace(&mut self.current_scope, loop_scope);

                for token in loop_token.body.borrow().iter() {
                    if let Token::Let(let_token) = token {
                        self.current_scope
                            .borrow_mut()
                            .set(&let_token.name, let_token.value.clone());
                    }
                }

                let body = loop_token.body.borrow();

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

                self.current_scope = old_scope;
                self.call_stack.pop();
            }
            Token::If(if_token) => {
                self.call_stack.push(Token::If(if_token.clone()));

                let condition = self.extract_value(&if_token.condition).unwrap();

                if (if_token.reversed && !condition.truthy())
                    || (!if_token.reversed && condition.truthy())
                {
                    let if_scope =
                        Rc::new(RefCell::new(Scope::new(Some(self.current_scope.clone()))));
                    let old_scope = std::mem::replace(&mut self.current_scope, if_scope);

                    for token in if_token.body.borrow().iter() {
                        if let Token::Let(let_token) = token {
                            self.current_scope
                                .borrow_mut()
                                .set(&let_token.name, let_token.value.clone());
                        }
                    }

                    let body = if_token.body.borrow();

                    for token in body.iter() {
                        if self.execute(token).is_none() {
                            self.current_scope = old_scope;
                            self.call_stack.pop();
                            return None;
                        }
                    }

                    self.current_scope = old_scope;
                }

                self.call_stack.pop();
            }
            Token::Break(_) => {
                for token in self.call_stack.iter().rev() {
                    if let Token::Loop(_) = token {
                        return None;
                    }
                }
            }
            Token::Return(token) => {
                let value = self.extract_value(&token.value).unwrap();
                return Some(ExpressionToken::Value(value));
            }
            Token::FnCall(call_token) => {
                if runtime::FUNCTIONS.contains(&call_token.name.as_str()) {
                    return runtime::run(call_token.name.as_str(), &call_token.args, self);
                }

                let fn_token_opt = self.tokens.iter().find_map(|token| {
                    if let Token::Fn(fn_token) = token {
                        if call_token.name == fn_token.name {
                            Some(fn_token.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });

                if let Some(fn_token) = fn_token_opt {
                    self.call_stack.push(Token::Fn(fn_token.clone()));

                    let fn_scope =
                        Rc::new(RefCell::new(Scope::new(Some(self.current_scope.clone()))));
                    let old_scope = std::mem::replace(&mut self.current_scope, fn_scope);

                    for token in fn_token.body.borrow().iter() {
                        if let Token::Let(let_token) = token {
                            self.current_scope
                                .borrow_mut()
                                .set(&let_token.name, let_token.value.clone());
                        }
                    }

                    let mut evaluated_args = Vec::new();
                    for (index, arg_name) in fn_token.args.iter().enumerate() {
                        let arg_expr = call_token
                            .args
                            .get(index)
                            .unwrap_or(&Rc::new(ExpressionToken::Value(ValueToken::Null(
                                NullToken {
                                    location: Default::default(),
                                },
                            ))))
                            .clone();

                        let arg_value = self.extract_value(&arg_expr).unwrap();
                        evaluated_args.push((arg_name.clone(), arg_value));
                    }

                    for (arg_name, arg_value) in evaluated_args {
                        for token in fn_token.body.borrow().iter() {
                            if let Token::Let(let_token) = token {
                                if let_token.name == arg_name {
                                    *let_token.value.borrow_mut() =
                                        ExpressionToken::Value(arg_value.clone());
                                    break;
                                }
                            }
                        }
                    }

                    let mut return_value = None;
                    let body_tokens = fn_token.body.borrow().clone();

                    for token in body_tokens.iter() {
                        if let Token::Return(return_token) = token {
                            let return_expr = return_token.value.clone();
                            let value = self.extract_value(&return_expr).unwrap();
                            return_value = Some(ExpressionToken::Value(value));
                            break;
                        } else if self.execute(token).is_none() {
                            break;
                        }
                    }

                    self.current_scope = old_scope;
                    self.call_stack.pop();

                    return return_value;
                }
            }
            Token::LetAssign(assign_token) => {
                let value = self.extract_value(&assign_token.value).unwrap();
                let expr_value = ExpressionToken::Value(value);

                if let Some(var) = self.current_scope.borrow().get(&assign_token.name) {
                    *var.borrow_mut() = expr_value;
                } else {
                    self.current_scope
                        .borrow_mut()
                        .set(&assign_token.name, Rc::new(RefCell::new(expr_value)));
                }
            }
            _ => {}
        }

        Some(ExpressionToken::Value(ValueToken::Null(NullToken {
            location: Default::default(),
        })))
    }

    pub fn extract_value(&mut self, token: &ExpressionToken) -> Option<ValueToken> {
        match token {
            ExpressionToken::Value(value) => Some(value.clone()),
            ExpressionToken::Let(LetToken { name, .. }) => {
                let var_opt = self.current_scope.borrow().get(name).clone();

                if let Some(var) = var_opt {
                    if let ExpressionToken::Value(value) = &*var.borrow() {
                        Some(value.clone())
                    } else {
                        self.extract_value(&var.borrow())
                    }
                } else {
                    Some(ValueToken::Null(NullToken {
                        location: Default::default(),
                    }))
                }
            }
            ExpressionToken::FnCall(value) => {
                let value =
                    self.execute(&Token::FnCall(value.clone()))
                        .unwrap_or(ExpressionToken::Value(ValueToken::Null(NullToken {
                            location: Default::default(),
                        })));

                self.extract_value(&value)
            }
        }
    }
}
