//! Condition string parser

use crate::condition::ast::{AstNode, ConditionValue, Operator, SingleCondition};
use crate::error::{LifeRestartError, Result};

/// Parse a condition string into an AST
pub fn parse(condition: &str) -> Result<AstNode> {
    let condition = condition.trim();
    if condition.is_empty() {
        return Err(LifeRestartError::InvalidCondition(
            "Empty condition".to_string(),
        ));
    }

    let tokens = tokenize(condition)?;
    parse_tokens(&tokens)
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Condition(String),
    And,
    Or,
    OpenParen,
    CloseParen,
}

fn tokenize(condition: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = condition.chars().peekable();
    let mut paren_depth = 0;

    while let Some(c) = chars.next() {
        match c {
            ' ' => {
                if !current.is_empty() {
                    tokens.push(Token::Condition(current.clone()));
                    current.clear();
                }
            }
            '(' => {
                if !current.is_empty() {
                    tokens.push(Token::Condition(current.clone()));
                    current.clear();
                }
                tokens.push(Token::OpenParen);
                paren_depth += 1;
            }
            ')' => {
                if !current.is_empty() {
                    tokens.push(Token::Condition(current.clone()));
                    current.clear();
                }
                tokens.push(Token::CloseParen);
                paren_depth -= 1;
            }
            '&' => {
                if !current.is_empty() {
                    tokens.push(Token::Condition(current.clone()));
                    current.clear();
                }
                tokens.push(Token::And);
            }
            '|' => {
                if !current.is_empty() {
                    tokens.push(Token::Condition(current.clone()));
                    current.clear();
                }
                tokens.push(Token::Or);
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        tokens.push(Token::Condition(current));
    }

    if paren_depth != 0 {
        return Err(LifeRestartError::InvalidCondition(
            "Unbalanced parentheses".to_string(),
        ));
    }

    Ok(tokens)
}

fn parse_tokens(tokens: &[Token]) -> Result<AstNode> {
    if tokens.is_empty() {
        return Err(LifeRestartError::InvalidCondition(
            "Empty token list".to_string(),
        ));
    }

    // Find the lowest precedence operator (OR has lower precedence than AND)
    let mut paren_depth = 0;
    let mut or_pos = None;
    let mut and_pos = None;

    for (i, token) in tokens.iter().enumerate() {
        match token {
            Token::OpenParen => paren_depth += 1,
            Token::CloseParen => paren_depth -= 1,
            Token::Or if paren_depth == 0 => or_pos = Some(i),
            Token::And if paren_depth == 0 && or_pos.is_none() => and_pos = Some(i),
            _ => {}
        }
    }

    // Handle OR (lowest precedence)
    if let Some(pos) = or_pos {
        let left = parse_tokens(&tokens[..pos])?;
        let right = parse_tokens(&tokens[pos + 1..])?;
        return Ok(AstNode::Or(Box::new(left), Box::new(right)));
    }

    // Handle AND
    if let Some(pos) = and_pos {
        let left = parse_tokens(&tokens[..pos])?;
        let right = parse_tokens(&tokens[pos + 1..])?;
        return Ok(AstNode::And(Box::new(left), Box::new(right)));
    }

    // Handle parentheses
    if tokens.len() >= 2 {
        if let (Token::OpenParen, Token::CloseParen) = (&tokens[0], &tokens[tokens.len() - 1]) {
            return parse_tokens(&tokens[1..tokens.len() - 1]);
        }
    }

    // Single condition
    if tokens.len() == 1 {
        if let Token::Condition(cond) = &tokens[0] {
            return parse_single_condition(cond);
        }
    }

    Err(LifeRestartError::InvalidCondition(format!(
        "Cannot parse tokens: {:?}",
        tokens
    )))
}

fn parse_single_condition(condition: &str) -> Result<AstNode> {
    // Find operator position
    let operators = [">=", "<=", "!=", ">", "<", "=", "?", "!"];

    for op_str in operators {
        if let Some(pos) = condition.find(op_str) {
            let property = condition[..pos].trim().to_string();
            let value_str = condition[pos + op_str.len()..].trim();

            let operator = match op_str {
                ">" => Operator::Greater,
                "<" => Operator::Less,
                ">=" => Operator::GreaterEqual,
                "<=" => Operator::LessEqual,
                "=" => Operator::Equal,
                "!=" => Operator::NotEqual,
                "?" => Operator::IncludesAny,
                "!" => Operator::ExcludesAll,
                _ => unreachable!(),
            };

            let value = parse_value(value_str)?;

            return Ok(AstNode::Single(SingleCondition {
                property,
                operator,
                value,
            }));
        }
    }

    Err(LifeRestartError::InvalidCondition(format!(
        "No operator found in: {}",
        condition
    )))
}

fn parse_value(value_str: &str) -> Result<ConditionValue> {
    let value_str = value_str.trim();

    // Try to parse as array
    if value_str.starts_with('[') && value_str.ends_with(']') {
        let inner = &value_str[1..value_str.len() - 1];
        let values: std::result::Result<Vec<i32>, _> = inner
            .split(',')
            .map(|s| s.trim().parse::<i32>())
            .collect();

        match values {
            Ok(arr) => return Ok(ConditionValue::Array(arr)),
            Err(_) => {
                return Err(LifeRestartError::InvalidCondition(format!(
                    "Invalid array: {}",
                    value_str
                )))
            }
        }
    }

    // Try to parse as integer
    if let Ok(i) = value_str.parse::<i32>() {
        return Ok(ConditionValue::Integer(i));
    }

    // Try to parse as float
    if let Ok(f) = value_str.parse::<f64>() {
        return Ok(ConditionValue::Float(f));
    }

    // Treat as string
    Ok(ConditionValue::String(value_str.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_condition() {
        let ast = parse("CHR>5").unwrap();
        match ast {
            AstNode::Single(cond) => {
                assert_eq!(cond.property, "CHR");
                assert_eq!(cond.operator, Operator::Greater);
                assert_eq!(cond.value, ConditionValue::Integer(5));
            }
            _ => panic!("Expected single condition"),
        }
    }

    #[test]
    fn test_parse_and_condition() {
        let ast = parse("CHR>5 & INT<10").unwrap();
        match ast {
            AstNode::And(_, _) => {}
            _ => panic!("Expected AND condition"),
        }
    }

    #[test]
    fn test_parse_or_condition() {
        let ast = parse("CHR>5 | INT<10").unwrap();
        match ast {
            AstNode::Or(_, _) => {}
            _ => panic!("Expected OR condition"),
        }
    }

    #[test]
    fn test_parse_array_condition() {
        let ast = parse("TLT?[1,2,3]").unwrap();
        match ast {
            AstNode::Single(cond) => {
                assert_eq!(cond.property, "TLT");
                assert_eq!(cond.operator, Operator::IncludesAny);
                assert_eq!(cond.value, ConditionValue::Array(vec![1, 2, 3]));
            }
            _ => panic!("Expected single condition"),
        }
    }

    #[test]
    fn test_parse_all_operators() {
        // Test all comparison operators
        let operators = [
            ("CHR>5", Operator::Greater),
            ("CHR<5", Operator::Less),
            ("CHR>=5", Operator::GreaterEqual),
            ("CHR<=5", Operator::LessEqual),
            ("CHR=5", Operator::Equal),
            ("CHR!=5", Operator::NotEqual),
            ("TLT?[1]", Operator::IncludesAny),
            ("TLT![1]", Operator::ExcludesAll),
        ];

        for (cond_str, expected_op) in operators {
            let ast = parse(cond_str).unwrap();
            match ast {
                AstNode::Single(cond) => {
                    assert_eq!(cond.operator, expected_op, "Failed for: {}", cond_str);
                }
                _ => panic!("Expected single condition for: {}", cond_str),
            }
        }
    }

    #[test]
    fn test_parse_nested_parentheses() {
        // (A & B) | C
        let ast = parse("(CHR>5 & INT>5) | STR>5").unwrap();
        match ast {
            AstNode::Or(left, _) => match *left {
                AstNode::And(_, _) => {}
                _ => panic!("Expected AND inside OR"),
            },
            _ => panic!("Expected OR condition"),
        }
    }

    #[test]
    fn test_parse_operator_precedence() {
        // A | B & C should be parsed as A | (B & C) because OR has lower precedence
        let ast = parse("CHR>5 | INT>5 & STR>5").unwrap();
        match ast {
            AstNode::Or(_, right) => match *right {
                AstNode::And(_, _) => {}
                _ => panic!("Expected AND on right side of OR"),
            },
            _ => panic!("Expected OR condition"),
        }
    }

    #[test]
    fn test_parse_float_value() {
        let ast = parse("CHR>5.5").unwrap();
        match ast {
            AstNode::Single(cond) => {
                assert_eq!(cond.value, ConditionValue::Float(5.5));
            }
            _ => panic!("Expected single condition"),
        }
    }

    #[test]
    fn test_parse_excludes_all() {
        let ast = parse("EVT![10001,10002]").unwrap();
        match ast {
            AstNode::Single(cond) => {
                assert_eq!(cond.property, "EVT");
                assert_eq!(cond.operator, Operator::ExcludesAll);
                assert_eq!(cond.value, ConditionValue::Array(vec![10001, 10002]));
            }
            _ => panic!("Expected single condition"),
        }
    }

    #[test]
    fn test_parse_complex_condition() {
        // Complex condition from real game data
        let ast = parse("AGE>=18 & CHR>5 & (TLT?[1001] | EVT?[10001])").unwrap();
        match ast {
            AstNode::And(_, _) => {}
            _ => panic!("Expected AND condition"),
        }
    }
}
