//! Property tests for event module
//!
//! Feature: life-restart-rust
//! Property 4: Event Selection Filtering
//! Property 5: Weighted Random Distribution
//! Property 6: Event Branch Evaluation Order
//! Validates: Requirements 4.1, 4.2, 4.3

use proptest::prelude::*;
use std::collections::HashMap;

use crate::config::{EventBranch, EventConfig};
use crate::event::processor::process_event;
use crate::event::selector::{select_event, weighted_random};
use crate::property::PropertyState;

// ═══════════════════════════════════════════════════════════════════════════
// Strategy generators for property tests
// ═══════════════════════════════════════════════════════════════════════════

/// Generate weighted items for random selection
fn weighted_items_strategy() -> impl Strategy<Value = Vec<(i32, f64)>> {
    prop::collection::vec((1..=1000i32, 0.1..=10.0f64), 1..=10)
}

/// Generate a PropertyState for testing
fn property_state_strategy() -> impl Strategy<Value = PropertyState> {
    (
        -10..=100i32, // age
        -10..=20i32,  // chr
        -10..=20i32,  // int
        -10..=20i32,  // str
        -10..=20i32,  // mny
        -10..=20i32,  // spr
        1..=10i32,    // lif
    )
        .prop_map(|(age, chr, int, str_, mny, spr, lif)| {
            let mut state = PropertyState::new(chr, int, str_, mny, spr, lif);
            state.age = age;
            state
        })
}

// ═══════════════════════════════════════════════════════════════════════════
// Property Tests
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// Property 4.1: NoRandom events should never be selected
    /// Validates: Requirement 4.1 (Event Selection Filtering)
    #[test]
    fn prop_no_random_events_excluded(
        state in property_state_strategy()
    ) {
        let mut events = HashMap::new();
        events.insert(1, EventConfig {
            id: 1,
            event: "Random event".to_string(),
            grade: 1,
            no_random: false,
            include: None,
            exclude: None,
            effect: None,
            branch: None,
            post_event: None,
        });
        events.insert(2, EventConfig {
            id: 2,
            event: "NoRandom event".to_string(),
            grade: 1,
            no_random: true,
            include: None,
            exclude: None,
            effect: None,
            branch: None,
            post_event: None,
        });

        let pool = vec![(1, 1.0), (2, 1.0)];

        // Run selection multiple times
        for _ in 0..100 {
            if let Some(selected) = select_event(&pool, &events, &state) {
                prop_assert_ne!(selected, 2, "NoRandom event should never be selected");
            }
        }
    }

    /// Property 4.2: Events with failing include condition should not be selected
    /// Validates: Requirement 4.1 (Event Selection Filtering)
    #[test]
    fn prop_include_condition_filtering(
        chr in -10..=20i32
    ) {
        let mut state = PropertyState::default();
        state.chr = chr;

        let mut events = HashMap::new();
        events.insert(1, EventConfig {
            id: 1,
            event: "Conditional event".to_string(),
            grade: 1,
            no_random: false,
            include: Some("CHR>10".to_string()),
            exclude: None,
            effect: None,
            branch: None,
            post_event: None,
        });
        events.insert(2, EventConfig {
            id: 2,
            event: "Always available".to_string(),
            grade: 1,
            no_random: false,
            include: None,
            exclude: None,
            effect: None,
            branch: None,
            post_event: None,
        });

        let pool = vec![(1, 1.0), (2, 1.0)];

        // Run selection multiple times
        for _ in 0..100 {
            if let Some(selected) = select_event(&pool, &events, &state) {
                if chr <= 10 {
                    prop_assert_ne!(selected, 1, "Event with failing include should not be selected");
                }
            }
        }
    }

    /// Property 4.3: Events with passing exclude condition should not be selected
    /// Validates: Requirement 4.1 (Event Selection Filtering)
    #[test]
    fn prop_exclude_condition_filtering(
        int in -10..=20i32
    ) {
        let mut state = PropertyState::default();
        state.int = int;

        let mut events = HashMap::new();
        events.insert(1, EventConfig {
            id: 1,
            event: "Excludable event".to_string(),
            grade: 1,
            no_random: false,
            include: None,
            exclude: Some("INT>10".to_string()),
            effect: None,
            branch: None,
            post_event: None,
        });
        events.insert(2, EventConfig {
            id: 2,
            event: "Always available".to_string(),
            grade: 1,
            no_random: false,
            include: None,
            exclude: None,
            effect: None,
            branch: None,
            post_event: None,
        });

        let pool = vec![(1, 1.0), (2, 1.0)];

        // Run selection multiple times
        for _ in 0..100 {
            if let Some(selected) = select_event(&pool, &events, &state) {
                if int > 10 {
                    prop_assert_ne!(selected, 1, "Event with passing exclude should not be selected");
                }
            }
        }
    }

    /// Property 5: Weighted random should select items proportionally
    /// Validates: Requirement 4.2 (Weighted Random Selection)
    #[test]
    fn prop_weighted_random_proportional(
        weight1 in 1.0..=10.0f64,
        weight2 in 1.0..=10.0f64
    ) {
        let items = vec![(1, weight1), (2, weight2)];
        let mut counts = [0u32, 0u32];
        let iterations = 10000;

        for _ in 0..iterations {
            if let Some(id) = weighted_random(&items) {
                counts[(id - 1) as usize] += 1;
            }
        }

        // Check that the ratio is approximately correct (within 20% tolerance)
        let expected_ratio = weight1 / weight2;
        let actual_ratio = counts[0] as f64 / counts[1] as f64;

        // Allow 30% tolerance for statistical variation
        let tolerance = 0.3;
        let lower_bound = expected_ratio * (1.0 - tolerance);
        let upper_bound = expected_ratio * (1.0 + tolerance);

        prop_assert!(
            actual_ratio >= lower_bound && actual_ratio <= upper_bound,
            "Ratio {} not within expected range [{}, {}] for weights ({}, {})",
            actual_ratio, lower_bound, upper_bound, weight1, weight2
        );
    }

    /// Property 5.2: Weighted random should always return Some for non-empty input
    /// Validates: Requirement 4.2 (Weighted Random Selection)
    #[test]
    fn prop_weighted_random_always_returns(
        items in weighted_items_strategy()
    ) {
        let result = weighted_random(&items);
        prop_assert!(result.is_some(), "weighted_random should return Some for non-empty input");
    }

    /// Property 6: Event branches should be evaluated in order
    /// Validates: Requirement 4.3 (Event Branch Evaluation Order)
    #[test]
    fn prop_branch_evaluation_order(
        chr in 0..=20i32
    ) {
        let mut state = PropertyState::default();
        state.chr = chr;

        let mut events = HashMap::new();
        events.insert(1, EventConfig {
            id: 1,
            event: "Branching event".to_string(),
            grade: 1,
            no_random: false,
            include: None,
            exclude: None,
            effect: None,
            branch: Some(vec![
                EventBranch {
                    condition: "CHR>15".to_string(),
                    event_id: 100,
                },
                EventBranch {
                    condition: "CHR>10".to_string(),
                    event_id: 200,
                },
                EventBranch {
                    condition: "CHR>5".to_string(),
                    event_id: 300,
                },
            ]),
            post_event: None,
        });

        let result = process_event(1, &events, &state).unwrap();

        // First matching branch should be selected
        if chr > 15 {
            prop_assert_eq!(result.next_event_id, Some(100), "CHR>15 should select branch 100");
        } else if chr > 10 {
            prop_assert_eq!(result.next_event_id, Some(200), "CHR>10 should select branch 200");
        } else if chr > 5 {
            prop_assert_eq!(result.next_event_id, Some(300), "CHR>5 should select branch 300");
        } else {
            prop_assert!(result.next_event_id.is_none(), "No branch should match for CHR<=5");
        }
    }

    /// Property 6.2: Event without branches should have no next_event_id
    /// Validates: Requirement 4.3 (Event Branch Evaluation Order)
    #[test]
    fn prop_no_branch_no_next_event(
        state in property_state_strategy()
    ) {
        let mut events = HashMap::new();
        events.insert(1, EventConfig {
            id: 1,
            event: "Simple event".to_string(),
            grade: 1,
            no_random: false,
            include: None,
            exclude: None,
            effect: None,
            branch: None,
            post_event: None,
        });

        let result = process_event(1, &events, &state).unwrap();
        prop_assert!(result.next_event_id.is_none(), "Event without branches should have no next_event_id");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_property_tests_compile() {
        assert!(true);
    }
}
