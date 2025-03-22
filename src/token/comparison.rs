use std::sync::Arc;

use super::logic::ExpressionToken;

#[derive(Debug, Clone)]
pub struct ComparisonToken {
    pub left: Arc<ExpressionToken>,
    pub right: Arc<ExpressionToken>,
    pub operator: ComparisonOperator,
}

#[derive(Debug, Clone, Copy)]
pub enum ComparisonOperator {
    Equals,
    NotEquals,
    EqualsStrict,
    NotEqualsStrict,
    LessThan,
    LessThanEquals,
    GreaterThan,
    GreaterThanEquals,
}

pub const COMPARISON_OPERATORS: [&str; 8] = ["===", "!==", "==", "!=", "<=", "<", ">=", ">"];

impl ComparisonToken {
    pub fn parse_operator(operator: &str) -> Option<ComparisonOperator> {
        match operator {
            "===" => Some(ComparisonOperator::EqualsStrict),
            "!==" => Some(ComparisonOperator::NotEqualsStrict),
            "==" => Some(ComparisonOperator::Equals),
            "!=" => Some(ComparisonOperator::NotEquals),
            "<=" => Some(ComparisonOperator::LessThanEquals),
            "<" => Some(ComparisonOperator::LessThan),
            ">=" => Some(ComparisonOperator::GreaterThanEquals),
            ">" => Some(ComparisonOperator::GreaterThan),
            _ => None,
        }
    }
}
