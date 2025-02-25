use crate::token::{
    Token,
    base::{BaseToken, NullToken, ValueToken},
    logic::{ExpressionToken, LetToken, NumOperation, ReturnToken},
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

    fn update(&mut self, name: &str, value: ExpressionToken) -> bool {
        if let Some(var) = self.variables.get(name) {
            *var.borrow_mut() = value;
            return true;
        } else if let Some(parent) = &self.parent {
            return parent.borrow_mut().update(name, value);
        }
        false
    }
}

pub struct Runtime {
    tokens: Vec<Token>,
    current_scope: Rc<RefCell<Scope>>,
    call_stack: Vec<Token>,
    globals: HashMap<String, Rc<RefCell<ExpressionToken>>>,
}

impl Runtime {
    pub fn new(tokens: Vec<Token>) -> Self {
        let global_scope = Rc::new(RefCell::new(Scope::new(None)));
        let mut globals = HashMap::new();

        for token in &tokens {
            if let Token::Let(let_token) = token {
                let var = let_token.value.clone();
                global_scope.borrow_mut().set(&let_token.name, var.clone());

                globals.insert(let_token.name.clone(), var);
            }
        }

        Self {
            tokens,
            current_scope: global_scope,
            call_stack: Vec::new(),
            globals,
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
                let expr_value = ExpressionToken::Value(value);
                *let_token.value.borrow_mut() = expr_value.clone();

                if self.globals.contains_key(&let_token.name) {
                    *self.globals.get(&let_token.name).unwrap().borrow_mut() = expr_value;
                }
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
                        let value = self.execute(token);

                        if value.is_none() {
                            self.current_scope = old_scope;
                            self.call_stack.pop();
                            return None;
                        } else if let Some(ExpressionToken::Return(return_token)) = value {
                            self.current_scope = old_scope;
                            self.call_stack.pop();
                            return Some(ExpressionToken::Return(return_token));
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

                for token in self.call_stack.iter().rev() {
                    if let Token::Fn(_) = token {
                        return Some(ExpressionToken::Return(ReturnToken {
                            value: Rc::new(ExpressionToken::Value(value)),
                        }));
                    }
                }
            }
            Token::FnCall(call_token) => {
                if runtime::FUNCTIONS.contains(&call_token.name.as_str()) {
                    let mut arg_refs = Vec::new();
                    for arg in &call_token.args {
                        arg_refs.push(arg.clone());
                    }

                    let result = runtime::run(call_token.name.as_str(), &call_token.args, self);

                    return result;
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
                            if !fn_token.args.contains(&let_token.name) {
                                self.current_scope
                                    .borrow_mut()
                                    .set(&let_token.name, let_token.value.clone());
                            }
                        }
                    }

                    let mut evaluated_args = Vec::new();
                    for (index, arg_name) in fn_token.args.iter().enumerate() {
                        if let Some(arg_expr) = call_token.args.get(index) {
                            if let ExpressionToken::Let(let_token) = &**arg_expr {
                                if let Some(original_var) =
                                    self.find_original_variable(&let_token.name)
                                {
                                    evaluated_args.push((arg_name.clone(), original_var));
                                    continue;
                                }
                            }

                            let arg_value = self.extract_value(&arg_expr).unwrap();
                            let value_expr = ExpressionToken::Value(arg_value);
                            let value_ref = Rc::new(RefCell::new(value_expr));
                            evaluated_args.push((arg_name.clone(), value_ref));
                        } else {
                            let null_value = ExpressionToken::Value(ValueToken::Null(NullToken {
                                location: Default::default(),
                            }));
                            evaluated_args
                                .push((arg_name.clone(), Rc::new(RefCell::new(null_value))));
                        }
                    }

                    for (arg_name, arg_value) in evaluated_args {
                        self.current_scope.borrow_mut().set(&arg_name, arg_value);
                    }

                    let mut return_value = None;
                    let body_tokens = fn_token.body.borrow().clone();

                    for token in body_tokens.iter() {
                        let value = self.execute(token);

                        if value.is_none() {
                            break;
                        } else if let Some(ExpressionToken::Return(return_token)) = value {
                            return_value = Some((*return_token.value).clone());
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

                let updated = self
                    .current_scope
                    .borrow_mut()
                    .update(&assign_token.name, expr_value.clone());

                if !updated {
                    self.current_scope.borrow_mut().set(
                        &assign_token.name,
                        Rc::new(RefCell::new(expr_value.clone())),
                    );
                }

                if self.globals.contains_key(&assign_token.name) {
                    *self.globals.get(&assign_token.name).unwrap().borrow_mut() = expr_value;
                }
            }
            Token::LetAssignNum(assign_token) => {
                let value = self.extract_value(&assign_token.value).unwrap();

                if let Some(var) = self.current_scope.borrow().get(&assign_token.name) {
                    let mut var_ref = var.borrow_mut();

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

                    if self.globals.contains_key(&assign_token.name) {
                        *self.globals.get(&assign_token.name).unwrap().borrow_mut() =
                            ExpressionToken::Value(value);
                    }
                } else {
                    let value_expr = ExpressionToken::Value(value.clone());
                    let value_ref = Rc::new(RefCell::new(value_expr));

                    self.current_scope
                        .borrow_mut()
                        .set(&assign_token.name, value_ref.clone());

                    if self.call_stack.is_empty() {
                        self.globals.insert(assign_token.name.clone(), value_ref);
                    }
                }
            }
            _ => {}
        }

        Some(ExpressionToken::Value(ValueToken::Null(NullToken {
            location: Default::default(),
        })))
    }

    fn find_original_variable(&self, name: &str) -> Option<Rc<RefCell<ExpressionToken>>> {
        if let Some(global_var) = self.globals.get(name) {
            return Some(global_var.clone());
        }

        self.current_scope.borrow().get(name)
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
                    if let Some(global_var) = self.globals.get(name).cloned() {
                        if let ExpressionToken::Value(value) = &*global_var.borrow() {
                            Some(value.clone())
                        } else {
                            self.extract_value(&global_var.borrow())
                        }
                    } else {
                        Some(ValueToken::Null(NullToken {
                            location: Default::default(),
                        }))
                    }
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
            ExpressionToken::Return(value) => self.extract_value(&value.value),
        }
    }
}
