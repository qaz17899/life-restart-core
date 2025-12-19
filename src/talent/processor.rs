//! Talent processing logic - Optimized version

use crate::condition::cache::check_condition;
use crate::config::{TalentConfig, TalentEffect};
use crate::property::PropertyState;
use std::collections::HashMap;

/// Result of a talent trigger
#[derive(Debug, Clone)]
pub struct TalentTriggerResult {
    pub talent_id: i32,
    pub name: String,
    pub description: String,
    pub grade: i32,
    pub effect: Option<TalentEffect>,
}

/// Process talents for the current state - optimized version
#[inline]
pub fn process_talents(
    state: &PropertyState,
    talents: &HashMap<i32, TalentConfig>,
    trigger_counts: &mut HashMap<i32, i32>,
) -> Vec<TalentTriggerResult> {
    // Pre-allocate with expected capacity
    let mut results = Vec::with_capacity(state.tlt.len());

    for talent_id in &state.tlt {
        if let Some(talent) = talents.get(talent_id) {
            // Check trigger count limit
            let current_count = trigger_counts.get(talent_id).copied().unwrap_or(0);
            if current_count >= talent.max_triggers {
                continue;
            }

            // Check condition
            if let Some(ref condition) = talent.condition {
                if !check_condition(condition, state).unwrap_or(false) {
                    continue;
                }
            }

            // Trigger talent
            *trigger_counts.entry(*talent_id).or_insert(0) += 1;

            results.push(TalentTriggerResult {
                talent_id: *talent_id,
                name: talent.name.clone(),
                description: talent.description.clone(),
                grade: talent.grade,
                effect: talent.effect.clone(),
            });
        }
    }

    results
}

/// Apply talent effect to property state - optimized with direct field access
#[inline]
pub fn apply_talent_effect(state: &mut PropertyState, effect: &TalentEffect) {
    // Direct field access is faster than string matching
    if effect.chr != 0 {
        state.chr += effect.chr;
        state.lchr = state.lchr.min(state.chr);
        state.hchr = state.hchr.max(state.chr);
    }
    if effect.int != 0 {
        state.int += effect.int;
        state.lint = state.lint.min(state.int);
        state.hint = state.hint.max(state.int);
    }
    if effect.str_ != 0 {
        state.str_ += effect.str_;
        state.lstr = state.lstr.min(state.str_);
        state.hstr = state.hstr.max(state.str_);
    }
    if effect.mny != 0 {
        state.mny += effect.mny;
        state.lmny = state.lmny.min(state.mny);
        state.hmny = state.hmny.max(state.mny);
    }
    if effect.spr != 0 {
        state.spr += effect.spr;
        state.lspr = state.lspr.min(state.spr);
        state.hspr = state.hspr.max(state.spr);
    }
    if effect.lif != 0 {
        state.lif += effect.lif;
    }
    if effect.age != 0 {
        state.age += effect.age;
        state.lage = state.lage.min(state.age);
        state.hage = state.hage.max(state.age);
    }
    if effect.rdm != 0 {
        state.change("RDM", effect.rdm);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_talents_trigger_limit() {
        let mut talents = HashMap::new();
        talents.insert(
            1,
            TalentConfig {
                id: 1,
                name: "Test".to_string(),
                description: "Test talent".to_string(),
                grade: 1,
                max_triggers: 2,
                condition: None,
                effect: None,
                exclusive: false,
                exclude: None,
                replacement: None,
                status: 0,
            },
        );

        let state = PropertyState {
            tlt: vec![1],
            ..Default::default()
        };

        let mut trigger_counts = HashMap::new();

        // First trigger
        let results = process_talents(&state, &talents, &mut trigger_counts);
        assert_eq!(results.len(), 1);

        // Second trigger
        let results = process_talents(&state, &talents, &mut trigger_counts);
        assert_eq!(results.len(), 1);

        // Third trigger - should not trigger (max_triggers = 2)
        let results = process_talents(&state, &talents, &mut trigger_counts);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_apply_talent_effect() {
        let mut state = PropertyState::new(5, 5, 5, 5, 5, 1);
        let effect = TalentEffect {
            chr: 2,
            int: -1,
            ..Default::default()
        };

        apply_talent_effect(&mut state, &effect);

        assert_eq!(state.chr, 7);
        assert_eq!(state.int, 4);
    }
}
