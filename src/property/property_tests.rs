//! Property tests for property module
//!
//! Feature: life-restart-rust
//! Property 2: PropertyState Min/Max Tracking
//! Property 3: Summary Score Calculation
//! Validates: Requirements 3.2, 3.3, 3.5, 3.6

use proptest::prelude::*;

use crate::property::PropertyState;

// ═══════════════════════════════════════════════════════════════════════════
// Strategy generators for property tests
// ═══════════════════════════════════════════════════════════════════════════

/// Generate initial property values
fn initial_values_strategy() -> impl Strategy<Value = (i32, i32, i32, i32, i32, i32)> {
    (
        -10..=20i32, // chr
        -10..=20i32, // int
        -10..=20i32, // str
        -10..=20i32, // mny
        -10..=20i32, // spr
        1..=10i32,   // lif
    )
}

/// Generate a sequence of property changes
fn change_sequence_strategy() -> impl Strategy<Value = Vec<(&'static str, i32)>> {
    prop::collection::vec(
        (
            prop_oneof![
                Just("CHR"),
                Just("INT"),
                Just("STR"),
                Just("MNY"),
                Just("SPR"),
                Just("AGE"),
            ],
            -20..=20i32,
        ),
        1..=20,
    )
}

// ═══════════════════════════════════════════════════════════════════════════
// Property Tests
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    /// Property 2.1: Min tracking should always be <= current value
    /// Validates: Requirement 3.2 (Min Value Tracking)
    #[test]
    fn prop_min_tracking_invariant(
        initial in initial_values_strategy(),
        changes in change_sequence_strategy()
    ) {
        let (chr, int, str_, mny, spr, lif) = initial;
        let mut state = PropertyState::new(chr, int, str_, mny, spr, lif);

        for (prop, delta) in changes {
            state.change(prop, delta);

            // Verify min tracking invariant
            prop_assert!(state.lchr <= state.chr, "LCHR {} > CHR {}", state.lchr, state.chr);
            prop_assert!(state.lint <= state.int, "LINT {} > INT {}", state.lint, state.int);
            prop_assert!(state.lstr <= state.str_, "LSTR {} > STR {}", state.lstr, state.str_);
            prop_assert!(state.lmny <= state.mny, "LMNY {} > MNY {}", state.lmny, state.mny);
            prop_assert!(state.lspr <= state.spr, "LSPR {} > SPR {}", state.lspr, state.spr);
            prop_assert!(state.lage <= state.age, "LAGE {} > AGE {}", state.lage, state.age);
        }
    }

    /// Property 2.2: Max tracking should always be >= current value
    /// Validates: Requirement 3.3 (Max Value Tracking)
    #[test]
    fn prop_max_tracking_invariant(
        initial in initial_values_strategy(),
        changes in change_sequence_strategy()
    ) {
        let (chr, int, str_, mny, spr, lif) = initial;
        let mut state = PropertyState::new(chr, int, str_, mny, spr, lif);

        for (prop, delta) in changes {
            state.change(prop, delta);

            // Verify max tracking invariant
            prop_assert!(state.hchr >= state.chr, "HCHR {} < CHR {}", state.hchr, state.chr);
            prop_assert!(state.hint >= state.int, "HINT {} < INT {}", state.hint, state.int);
            prop_assert!(state.hstr >= state.str_, "HSTR {} < STR {}", state.hstr, state.str_);
            prop_assert!(state.hmny >= state.mny, "HMNY {} < MNY {}", state.hmny, state.mny);
            prop_assert!(state.hspr >= state.spr, "HSPR {} < SPR {}", state.hspr, state.spr);
            prop_assert!(state.hage >= state.age, "HAGE {} < AGE {}", state.hage, state.age);
        }
    }

    /// Property 2.3: Min should be the actual minimum seen
    /// Validates: Requirement 3.5 (Min/Max Accuracy)
    #[test]
    fn prop_min_is_actual_minimum(
        initial in initial_values_strategy(),
        changes in change_sequence_strategy()
    ) {
        let (chr, int, str_, mny, spr, lif) = initial;
        let mut state = PropertyState::new(chr, int, str_, mny, spr, lif);

        // Track actual minimums
        let mut actual_min_chr = chr;
        let mut actual_min_int = int;
        let mut actual_min_str = str_;
        let mut actual_min_mny = mny;
        let mut actual_min_spr = spr;

        for (prop, delta) in changes {
            state.change(prop, delta);

            // Update actual minimums
            actual_min_chr = actual_min_chr.min(state.chr);
            actual_min_int = actual_min_int.min(state.int);
            actual_min_str = actual_min_str.min(state.str_);
            actual_min_mny = actual_min_mny.min(state.mny);
            actual_min_spr = actual_min_spr.min(state.spr);
        }

        // Verify tracked minimums match actual minimums
        prop_assert_eq!(state.lchr, actual_min_chr, "LCHR mismatch");
        prop_assert_eq!(state.lint, actual_min_int, "LINT mismatch");
        prop_assert_eq!(state.lstr, actual_min_str, "LSTR mismatch");
        prop_assert_eq!(state.lmny, actual_min_mny, "LMNY mismatch");
        prop_assert_eq!(state.lspr, actual_min_spr, "LSPR mismatch");
    }

    /// Property 2.4: Max should be the actual maximum seen
    /// Validates: Requirement 3.5 (Min/Max Accuracy)
    #[test]
    fn prop_max_is_actual_maximum(
        initial in initial_values_strategy(),
        changes in change_sequence_strategy()
    ) {
        let (chr, int, str_, mny, spr, lif) = initial;
        let mut state = PropertyState::new(chr, int, str_, mny, spr, lif);

        // Track actual maximums
        let mut actual_max_chr = chr;
        let mut actual_max_int = int;
        let mut actual_max_str = str_;
        let mut actual_max_mny = mny;
        let mut actual_max_spr = spr;

        for (prop, delta) in changes {
            state.change(prop, delta);

            // Update actual maximums
            actual_max_chr = actual_max_chr.max(state.chr);
            actual_max_int = actual_max_int.max(state.int);
            actual_max_str = actual_max_str.max(state.str_);
            actual_max_mny = actual_max_mny.max(state.mny);
            actual_max_spr = actual_max_spr.max(state.spr);
        }

        // Verify tracked maximums match actual maximums
        prop_assert_eq!(state.hchr, actual_max_chr, "HCHR mismatch");
        prop_assert_eq!(state.hint, actual_max_int, "HINT mismatch");
        prop_assert_eq!(state.hstr, actual_max_str, "HSTR mismatch");
        prop_assert_eq!(state.hmny, actual_max_mny, "HMNY mismatch");
        prop_assert_eq!(state.hspr, actual_max_spr, "HSPR mismatch");
    }

    /// Property 3: Summary score calculation is correct
    /// Formula: (HCHR + HINT + HSTR + HMNY + HSPR) * 2 + HAGE / 2
    /// Validates: Requirement 3.6 (Summary Score)
    #[test]
    fn prop_summary_score_calculation(
        initial in initial_values_strategy(),
        changes in change_sequence_strategy(),
        age_changes in prop::collection::vec(-5..=10i32, 0..=10)
    ) {
        let (chr, int, str_, mny, spr, lif) = initial;
        let mut state = PropertyState::new(chr, int, str_, mny, spr, lif);

        // Apply property changes
        for (prop, delta) in changes {
            state.change(prop, delta);
        }

        // Apply age changes
        for delta in age_changes {
            state.change("AGE", delta);
        }

        // Calculate expected score
        let hchr = state.hchr.max(state.chr);
        let hint = state.hint.max(state.int);
        let hstr = state.hstr.max(state.str_);
        let hmny = state.hmny.max(state.mny);
        let hspr = state.hspr.max(state.spr);
        let hage = state.hage.max(state.age);

        let expected = (hchr + hint + hstr + hmny + hspr) * 2 + hage / 2;
        let actual = state.calculate_summary_score();

        prop_assert_eq!(actual, expected, "Summary score mismatch");
    }

    /// Property 4: is_end should return true when LIF < 1
    /// Validates: Requirement 3.4 (Game End Condition)
    #[test]
    fn prop_is_end_condition(
        initial in initial_values_strategy(),
        lif_changes in prop::collection::vec(-5..=5i32, 1..=10)
    ) {
        let (chr, int, str_, mny, spr, lif) = initial;
        let mut state = PropertyState::new(chr, int, str_, mny, spr, lif);

        for delta in lif_changes {
            state.change("LIF", delta);
            let expected_end = state.lif < 1;
            prop_assert_eq!(state.is_end(), expected_end, "is_end mismatch: LIF={}", state.lif);
        }
    }

    /// Property 5: TLT list should not contain duplicates
    /// Validates: Requirement 3.1 (List Properties)
    #[test]
    fn prop_tlt_no_duplicates(
        talent_ids in prop::collection::vec(1..=10000i32, 1..=20)
    ) {
        let mut state = PropertyState::default();

        for id in &talent_ids {
            state.change("TLT", *id);
        }

        // Check no duplicates
        let mut seen = std::collections::HashSet::new();
        for id in &state.tlt {
            prop_assert!(!seen.contains(id), "Duplicate talent ID: {}", id);
            seen.insert(*id);
        }
    }

    /// Property 6: EVT list should not contain duplicates
    /// Validates: Requirement 3.1 (List Properties)
    #[test]
    fn prop_evt_no_duplicates(
        event_ids in prop::collection::vec(1..=100000i32, 1..=50)
    ) {
        let mut state = PropertyState::default();

        for id in &event_ids {
            state.change("EVT", *id);
        }

        // Check no duplicates
        let mut seen = std::collections::HashSet::new();
        for id in &state.evt {
            prop_assert!(!seen.contains(id), "Duplicate event ID: {}", id);
            seen.insert(*id);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_property_tests_compile() {
        assert!(true);
    }
}
