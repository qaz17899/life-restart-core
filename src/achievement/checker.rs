//! Achievement checking logic

use crate::condition::cache::check_condition;
use crate::config::{AchievementConfig, Opportunity};
use crate::property::PropertyState;
use std::collections::{HashMap, HashSet};

/// Achievement info for results
#[derive(Debug, Clone)]
pub struct AchievementInfo {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub grade: i32,
}

/// Check achievements for a given opportunity
#[inline]
pub fn check_achievements(
    opportunity: Opportunity,
    state: &PropertyState,
    achieved: &HashSet<i32>,
    achievements: &HashMap<i32, AchievementConfig>,
) -> Vec<AchievementInfo> {
    let mut new_achievements = Vec::with_capacity(4);

    for achievement in achievements.values() {
        // Check opportunity matches
        let ach_opportunity = Opportunity::from_str(&achievement.opportunity);
        if ach_opportunity != Some(opportunity) {
            continue;
        }

        // Check if already achieved (O(1) lookup with HashSet)
        if achieved.contains(&achievement.id) {
            continue;
        }

        // Check condition
        if check_condition(&achievement.condition, state).unwrap_or(false) {
            new_achievements.push(AchievementInfo {
                id: achievement.id,
                name: achievement.name.clone(),
                description: achievement.description.clone(),
                grade: achievement.grade,
            });
        }
    }

    new_achievements
}

/// Unlock an achievement (add to achieved set)
#[inline]
pub fn unlock_achievement(achievement_id: i32, achieved: &mut HashSet<i32>) {
    achieved.insert(achievement_id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_achieved() {
        let achieved: HashSet<i32> = [1, 2, 3, 4, 5].into_iter().collect();

        assert!(achieved.contains(&1));
        assert!(achieved.contains(&5));
        assert!(!achieved.contains(&6));
    }

    #[test]
    fn test_unlock_achievement() {
        let mut achieved: HashSet<i32> = [1, 2].into_iter().collect();
        unlock_achievement(3, &mut achieved);

        assert!(achieved.contains(&3));
        assert_eq!(achieved.len(), 3);
    }

    #[test]
    fn test_check_achievements() {
        let mut achievements = HashMap::new();
        achievements.insert(
            1,
            AchievementConfig {
                id: 1,
                name: "Test".to_string(),
                description: "Test achievement".to_string(),
                grade: 1,
                opportunity: "START".to_string(),
                condition: "CHR>5".to_string(),
            },
        );

        let state = PropertyState {
            chr: 10,
            ..Default::default()
        };
        let achieved: HashSet<i32> = HashSet::new();

        let new_achievements =
            check_achievements(Opportunity::Start, &state, &achieved, &achievements);

        assert_eq!(new_achievements.len(), 1);
        assert_eq!(new_achievements[0].id, 1);
    }
}
