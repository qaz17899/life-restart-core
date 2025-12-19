//! Property tests for talent module
//!
//! Feature: life-restart-rust
//! Property 7: Talent Trigger Count Limit
//! Property 8: Bidirectional Exclusion Check
//! Validates: Requirements 5.1, 5.5

use proptest::prelude::*;
use std::collections::HashMap;

use crate::config::TalentConfig;
use crate::property::PropertyState;
use crate::talent::processor::process_talents;
use crate::talent::replacer::check_exclusion;

// ═══════════════════════════════════════════════════════════════════════════
// Strategy generators for property tests
// ═══════════════════════════════════════════════════════════════════════════

/// Generate a simple talent config
fn talent_config_strategy(id: i32, max_triggers: i32) -> TalentConfig {
    TalentConfig {
        id,
        name: format!("Talent {}", id),
        description: format!("Description for talent {}", id),
        grade: 1,
        max_triggers,
        condition: None,
        effect: None,
        exclusive: false,
        exclude: None,
        replacement: None,
        status: 0,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Property Tests
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// Property 7: Talent should trigger at most max_triggers times
    /// Validates: Requirement 5.1 (Talent Trigger Count Limit)
    #[test]
    fn prop_talent_trigger_limit(
        max_triggers in 1..=10i32,
        iterations in 1..=20usize
    ) {
        let mut talents = HashMap::new();
        talents.insert(1, talent_config_strategy(1, max_triggers));

        let state = PropertyState {
            tlt: vec![1],
            ..Default::default()
        };

        let mut trigger_counts = HashMap::new();
        let mut total_triggers = 0;

        for _ in 0..iterations {
            let results = process_talents(&state, &talents, &mut trigger_counts);
            total_triggers += results.len();
        }

        prop_assert!(
            total_triggers <= max_triggers as usize,
            "Talent triggered {} times, but max_triggers is {}",
            total_triggers, max_triggers
        );
    }

    /// Property 7.2: Talent with condition should only trigger when condition is met
    /// Validates: Requirement 5.2 (Talent Condition)
    #[test]
    fn prop_talent_condition_check(
        chr in -10..=20i32,
        threshold in 0..=10i32
    ) {
        let mut talents = HashMap::new();
        talents.insert(1, TalentConfig {
            id: 1,
            name: "Conditional Talent".to_string(),
            description: "".to_string(),
            grade: 1,
            max_triggers: 100, // High limit to not interfere
            condition: Some(format!("CHR>{}", threshold)),
            effect: None,
            exclusive: false,
            exclude: None,
            replacement: None,
            status: 0,
        });

        let state = PropertyState {
            chr,
            tlt: vec![1],
            ..Default::default()
        };

        let mut trigger_counts = HashMap::new();
        let results = process_talents(&state, &talents, &mut trigger_counts);

        if chr > threshold {
            prop_assert_eq!(results.len(), 1, "Talent should trigger when CHR {} > {}", chr, threshold);
        } else {
            prop_assert_eq!(results.len(), 0, "Talent should not trigger when CHR {} <= {}", chr, threshold);
        }
    }

    /// Property 8: Bidirectional exclusion check
    /// If A excludes B, then check_exclusion([A], B) should return Some
    /// AND check_exclusion([B], A) should also return Some (bidirectional)
    /// Validates: Requirement 5.5 (Bidirectional Exclusion)
    #[test]
    fn prop_bidirectional_exclusion(
        a_id in 1..=100i32,
        b_id in 101..=200i32
    ) {
        let mut talents = HashMap::new();

        // A excludes B
        talents.insert(a_id, TalentConfig {
            id: a_id,
            name: format!("Talent A {}", a_id),
            description: "".to_string(),
            grade: 1,
            max_triggers: 1,
            condition: None,
            effect: None,
            exclusive: false,
            exclude: Some(vec![b_id]),
            replacement: None,
            status: 0,
        });

        // B has no exclusions
        talents.insert(b_id, TalentConfig {
            id: b_id,
            name: format!("Talent B {}", b_id),
            description: "".to_string(),
            grade: 1,
            max_triggers: 1,
            condition: None,
            effect: None,
            exclusive: false,
            exclude: None,
            replacement: None,
            status: 0,
        });

        // Check if B can be added when A is present
        // A's exclude list contains B, so should return Some(a_id)
        let result1 = check_exclusion(&[a_id], b_id, &talents);
        prop_assert!(result1.is_some(), "A excludes B, so check_exclusion([A], B) should return Some");

        // Check if A can be added when B is present (bidirectional)
        // A's exclude list contains B (which is in talents), so should return Some(b_id)
        let result2 = check_exclusion(&[b_id], a_id, &talents);
        prop_assert!(result2.is_some(), "Bidirectional: check_exclusion([B], A) should also return Some");
    }

    /// Property 8.2: No exclusion when talents don't exclude each other
    /// Validates: Requirement 5.5 (Bidirectional Exclusion)
    #[test]
    fn prop_no_exclusion_when_independent(
        a_id in 1..=100i32,
        b_id in 101..=200i32
    ) {
        let mut talents = HashMap::new();

        // A has no exclusions
        talents.insert(a_id, TalentConfig {
            id: a_id,
            name: format!("Talent A {}", a_id),
            description: "".to_string(),
            grade: 1,
            max_triggers: 1,
            condition: None,
            effect: None,
            exclusive: false,
            exclude: None,
            replacement: None,
            status: 0,
        });

        // B has no exclusions
        talents.insert(b_id, TalentConfig {
            id: b_id,
            name: format!("Talent B {}", b_id),
            description: "".to_string(),
            grade: 1,
            max_triggers: 1,
            condition: None,
            effect: None,
            exclusive: false,
            exclude: None,
            replacement: None,
            status: 0,
        });

        // Neither excludes the other
        let result1 = check_exclusion(&[a_id], b_id, &talents);
        let result2 = check_exclusion(&[b_id], a_id, &talents);

        prop_assert!(result1.is_none(), "Independent talents should not exclude each other");
        prop_assert!(result2.is_none(), "Independent talents should not exclude each other");
    }

    /// Property 8.3: Multiple talents with exclusions
    /// Validates: Requirement 5.5 (Bidirectional Exclusion)
    #[test]
    fn prop_multiple_talent_exclusion(
        a_id in 1..=50i32,
        b_id in 51..=100i32,
        c_id in 101..=150i32
    ) {
        let mut talents = HashMap::new();

        // A excludes B
        talents.insert(a_id, TalentConfig {
            id: a_id,
            name: format!("Talent A {}", a_id),
            description: "".to_string(),
            grade: 1,
            max_triggers: 1,
            condition: None,
            effect: None,
            exclusive: false,
            exclude: Some(vec![b_id]),
            replacement: None,
            status: 0,
        });

        // B excludes C
        talents.insert(b_id, TalentConfig {
            id: b_id,
            name: format!("Talent B {}", b_id),
            description: "".to_string(),
            grade: 1,
            max_triggers: 1,
            condition: None,
            effect: None,
            exclusive: false,
            exclude: Some(vec![c_id]),
            replacement: None,
            status: 0,
        });

        // C has no exclusions
        talents.insert(c_id, TalentConfig {
            id: c_id,
            name: format!("Talent C {}", c_id),
            description: "".to_string(),
            grade: 1,
            max_triggers: 1,
            condition: None,
            effect: None,
            exclusive: false,
            exclude: None,
            replacement: None,
            status: 0,
        });

        // A and B exclude each other (bidirectional)
        prop_assert!(check_exclusion(&[a_id], b_id, &talents).is_some());
        prop_assert!(check_exclusion(&[b_id], a_id, &talents).is_some());

        // B and C exclude each other (bidirectional)
        prop_assert!(check_exclusion(&[b_id], c_id, &talents).is_some());
        prop_assert!(check_exclusion(&[c_id], b_id, &talents).is_some());

        // A and C don't exclude each other
        prop_assert!(check_exclusion(&[a_id], c_id, &talents).is_none());
        prop_assert!(check_exclusion(&[c_id], a_id, &talents).is_none());
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_property_tests_compile() {
        assert!(true);
    }
}
