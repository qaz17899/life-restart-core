//! Talent replacement logic

use crate::config::TalentConfig;
use crate::event::selector::weighted_random;
use std::collections::HashMap;

/// Result of a talent replacement
#[derive(Debug, Clone)]
pub struct ReplacementResult {
    pub source_id: i32,
    pub source_name: String,
    pub target_id: i32,
    pub target_name: String,
}

/// Apply talent replacements
pub fn apply_replacements(
    talent_ids: &[i32],
    talents: &HashMap<i32, TalentConfig>,
) -> (Vec<i32>, Vec<ReplacementResult>) {
    let mut new_talents = talent_ids.to_vec();
    let mut replacements = Vec::new();

    for (i, &talent_id) in talent_ids.iter().enumerate() {
        let replaced_id = replace_talent(talent_id, &new_talents, talents);
        if replaced_id != talent_id {
            if let (Some(source), Some(target)) = (talents.get(&talent_id), talents.get(&replaced_id))
            {
                replacements.push(ReplacementResult {
                    source_id: talent_id,
                    source_name: source.name.clone(),
                    target_id: replaced_id,
                    target_name: target.name.clone(),
                });
            }
            new_talents[i] = replaced_id;
            new_talents.push(replaced_id);
        }
    }

    // Return only the original count of talents
    new_talents.truncate(talent_ids.len());
    (new_talents, replacements)
}

/// Replace a single talent recursively
fn replace_talent(
    talent_id: i32,
    existing_talents: &[i32],
    talents: &HashMap<i32, TalentConfig>,
) -> i32 {
    let talent = match talents.get(&talent_id) {
        Some(t) => t,
        None => return talent_id,
    };

    let replacement = match &talent.replacement {
        Some(r) => r,
        None => return talent_id,
    };

    let mut replace_list: Vec<(i32, f64)> = Vec::new();

    // Replace by grade
    if let Some(ref grade_map) = replacement.grade {
        for t in talents.values() {
            if t.exclusive {
                continue;
            }
            let grade_key = t.grade.to_string();
            if let Some(&weight) = grade_map.get(&grade_key) {
                if check_exclusion(existing_talents, t.id, talents).is_none() {
                    replace_list.push((t.id, weight));
                }
            }
        }
    }

    // Replace by specific talent
    if let Some(ref talent_map) = replacement.talent {
        for (tid_str, &weight) in talent_map {
            if let Ok(tid) = tid_str.parse::<i32>() {
                if check_exclusion(existing_talents, tid, talents).is_none() {
                    replace_list.push((tid, weight));
                }
            }
        }
    }

    if replace_list.is_empty() {
        return talent_id;
    }

    // Weighted random selection
    let replaced_id = weighted_random(&replace_list).unwrap_or(talent_id);

    // Recursive replacement
    let mut new_existing = existing_talents.to_vec();
    new_existing.push(replaced_id);
    replace_talent(replaced_id, &new_existing, talents)
}

/// Check talent exclusion (bidirectional)
pub fn check_exclusion(
    talents: &[i32],
    exclude_id: i32,
    talent_configs: &HashMap<i32, TalentConfig>,
) -> Option<i32> {
    let exclude_talent = talent_configs.get(&exclude_id)?;

    for &talent_id in talents {
        // Check exclude_id's exclude list
        if let Some(ref exclude_list) = exclude_talent.exclude {
            if exclude_list.contains(&talent_id) {
                return Some(talent_id);
            }
        }

        // Check talent_id's exclude list (bidirectional)
        if let Some(talent) = talent_configs.get(&talent_id) {
            if let Some(ref exclude_list) = talent.exclude {
                if exclude_list.contains(&exclude_id) {
                    return Some(talent_id);
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_exclusion_bidirectional() {
        let mut talents = HashMap::new();
        talents.insert(
            1,
            TalentConfig {
                id: 1,
                name: "A".to_string(),
                description: "".to_string(),
                grade: 0,
                max_triggers: 1,
                condition: None,
                effect: None,
                exclusive: false,
                exclude: Some(vec![2]),
                replacement: None,
                status: 0,
            },
        );
        talents.insert(
            2,
            TalentConfig {
                id: 2,
                name: "B".to_string(),
                description: "".to_string(),
                grade: 0,
                max_triggers: 1,
                condition: None,
                effect: None,
                exclusive: false,
                exclude: None,
                replacement: None,
                status: 0,
            },
        );
        talents.insert(
            3,
            TalentConfig {
                id: 3,
                name: "C".to_string(),
                description: "".to_string(),
                grade: 0,
                max_triggers: 1,
                condition: None,
                effect: None,
                exclusive: false,
                exclude: None,
                replacement: None,
                status: 0,
            },
        );

        // A excludes B: checking if B can be added when A is present
        // A's exclude list contains B, so return Some(1)
        assert_eq!(check_exclusion(&[1], 2, &talents), Some(1));

        // Checking if A can be added when B is present
        // A's exclude list contains B (which is in talents), so return Some(2)
        assert_eq!(check_exclusion(&[2], 1, &talents), Some(2));

        // Bidirectional: B doesn't exclude A, but when checking if A can be added
        // when B is present, we also check A's exclude list which contains B
        // This is the bidirectional check

        // C has no exclusions, so it can be added with anyone
        assert_eq!(check_exclusion(&[1], 3, &talents), None);
        assert_eq!(check_exclusion(&[2], 3, &talents), None);
        assert_eq!(check_exclusion(&[3], 1, &talents), None);
        assert_eq!(check_exclusion(&[3], 2, &talents), None);
    }

    #[test]
    fn test_no_replacement() {
        let mut talents = HashMap::new();
        talents.insert(
            1,
            TalentConfig {
                id: 1,
                name: "A".to_string(),
                description: "".to_string(),
                grade: 0,
                max_triggers: 1,
                condition: None,
                effect: None,
                exclusive: false,
                exclude: None,
                replacement: None,
                status: 0,
            },
        );

        let (new_talents, replacements) = apply_replacements(&[1], &talents);
        assert_eq!(new_talents, vec![1]);
        assert!(replacements.is_empty());
    }
}
