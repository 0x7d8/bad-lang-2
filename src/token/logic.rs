use super::{Token, TokenLocation, base::ValueToken, comparison::ComparisonToken};

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
}

#[derive(Debug, Clone)]
pub enum ExpressionToken {
    Comparison(ComparisonToken),
    Return(ReturnToken),
    FnCall(FnCallToken),
    ClassInstantiation(ClassInstantiationToken),
    StaticClassFnCall(StaticClassFnCallToken),
    ClassFnCall(ClassFnCallToken),
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
    pub args: Vec<Arc<ExpressionToken>>,

    pub location: TokenLocation,
}

#[derive(Debug, Clone)]
pub struct ClassInstantiationToken {
    pub class: String,
    pub args: Vec<Arc<ExpressionToken>>,
}

#[derive(Debug, Clone)]
pub struct StaticClassFnCallToken {
    pub name: String,
    pub class: String,
    pub args: Vec<Arc<ExpressionToken>>,
}

#[derive(Debug, Clone)]
pub struct ClassFnCallToken {
    pub name: String,
    pub instance: String,
    pub args: Vec<Arc<ExpressionToken>>,
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
}

#[derive(Debug, Clone, Copy)]
pub struct BreakToken;

#[derive(Debug, Clone)]
pub struct ReturnToken {
    pub value: Arc<ExpressionToken>,
}
