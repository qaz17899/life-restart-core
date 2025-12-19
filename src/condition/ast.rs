//! Abstract Syntax Tree for condition expressions

/// AST node for condition expressions
#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    /// Single condition like "CHR>5"
    Single(SingleCondition),
    /// AND operation
    And(Box<AstNode>, Box<AstNode>),
    /// OR operation
    Or(Box<AstNode>, Box<AstNode>),
}

/// Single condition expression
#[derive(Debug, Clone, PartialEq)]
pub struct SingleCondition {
    pub property: String,
    pub operator: Operator,
    pub value: ConditionValue,
}

/// Comparison operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    /// Greater than (>)
    Greater,
    /// Less than (<)
    Less,
    /// Greater than or equal (>=)
    GreaterEqual,
    /// Less than or equal (<=)
    LessEqual,
    /// Equal (=)
    Equal,
    /// Not equal (!=)
    NotEqual,
    /// Includes any (?)
    IncludesAny,
    /// Excludes all (!)
    ExcludesAll,
}

/// Condition value types
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionValue {
    Integer(i32),
    Float(f64),
    Array(Vec<i32>),
    String(String),
}
