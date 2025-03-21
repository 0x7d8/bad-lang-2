use super::{base::{ClassInstanceToken, ClassToken, ValueToken}, Token, TokenLocation};

use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub enum NumOperation {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone)]
pub struct LetToken {
    pub name: String,
    pub is_const: bool,
    pub is_function: bool,
    pub is_class: bool,
    pub value: Arc<RwLock<ExpressionToken>>,

    #[allow(dead_code)]
    pub location: TokenLocation,
}

#[derive(Debug, Clone)]
pub enum ExpressionToken {
    Return(ReturnToken),
    FnCall(FnCallToken),
    Value(ValueToken),
    Math(meval::Expr),
    Let(LetToken),
}

#[derive(Debug, Clone)]
pub struct LetAssignToken {
    pub name: String,
    pub value: Arc<ExpressionToken>,
}

#[derive(Debug, Clone)]
pub struct LetAssignNumToken {
    pub name: String,
    pub operation: NumOperation,
    pub value: Arc<ExpressionToken>,
}

#[derive(Debug, Clone)]
pub struct FnCallToken {
    pub name: String,
    pub class: Option<ClassToken>,
    pub class_instance: Option<ClassInstanceToken>,
    pub args: Vec<Arc<ExpressionToken>>,

    pub location: TokenLocation,
}

#[derive(Debug, Clone)]
pub struct LoopToken {
    pub body: Arc<RwLock<Vec<Token>>>,
}

#[derive(Debug, Clone)]
pub struct IfToken {
    pub reversed: bool,
    pub condition: Arc<ExpressionToken>,
    pub body: Arc<RwLock<Vec<Token>>>,

    #[allow(dead_code)]
    pub location: TokenLocation,
}

#[derive(Debug, Clone, Copy)]
pub struct BreakToken;

#[derive(Debug, Clone)]
pub struct ReturnToken {
    pub value: Arc<ExpressionToken>,
}
