use super::{base::ValueToken, Token};

use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub struct LetToken {
    pub name: String,
    pub is_const: bool,
    pub value: Rc<RefCell<ExpressionToken>>,
}

#[derive(Debug, Clone)]
pub enum ExpressionToken {
    FnCall(FnCallToken),
    Value(ValueToken),
    Let(LetToken),
}

#[derive(Debug, Clone)]
pub struct LetAssignToken {
    pub name: String,
    pub value: Rc<ExpressionToken>,
}

#[derive(Debug, Clone)]
pub struct FnToken {
    pub name: String,
    pub args: Vec<String>,
    pub body: Rc<RefCell<Vec<Token>>>,
}

#[derive(Debug, Clone)]
pub struct FnCallToken {
    pub name: String,
    pub args: Vec<Rc<ExpressionToken>>,
}

#[derive(Debug, Clone)]
pub struct LoopToken {
    pub body: Rc<RefCell<Vec<Token>>>,
}

#[derive(Debug, Clone)]
pub struct IfToken {
    pub reversed: bool,
    pub condition: Rc<ExpressionToken>,
    pub body: Rc<RefCell<Vec<Token>>>,
}

#[derive(Debug, Clone)]
pub struct BreakToken;

#[derive(Debug, Clone)]
pub struct ReturnToken {
    pub value: Rc<ExpressionToken>,
}
