use crate::token::{
    InsideToken, Token,
    base::{BaseToken, NullToken, NumberToken, ValueToken},
    logic::{ExpressionToken, LetToken, NumOperation, ReturnToken},
    runtime,
};

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

pub struct Runtime {
    tokens: Vec<Token>,
    call_stack: Vec<InsideToken>,
    scopes: Vec<HashMap<String, Arc<RwLock<ExpressionToken>>>>,

    lookup_cache: RefCell<HashMap<String, Arc<RwLock<ExpressionToken>>>>,
    modified_vars: RefCell<HashSet<String>>,
}

impl Runtime {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            call_stack: Vec::new(),
            scopes: vec![HashMap::new()],
            lookup_cache: RefCell::new(HashMap::new()),
            modified_vars: RefCell::new(HashSet::new()),
        }
    }

    pub fn run(&mut self) {
        let tokens_clone = self.tokens.clone();

        for token in tokens_clone {
            self.execute(&token);
        }
    }

    fn scope_set(&mut self, name: &str, value: Arc<RwLock<ExpressionToken>>) {
        self.modified_vars.borrow_mut().insert(name.to_string());
        self.lookup_cache
            .borrow_mut()
            .insert(name.to_string(), Arc::clone(&value));

        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.to_string(), value);
    }

    fn lookup_variable(&self, name: &str) -> Option<Arc<RwLock<ExpressionToken>>> {
        if !self.modified_vars.borrow().contains(name) {
            if let Some(value) = self.lookup_cache.borrow().get(name) {
                return Some(Arc::clone(value));
            }
        }

        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                self.lookup_cache
                    .borrow_mut()
                    .insert(name.to_string(), Arc::clone(value));
                self.modified_vars.borrow_mut().remove(name);

                return Some(Arc::clone(value));
            }
        }

        None
    }

    pub fn scope_aggregate(&self) -> HashMap<String, Arc<RwLock<ExpressionToken>>> {
        if self.modified_vars.borrow().is_empty() && !self.lookup_cache.borrow().is_empty() {
            return self.lookup_cache.borrow().clone();
        }

        self.rebuild_lookup_cache();
        self.lookup_cache.borrow().clone()
    }

    fn rebuild_lookup_cache(&self) {
        let mut cache = self.lookup_cache.borrow_mut();
        cache.clear();

        for scope in self.scopes.iter().rev() {
            for (name, value) in scope.iter() {
                if !cache.contains_key(name) {
                    cache.insert(name.clone(), Arc::clone(value));
                }
            }
        }

        self.modified_vars.borrow_mut().clear();
    }

    fn scope_create(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn execute(&mut self, token: &Token) -> Option<ExpressionToken> {
        match token {
            Token::Let(let_token) => {
                let value = self
                    .extract_value(&let_token.value.read().unwrap())
                    .unwrap();

                if self.scopes.last().unwrap().contains_key(&let_token.name) {
                    return Some(ExpressionToken::Value(ValueToken::Null(NullToken {
                        location: Default::default(),
                    })));
                }

                self.scope_set(
                    &let_token.name,
                    Arc::new(RwLock::new(ExpressionToken::Value(value))),
                );
            }
            Token::Loop(loop_token) => {
                self.call_stack.push(InsideToken::Loop(loop_token.clone()));
                self.scope_create();

                let body = loop_token.body.read().unwrap();

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

                    self.modified_vars.borrow_mut().clear();
                    self.lookup_cache.borrow_mut().clear();
                }

                self.scopes.pop();
                self.call_stack.pop();

                self.rebuild_lookup_cache();
            }
            Token::If(if_token) => {
                self.call_stack.push(InsideToken::If(if_token.clone()));

                let condition = self.extract_value(&if_token.condition).unwrap();

                if (if_token.reversed && !condition.truthy())
                    || (!if_token.reversed && condition.truthy())
                {
                    self.scope_create();

                    let body = if_token.body.read().unwrap();

                    for token in body.iter() {
                        let value = self.execute(token);

                        if value.is_none() {
                            self.scopes.pop();
                            self.call_stack.pop();

                            self.rebuild_lookup_cache();
                            return None;
                        } else if let Some(ExpressionToken::Return(return_token)) = value {
                            self.scopes.pop();
                            self.call_stack.pop();

                            self.rebuild_lookup_cache();
                            return Some(ExpressionToken::Return(return_token));
                        }
                    }

                    self.scopes.pop();
                    self.rebuild_lookup_cache();
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
                    let result = runtime::run(
                        call_token.name.as_str(),
                        &call_token.args,
                        self,
                        &call_token.location,
                    );

                    return result;
                }

                let fn_var = self.lookup_variable(&call_token.name);

                if let Some(fn_var) = fn_var {
                    if fn_var.try_read().is_err() {
                        return Some(ExpressionToken::Value(ValueToken::Null(NullToken {
                            location: Default::default(),
                        })));
                    }

                    if let ValueToken::Function(fn_token) =
                        self.extract_value(&fn_var.read().unwrap()).unwrap()
                    {
                        self.call_stack
                            .push(InsideToken::Function(fn_token.clone()));
                        self.scope_create();

                        for (index, arg) in fn_token.args.iter().enumerate() {
                            if let Some(arg_expr) = call_token.args.get(index) {
                                let extracted = self.extract_value(arg_expr).unwrap();

                                self.scope_set(
                                    arg,
                                    Arc::new(RwLock::new(ExpressionToken::Value(extracted))),
                                );
                            }
                        }

                        let body = fn_token.body.read().unwrap();

                        for token in body.iter() {
                            let value = self.execute(token);

                            if value.is_none() {
                                break;
                            } else if let Some(ExpressionToken::Return(return_token)) = value {
                                self.scopes.pop();
                                self.call_stack.pop();

                                self.rebuild_lookup_cache();
                                return Some(ExpressionToken::Return(return_token));
                            }
                        }

                        self.scopes.pop();
                        self.call_stack.pop();

                        self.rebuild_lookup_cache();
                    }
                }
            }
            Token::LetAssign(assign_token) => {
                let value = self.extract_value(&assign_token.value).unwrap();
                let expr_value = ExpressionToken::Value(value);

                if let Some(var) = self.lookup_variable(&assign_token.name) {
                    *var.write().unwrap() = expr_value;

                    self.modified_vars
                        .borrow_mut()
                        .insert(assign_token.name.clone());
                }
            }
            Token::LetAssignNum(assign_token) => {
                let value = self.extract_value(&assign_token.value).unwrap();

                if let Some(var) = self.lookup_variable(&assign_token.name) {
                    let mut var_ref = var.write().unwrap();

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

                    self.modified_vars
                        .borrow_mut()
                        .insert(assign_token.name.clone());
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
                if let Some(var) = self.lookup_variable(name) {
                    if let Ok(guard) = var.read() {
                        return self.extract_value(&guard);
                    }
                }

                None
            }
            ExpressionToken::Math(expression) => {
                let mut context = meval::Context::empty();

                for (name, value) in self.scope_aggregate() {
                    if let Ok(guard) = value.read() {
                        if let ValueToken::Number(number_token) =
                            self.extract_value(&guard).unwrap()
                        {
                            context.var(name, number_token.value);
                        }
                    }
                }

                let result = expression.eval_with_context(&context);
                if let Ok(value) = result {
                    Some(ValueToken::Number(NumberToken {
                        location: Default::default(),
                        value,
                    }))
                } else {
                    None
                }
            }
            ExpressionToken::FnCall(value) => {
                let value_clone = value.clone();
                let value =
                    self.execute(&Token::FnCall(value_clone))
                        .unwrap_or(ExpressionToken::Value(ValueToken::Null(NullToken {
                            location: Default::default(),
                        })));

                self.extract_value(&value)
            }
            ExpressionToken::Return(value) => self.extract_value(&value.value),
        }
    }
}
