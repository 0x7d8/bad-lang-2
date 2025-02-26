use crate::token::{
    InsideToken, Token,
    base::{BaseToken, NullToken, ValueToken},
    logic::{ExpressionToken, LetToken, NumOperation, ReturnToken},
    runtime,
};

use std::sync::Arc;
use std::{collections::HashMap, sync::Mutex};

pub struct Runtime {
    tokens: Vec<Token>,
    call_stack: Vec<InsideToken>,
    scopes: Vec<HashMap<String, Arc<Mutex<ExpressionToken>>>>,
}

impl Runtime {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            call_stack: Vec::new(),
            scopes: vec![HashMap::new()],
        }
    }

    pub fn run(&mut self) {
        for token in self.tokens.clone() {
            self.execute(&token);
        }
    }

    fn scope_set(&mut self, name: &str, value: Arc<Mutex<ExpressionToken>>) {
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.to_string(), value);
    }

    fn scope_aggregate(&self) -> HashMap<String, Arc<Mutex<ExpressionToken>>> {
        let mut aggregate = HashMap::new();

        for scope in self.scopes.iter().rev() {
            for (name, value) in scope.iter() {
                if !aggregate.contains_key(name) {
                    aggregate.insert(name.clone(), Arc::clone(value));
                }
            }
        }

        aggregate
    }

    fn scope_create(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn execute(&mut self, token: &Token) -> Option<ExpressionToken> {
        match token {
            Token::Let(let_token) => {
                let value = self
                    .extract_value(&let_token.value.lock().unwrap())
                    .unwrap();

                for variable in self.scopes.last().unwrap().iter() {
                    if variable.0 == &let_token.name {
                        return Some(ExpressionToken::Value(ValueToken::Null(NullToken {
                            location: Default::default(),
                        })));
                    }
                }

                self.scope_set(
                    &let_token.name,
                    Arc::new(Mutex::new(ExpressionToken::Value(value))),
                );
            }
            Token::Loop(loop_token) => {
                self.call_stack.push(InsideToken::Loop(loop_token.clone()));
                self.scope_create();

                let body = loop_token.body.lock().unwrap();

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

                    self.scopes.last_mut().unwrap().clear();
                }

                self.scopes.pop();
                self.call_stack.pop();
            }
            Token::If(if_token) => {
                self.call_stack.push(InsideToken::If(if_token.clone()));

                let condition = self.extract_value(&if_token.condition).unwrap();

                if (if_token.reversed && !condition.truthy())
                    || (!if_token.reversed && condition.truthy())
                {
                    self.scope_create();

                    let body = if_token.body.lock().unwrap();

                    for token in body.iter() {
                        let value = self.execute(token);

                        if value.is_none() {
                            self.scopes.pop();
                            self.call_stack.pop();
                            return None;
                        } else if let Some(ExpressionToken::Return(return_token)) = value {
                            self.scopes.pop();
                            self.call_stack.pop();
                            return Some(ExpressionToken::Return(return_token));
                        }
                    }

                    self.scopes.pop();
                }

                self.call_stack.pop();
            }
            Token::Break(_) => {
                for token in self.call_stack.iter().rev() {
                    if let InsideToken::Loop(_) = token {
                        return None;
                    }
                }
            }
            Token::Return(token) => {
                let value = self.extract_value(&token.value).unwrap();

                return Some(ExpressionToken::Return(ReturnToken {
                    value: Arc::new(ExpressionToken::Value(value)),
                }));
            }
            Token::FnCall(call_token) => {
                if runtime::FUNCTIONS.contains(&call_token.name.as_str()) {
                    let args: Vec<Arc<ExpressionToken>> = call_token
                        .args
                        .iter()
                        .map(|arg| Arc::new((*arg.lock().unwrap()).clone()))
                        .collect();

                    let result = runtime::run(call_token.name.as_str(), &args, self);

                    return result;
                }

                for variable in self.scope_aggregate().iter() {
                    if variable.1.try_lock().is_err() {
                        continue;
                    }

                    if let ValueToken::Function(fn_token) =
                        self.extract_value(&*variable.1.lock().unwrap()).unwrap()
                    {
                        if variable.0 == &call_token.name {
                            self.call_stack
                                .push(InsideToken::Function(fn_token.clone()));
                            self.scope_create();

                            for (index, arg) in fn_token.args.iter().enumerate() {
                                if let Some(arg_expr) = call_token.args.get(index) {
                                    let extracted =
                                        self.extract_value(&arg_expr.lock().unwrap()).unwrap();

                                    self.scope_set(
                                        arg,
                                        Arc::new(Mutex::new(ExpressionToken::Value(extracted))),
                                    );
                                }
                            }

                            let body = fn_token.body.lock().unwrap();

                            for token in body.iter() {
                                let value = self.execute(token);

                                if value.is_none() {
                                    break;
                                } else if let Some(ExpressionToken::Return(return_token)) = value {
                                    self.call_stack.pop();
                                    return Some(ExpressionToken::Return(return_token));
                                }
                            }

                            self.scopes.pop();
                            self.call_stack.pop();
                        }
                    }
                }
            }
            Token::LetAssign(assign_token) => {
                let value = self.extract_value(&assign_token.value).unwrap();
                let expr_value = ExpressionToken::Value(value);

                for variable in self.scope_aggregate().iter() {
                    if variable.0 == &assign_token.name {
                        *variable.1.lock().unwrap() = expr_value;
                        break;
                    }
                }
            }
            Token::LetAssignNum(assign_token) => {
                let value = self.extract_value(&assign_token.value).unwrap();

                for variable in self.scope_aggregate().iter() {
                    if variable.0 == &assign_token.name {
                        let mut var_ref = variable.1.lock().unwrap();

                        if let ExpressionToken::Value(ValueToken::Number(ref mut number_token)) =
                            *var_ref
                        {
                            if let ValueToken::Number(value_token) = &value {
                                match assign_token.operation {
                                    NumOperation::Add => {
                                        number_token.value += value_token.value;
                                    }
                                    NumOperation::Sub => {
                                        number_token.value -= value_token.value;
                                    }
                                    NumOperation::Mul => {
                                        number_token.value *= value_token.value;
                                    }
                                    NumOperation::Div => {
                                        number_token.value /= value_token.value;
                                    }
                                }
                            }
                        } else {
                            *var_ref = ExpressionToken::Value(value.clone());
                        }
                    }
                }
            }
        }

        Some(ExpressionToken::Value(ValueToken::Null(NullToken {
            location: Default::default(),
        })))
    }

    pub fn extract_value(&mut self, token: &ExpressionToken) -> Option<ValueToken> {
        match token {
            ExpressionToken::Value(value) => Some(value.clone()),
            ExpressionToken::Let(LetToken { name, .. }) => {
                for variable in self.scope_aggregate().iter() {
                    if variable.0 == name {
                        return self.extract_value(&*variable.1.lock().unwrap());
                    }
                }

                None
            }
            ExpressionToken::FnCall(value) => {
                let value =
                    self.execute(&Token::FnCall(value.clone()))
                        .unwrap_or(ExpressionToken::Value(ValueToken::Null(NullToken {
                            location: Default::default(),
                        })));

                self.extract_value(&value)
            }
            ExpressionToken::Return(value) => self.extract_value(&value.value),
        }
    }
}
