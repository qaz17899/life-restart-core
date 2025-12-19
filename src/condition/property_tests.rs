//! Property tests for condition module
//!
//! Feature: life-restart-rust
//! Property 1: Condition Parsing Correctness
//! Validates: Requirements 2.1, 2.2, 2.3, 2.4

use proptest::prelude::*;

use crate::condition::ast::{AstNode, ConditionValue, Operator};
use crate::condition::cache::{check_condition, clear_cache};
use crate::condition::evaluator::check;
use crate::condition::parser::parse;
use crate::property::PropertyState;

// ═══════════════════════════════════════════════════════════════════════════
// Strategy generators for property tests
// ═══════════════════════════════════════════════════════════════════════════

/// Generate valid property names
fn property_name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("AGE".to_string()),
        Just("CHR".to_string()),
        Just("INT".to_string()),
        Just("STR".to_string()),
        Just("MNY".to_string()),
        Just("SPR".to_string()),
        Just("LIF".to_string()),
        Just("HAGE".to_string()),
        Just("HCHR".to_string()),
        Just("HINT".to_string()),
        Just("HSTR".to_string()),
        Just("HMNY".to_string()),
        Just("HSPR".to_string()),
        Just("LAGE".to_string()),
        Just("LCHR".to_string()),
        Just("LINT".to_string()),
        Just("LSTR".to_string()),
        Just("LMNY".to_string()),
        Just("LSPR".to_string()),
    ]
}

/// Generate list property names
fn list_property_name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![Just("TLT".to_string()), Just("EVT".to_string()),]
}

/// Generate comparison operators
fn comparison_operator_strategy() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just(">"),
        Just("<"),
        Just(">="),
        Just("<="),
        Just("="),
        Just("!="),
    ]
}

/// Generate array operators
fn array_operator_strategy() -> impl Strategy<Value = &'static str> {
    prop_oneof![Just("?"), Just("!"),]
}

/// Generate integer values in reasonable range
fn integer_value_strategy() -> impl Strategy<Value = i32> {
    -100..=100i32
}

/// Generate array values
fn array_value_strategy() -> impl Strategy<Value = Vec<i32>> {
    prop::collection::vec(1..=10000i32, 1..=5)
}

/// Generate a simple condition string
fn simple_condition_strategy() -> impl Strategy<Value = String> {
    (property_name_strategy(), comparison_operator_strategy(), integer_value_strategy())
        .prop_map(|(prop, op, val)| format!("{}{}{}", prop, op, val))
}

/// Generate an array condition string
fn array_condition_strategy() -> impl Strategy<Value = String> {
    (list_property_name_strategy(), array_operator_strategy(), array_value_strategy()).prop_map(
        |(prop, op, arr)| {
            let arr_str = arr
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",");
            format!("{}{}[{}]", prop, op, arr_str)
        },
    )
}

/// Generate a PropertyState with random values
fn property_state_strategy() -> impl Strategy<Value = PropertyState> {
    (
        -10..=100i32,  // age
        -10..=20i32,   // chr
        -10..=20i32,   // int
        -10..=20i32,   // str
        -10..=20i32,   // mny
        -10..=20i32,   // spr
        0..=10i32,     // lif
        prop::collection::vec(1..=10000i32, 0..=10), // tlt
        prop::collection::vec(1..=100000i32, 0..=20), // evt
    )
        .prop_map(|(age, chr, int, str_, mny, spr, lif, tlt, evt)| {
            let mut state = PropertyState::new(chr, int, str_, mny, spr, lif);
            state.age = age;
            state.tlt = tlt;
            state.evt = evt;
            // Update min/max based on current values
            state.lage = state.lage.min(age);
            state.hage = state.hage.max(age);
            state
        })
}

// ═══════════════════════════════════════════════════════════════════════════
// Property Tests
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// Property 1.1: Simple conditions should parse without error
    /// Validates: Requirement 2.1 (Condition Parsing)
    #[test]
    fn prop_simple_condition_parses(cond in simple_condition_strategy()) {
        let result = parse(&cond);
        prop_assert!(result.is_ok(), "Failed to parse: {}", cond);
    }

    /// Property 1.2: Array conditions should parse without error
    /// Validates: Requirement 2.2 (Array Operators)
    #[test]
    fn prop_array_condition_parses(cond in array_condition_strategy()) {
        let result = parse(&cond);
        prop_assert!(result.is_ok(), "Failed to parse: {}", cond);
    }

    /// Property 1.3: Parsed conditions should have correct structure
    /// Validates: Requirement 2.1 (Condition Parsing)
    #[test]
    fn prop_parsed_condition_structure(
        prop in property_name_strategy(),
        val in integer_value_strategy()
    ) {
        let cond = format!("{}>={}", prop, val);
        let ast = parse(&cond).unwrap();

        match ast {
            AstNode::Single(single) => {
                prop_assert_eq!(single.property, prop);
                prop_assert_eq!(single.operator, Operator::GreaterEqual);
                prop_assert_eq!(single.value, ConditionValue::Integer(val));
            }
            _ => prop_assert!(false, "Expected single condition"),
        }
    }

    /// Property 1.4: AND conditions should evaluate correctly
    /// Validates: Requirement 2.3 (Logical Operators)
    #[test]
    fn prop_and_condition_evaluation(
        chr_threshold in 0..=10i32,
        int_threshold in 0..=10i32,
        state in property_state_strategy()
    ) {
        let cond = format!("CHR>={} & INT>={}", chr_threshold, int_threshold);
        let ast = parse(&cond).unwrap();
        let result = check(&ast, &state);

        let expected = state.chr >= chr_threshold && state.int >= int_threshold;
        prop_assert_eq!(result, expected, "Condition: {}, CHR={}, INT={}", cond, state.chr, state.int);
    }

    /// Property 1.5: OR conditions should evaluate correctly
    /// Validates: Requirement 2.3 (Logical Operators)
    #[test]
    fn prop_or_condition_evaluation(
        chr_threshold in 0..=10i32,
        int_threshold in 0..=10i32,
        state in property_state_strategy()
    ) {
        let cond = format!("CHR>={} | INT>={}", chr_threshold, int_threshold);
        let ast = parse(&cond).unwrap();
        let result = check(&ast, &state);

        let expected = state.chr >= chr_threshold || state.int >= int_threshold;
        prop_assert_eq!(result, expected, "Condition: {}, CHR={}, INT={}", cond, state.chr, state.int);
    }

    /// Property 1.6: Comparison operators should be mathematically correct
    /// Validates: Requirement 2.1 (Condition Parsing)
    #[test]
    fn prop_comparison_operators(
        prop_val in -10..=20i32,
        threshold in -10..=20i32
    ) {
        let state = PropertyState {
            chr: prop_val,
            ..Default::default()
        };

        // Greater than
        let ast = parse(&format!("CHR>{}", threshold)).unwrap();
        prop_assert_eq!(check(&ast, &state), prop_val > threshold);

        // Less than
        let ast = parse(&format!("CHR<{}", threshold)).unwrap();
        prop_assert_eq!(check(&ast, &state), prop_val < threshold);

        // Greater or equal
        let ast = parse(&format!("CHR>={}", threshold)).unwrap();
        prop_assert_eq!(check(&ast, &state), prop_val >= threshold);

        // Less or equal
        let ast = parse(&format!("CHR<={}", threshold)).unwrap();
        prop_assert_eq!(check(&ast, &state), prop_val <= threshold);

        // Equal
        let ast = parse(&format!("CHR={}", threshold)).unwrap();
        prop_assert_eq!(check(&ast, &state), prop_val == threshold);

        // Not equal
        let ast = parse(&format!("CHR!={}", threshold)).unwrap();
        prop_assert_eq!(check(&ast, &state), prop_val != threshold);
    }

    /// Property 1.7: Includes any (?) should work correctly
    /// Validates: Requirement 2.2 (Array Operators)
    #[test]
    fn prop_includes_any_operator(
        list in prop::collection::vec(1..=100i32, 1..=5),
        check_values in prop::collection::vec(1..=100i32, 1..=3)
    ) {
        let state = PropertyState {
            tlt: list.clone(),
            ..Default::default()
        };

        let arr_str = check_values.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",");
        let cond = format!("TLT?[{}]", arr_str);
        let ast = parse(&cond).unwrap();
        let result = check(&ast, &state);

        let expected = list.iter().any(|v| check_values.contains(v));
        prop_assert_eq!(result, expected, "List: {:?}, Check: {:?}", list, check_values);
    }

    /// Property 1.8: Excludes all (!) should work correctly
    /// Validates: Requirement 2.2 (Array Operators)
    #[test]
    fn prop_excludes_all_operator(
        list in prop::collection::vec(1..=100i32, 1..=5),
        check_values in prop::collection::vec(1..=100i32, 1..=3)
    ) {
        let state = PropertyState {
            evt: list.clone(),
            ..Default::default()
        };

        let arr_str = check_values.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",");
        let cond = format!("EVT![{}]", arr_str);
        let ast = parse(&cond).unwrap();
        let result = check(&ast, &state);

        let expected = list.iter().all(|v| !check_values.contains(v));
        prop_assert_eq!(result, expected, "List: {:?}, Check: {:?}", list, check_values);
    }

    /// Property 1.9: Cache should return same results as direct parsing
    /// Validates: Requirement 2.5 (Condition Caching)
    #[test]
    fn prop_cache_consistency(
        cond in simple_condition_strategy(),
        state in property_state_strategy()
    ) {
        clear_cache();

        // Direct parse and check
        let ast = parse(&cond).unwrap();
        let direct_result = check(&ast, &state);

        // Cached check (first call)
        let cached_result1 = check_condition(&cond, &state).unwrap();

        // Cached check (second call - should hit cache)
        let cached_result2 = check_condition(&cond, &state).unwrap();

        prop_assert_eq!(direct_result, cached_result1);
        prop_assert_eq!(cached_result1, cached_result2);
    }

    /// Property 1.10: Empty condition should always return true
    /// Validates: Requirement 2.4 (Condition Evaluation)
    #[test]
    fn prop_empty_condition_returns_true(state in property_state_strategy()) {
        let result = check_condition("", &state).unwrap();
        prop_assert!(result, "Empty condition should return true");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_property_tests_compile() {
        // This test just ensures the property tests compile correctly
        assert!(true);
    }
}
