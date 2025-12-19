//! Condition evaluator

use crate::condition::ast::{AstNode, ConditionValue, Operator, SingleCondition};
use crate::property::PropertyState;

/// Evaluate an AST against a PropertyState
pub fn check(ast: &AstNode, state: &PropertyState) -> bool {
    match ast {
        AstNode::Single(cond) => check_single(cond, state),
        AstNode::And(left, right) => check(left, state) && check(right, state),
        AstNode::Or(left, right) => check(left, state) || check(right, state),
    }
}

fn check_single(cond: &SingleCondition, state: &PropertyState) -> bool {
    let prop_value = state.get(&cond.property);

    match (&prop_value, &cond.value, cond.operator) {
        // Integer comparisons
        (PropertyValue::Integer(pv), ConditionValue::Integer(cv), Operator::Greater) => pv > cv,
        (PropertyValue::Integer(pv), ConditionValue::Integer(cv), Operator::Less) => pv < cv,
        (PropertyValue::Integer(pv), ConditionValue::Integer(cv), Operator::GreaterEqual) => {
            pv >= cv
        }
        (PropertyValue::Integer(pv), ConditionValue::Integer(cv), Operator::LessEqual) => pv <= cv,
        (PropertyValue::Integer(pv), ConditionValue::Integer(cv), Operator::Equal) => pv == cv,
        (PropertyValue::Integer(pv), ConditionValue::Integer(cv), Operator::NotEqual) => pv != cv,

        // Float comparisons
        (PropertyValue::Integer(pv), ConditionValue::Float(cv), Operator::Greater) => {
            (*pv as f64) > *cv
        }
        (PropertyValue::Integer(pv), ConditionValue::Float(cv), Operator::Less) => {
            (*pv as f64) < *cv
        }
        (PropertyValue::Integer(pv), ConditionValue::Float(cv), Operator::GreaterEqual) => {
            (*pv as f64) >= *cv
        }
        (PropertyValue::Integer(pv), ConditionValue::Float(cv), Operator::LessEqual) => {
            (*pv as f64) <= *cv
        }

        // List contains value (=)
        (PropertyValue::List(list), ConditionValue::Integer(cv), Operator::Equal) => {
            list.contains(cv)
        }
        // List not contains value (!=)
        (PropertyValue::List(list), ConditionValue::Integer(cv), Operator::NotEqual) => {
            !list.contains(cv)
        }

        // Includes any (?)
        (PropertyValue::List(list), ConditionValue::Array(arr), Operator::IncludesAny) => {
            list.iter().any(|v| arr.contains(v))
        }
        (PropertyValue::Integer(pv), ConditionValue::Array(arr), Operator::IncludesAny) => {
            arr.contains(pv)
        }

        // Excludes all (!)
        (PropertyValue::List(list), ConditionValue::Array(arr), Operator::ExcludesAll) => {
            list.iter().all(|v| !arr.contains(v))
        }
        (PropertyValue::Integer(pv), ConditionValue::Array(arr), Operator::ExcludesAll) => {
            !arr.contains(pv)
        }

        // Default: false for unsupported combinations
        _ => false,
    }
}

/// Property value types for evaluation
#[derive(Debug, Clone)]
pub enum PropertyValue {
    Integer(i32),
    List(Vec<i32>),
}

impl PropertyState {
    /// Get property value for condition evaluation
    pub fn get(&self, prop: &str) -> PropertyValue {
        match prop {
            "AGE" => PropertyValue::Integer(self.age),
            "CHR" => PropertyValue::Integer(self.chr),
            "INT" => PropertyValue::Integer(self.int),
            "STR" => PropertyValue::Integer(self.str_),
            "MNY" => PropertyValue::Integer(self.mny),
            "SPR" => PropertyValue::Integer(self.spr),
            "LIF" => PropertyValue::Integer(self.lif),
            "TLT" => PropertyValue::List(self.tlt.clone()),
            "EVT" => PropertyValue::List(self.evt.clone()),
            "LAGE" => PropertyValue::Integer(self.lage.min(self.age)),
            "LCHR" => PropertyValue::Integer(self.lchr.min(self.chr)),
            "LINT" => PropertyValue::Integer(self.lint.min(self.int)),
            "LSTR" => PropertyValue::Integer(self.lstr.min(self.str_)),
            "LMNY" => PropertyValue::Integer(self.lmny.min(self.mny)),
            "LSPR" => PropertyValue::Integer(self.lspr.min(self.spr)),
            "HAGE" => PropertyValue::Integer(self.hage.max(self.age)),
            "HCHR" => PropertyValue::Integer(self.hchr.max(self.chr)),
            "HINT" => PropertyValue::Integer(self.hint.max(self.int)),
            "HSTR" => PropertyValue::Integer(self.hstr.max(self.str_)),
            "HMNY" => PropertyValue::Integer(self.hmny.max(self.mny)),
            "HSPR" => PropertyValue::Integer(self.hspr.max(self.spr)),
            "SUM" => PropertyValue::Integer(self.calculate_summary_score()),
            _ => PropertyValue::Integer(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::parser::parse;

    #[test]
    fn test_simple_comparison() {
        let state = PropertyState {
            chr: 10,
            ..Default::default()
        };

        let ast = parse("CHR>5").unwrap();
        assert!(check(&ast, &state));

        let ast = parse("CHR<5").unwrap();
        assert!(!check(&ast, &state));
    }

    #[test]
    fn test_and_condition() {
        let state = PropertyState {
            chr: 10,
            int: 5,
            ..Default::default()
        };

        let ast = parse("CHR>5 & INT>=5").unwrap();
        assert!(check(&ast, &state));

        let ast = parse("CHR>5 & INT>5").unwrap();
        assert!(!check(&ast, &state));
    }

    #[test]
    fn test_list_includes() {
        let state = PropertyState {
            tlt: vec![1, 2, 3],
            ..Default::default()
        };

        let ast = parse("TLT?[1,4,5]").unwrap();
        assert!(check(&ast, &state));

        let ast = parse("TLT?[4,5,6]").unwrap();
        assert!(!check(&ast, &state));
    }

    #[test]
    fn test_list_excludes_all() {
        let state = PropertyState {
            evt: vec![1, 2, 3],
            ..Default::default()
        };

        // Should be false because evt contains 1
        let ast = parse("EVT![1,4,5]").unwrap();
        assert!(!check(&ast, &state));

        // Should be true because evt doesn't contain any of 4,5,6
        let ast = parse("EVT![4,5,6]").unwrap();
        assert!(check(&ast, &state));
    }

    #[test]
    fn test_list_equality() {
        let state = PropertyState {
            tlt: vec![1001, 1002, 1003],
            ..Default::default()
        };

        // TLT=1001 means "list contains 1001"
        let ast = parse("TLT=1001").unwrap();
        assert!(check(&ast, &state));

        let ast = parse("TLT=9999").unwrap();
        assert!(!check(&ast, &state));

        // TLT!=1001 means "list does not contain 1001"
        let ast = parse("TLT!=1001").unwrap();
        assert!(!check(&ast, &state));

        let ast = parse("TLT!=9999").unwrap();
        assert!(check(&ast, &state));
    }

    #[test]
    fn test_or_condition() {
        let state = PropertyState {
            chr: 3,
            int: 10,
            ..Default::default()
        };

        // CHR>5 is false, INT>5 is true, so OR should be true
        let ast = parse("CHR>5 | INT>5").unwrap();
        assert!(check(&ast, &state));

        // Both false
        let ast = parse("CHR>5 | INT>15").unwrap();
        assert!(!check(&ast, &state));
    }

    #[test]
    fn test_complex_condition() {
        let state = PropertyState {
            age: 20,
            chr: 10,
            tlt: vec![1001],
            ..Default::default()
        };

        // AGE>=18 & CHR>5 & TLT?[1001]
        let ast = parse("AGE>=18 & CHR>5 & TLT?[1001]").unwrap();
        assert!(check(&ast, &state));
    }

    #[test]
    fn test_min_max_properties() {
        let mut state = PropertyState::new(10, 10, 10, 10, 10, 1);
        state.change("CHR", -5); // chr = 5, lchr = 5, hchr = 10

        // HCHR should be 10 (max)
        let ast = parse("HCHR>=10").unwrap();
        assert!(check(&ast, &state));

        // LCHR should be 5 (min)
        let ast = parse("LCHR<=5").unwrap();
        assert!(check(&ast, &state));
    }

    #[test]
    fn test_float_comparison() {
        let state = PropertyState {
            chr: 6,
            ..Default::default()
        };

        let ast = parse("CHR>5.5").unwrap();
        assert!(check(&ast, &state));

        let ast = parse("CHR<5.5").unwrap();
        assert!(!check(&ast, &state));
    }

    #[test]
    fn test_integer_in_array() {
        let state = PropertyState {
            chr: 5,
            ..Default::default()
        };

        // Integer property with array check
        let ast = parse("CHR?[1,5,10]").unwrap();
        assert!(check(&ast, &state));

        let ast = parse("CHR?[1,2,3]").unwrap();
        assert!(!check(&ast, &state));

        let ast = parse("CHR![1,2,3]").unwrap();
        assert!(check(&ast, &state));

        let ast = parse("CHR![1,5,10]").unwrap();
        assert!(!check(&ast, &state));
    }
}
