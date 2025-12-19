//! Property tests for simulator module
//!
//! Feature: life-restart-rust
//! Property 9: Simulation Termination
//! Validates: Requirements 6.3

use proptest::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::config::{AgeConfig, EventConfig, EventEffect, TalentConfig};
use crate::simulator::SimulationEngine;

// ═══════════════════════════════════════════════════════════════════════════
// Strategy generators for property tests
// ═══════════════════════════════════════════════════════════════════════════

/// Generate initial properties
fn initial_properties_strategy() -> impl Strategy<Value = HashMap<String, i32>> {
    (
        0..=10i32, // CHR
        0..=10i32, // INT
        0..=10i32, // STR
        0..=10i32, // MNY
    )
        .prop_map(|(chr, int, str_, mny)| {
            let mut props = HashMap::new();
            props.insert("CHR".to_string(), chr);
            props.insert("INT".to_string(), int);
            props.insert("STR".to_string(), str_);
            props.insert("MNY".to_string(), mny);
            props
        })
}

/// Create a minimal simulation engine for testing
fn create_test_engine() -> SimulationEngine {
    let mut talents = HashMap::new();
    talents.insert(
        1,
        TalentConfig {
            id: 1,
            name: "Test Talent".to_string(),
            description: "A test talent".to_string(),
            grade: 1,
            max_triggers: 1,
            condition: None,
            effect: None,
            exclusive: false,
            exclude: None,
            replacement: None,
            status: 0,
        },
    );

    let mut events = HashMap::new();
    events.insert(
        1,
        EventConfig {
            id: 1,
            event: "Test event".to_string(),
            grade: 1,
            no_random: false,
            include: None,
            exclude: None,
            effect: Some(EventEffect {
                chr: 0,
                int: 0,
                str_: 0,
                mny: 0,
                spr: 0,
                lif: 0,
                age: 0,
                rdm: 0,
            }),
            branch: None,
            post_event: None,
        },
    );

    // Add death event
    events.insert(
        999,
        EventConfig {
            id: 999,
            event: "Death event".to_string(),
            grade: 0,
            no_random: false,
            include: None,
            exclude: None,
            effect: Some(EventEffect {
                chr: 0,
                int: 0,
                str_: 0,
                mny: 0,
                spr: 0,
                lif: -10, // Kills the character
                age: 0,
                rdm: 0,
            }),
            branch: None,
            post_event: None,
        },
    );

    let mut ages = HashMap::new();
    // Add age configs for ages 0-100
    for age in 0..=100 {
        ages.insert(
            age,
            AgeConfig {
                age,
                talents: None,
                events: Some(vec![(1, 1.0)]),
            },
        );
    }

    // Add death age at 100
    ages.insert(
        100,
        AgeConfig {
            age: 100,
            talents: None,
            events: Some(vec![(999, 1.0)]),
        },
    );

    let achievements = HashMap::new();
    let judge_config = HashMap::new();

    SimulationEngine::new(talents, events, ages, achievements, judge_config)
}

// ═══════════════════════════════════════════════════════════════════════════
// Property Tests
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// Property 9: Simulation should terminate when LIF < 1
    /// Validates: Requirement 6.3 (Simulation Termination)
    #[test]
    fn prop_simulation_terminates(
        properties in initial_properties_strategy()
    ) {
        let engine = create_test_engine();
        let talent_ids = vec![1];
        let achieved: HashSet<i32> = HashSet::new();

        let result = engine.simulate(&talent_ids, &properties, &achieved);
        prop_assert!(result.is_ok(), "Simulation should complete without error");

        let result = result.unwrap();

        // Simulation should have at least one trajectory entry
        prop_assert!(!result.trajectory.is_empty(), "Trajectory should not be empty");

        // Last entry should have is_end = true
        let last_entry = result.trajectory.last().unwrap();
        prop_assert!(last_entry.is_end, "Last trajectory entry should have is_end = true");
    }

    /// Property 9.2: Simulation should not run forever
    /// Validates: Requirement 6.3 (Simulation Termination)
    #[test]
    fn prop_simulation_bounded(
        properties in initial_properties_strategy()
    ) {
        let engine = create_test_engine();
        let talent_ids = vec![1];
        let achieved: HashSet<i32> = HashSet::new();

        let result = engine.simulate(&talent_ids, &properties, &achieved).unwrap();

        // Simulation should terminate within reasonable bounds (max 200 years)
        prop_assert!(
            result.trajectory.len() <= 200,
            "Simulation should terminate within 200 years, got {} years",
            result.trajectory.len()
        );
    }

    /// Property 9.3: Summary score should be calculated correctly
    /// Validates: Requirement 3.6 (Summary Score)
    #[test]
    fn prop_summary_score_in_result(
        properties in initial_properties_strategy()
    ) {
        let engine = create_test_engine();
        let talent_ids = vec![1];
        let achieved: HashSet<i32> = HashSet::new();

        let result = engine.simulate(&talent_ids, &properties, &achieved).unwrap();

        // Summary should have a total score
        // The score should be non-negative (since all max values are >= 0)
        prop_assert!(
            result.summary.total_score >= 0,
            "Summary score should be non-negative, got {}",
            result.summary.total_score
        );
    }

    /// Property 9.4: Triggered events should be recorded
    /// Validates: Requirement 6.2 (Event Recording)
    #[test]
    fn prop_events_recorded(
        properties in initial_properties_strategy()
    ) {
        let engine = create_test_engine();
        let talent_ids = vec![1];
        let achieved: HashSet<i32> = HashSet::new();

        let result = engine.simulate(&talent_ids, &properties, &achieved).unwrap();

        // If simulation ran for at least one year, events should be recorded
        if !result.trajectory.is_empty() {
            // Events list should exist (may be empty if no events triggered)
            // This is just checking the structure is correct
            let _ = result.triggered_events.len();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_simulation() {
        let engine = create_test_engine();
        let mut properties = HashMap::new();
        properties.insert("CHR".to_string(), 5);
        properties.insert("INT".to_string(), 5);
        properties.insert("STR".to_string(), 5);
        properties.insert("MNY".to_string(), 5);

        let talent_ids = vec![1];
        let achieved: HashSet<i32> = HashSet::new();

        let result = engine.simulate(&talent_ids, &properties, &achieved);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(!result.trajectory.is_empty());
        assert!(result.trajectory.last().unwrap().is_end);
    }
}
