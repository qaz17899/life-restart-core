//! Achievement checking logic

use crate::condition::cache::check_condition;
use crate::config::{AchievementConfig, Opportunity};
use crate::property::PropertyState;
use std::collections::HashMap;

/// Achievement info for results
#[derive(Debug, Clone)]
pub struct AchievementInfo {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub grade: i32,
}

/// Check achievements for a given opportunity
pub fn check_achievements(
    opportunity: Opportunity,
    state: &PropertyState,
    achieved: &[Vec<i32>],
    achievements: &HashMap<i32, AchievementConfig>,
) -> Vec<AchievementInfo> {
    let mut new_achievements = Vec::new();

    for achievement in achievements.values() {
        // Check opportunity matches
        let ach_opportunity = Opportunity::from_str(&achievement.opportunity);
        if ach_opportunity != Some(opportunity) {
            continue;
        }

        // Check if already achieved
        if is_achieved(achievement.id, achieved) {
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

/// Check if an achievement is already achieved
fn is_achieved(achievement_id: i32, achieved: &[Vec<i32>]) -> bool {
    for group in achieved {
        if group.contains(&achievement_id) {
            return true;
        }
    }
    false
}

/// Unlock an achievement (add to achieved list)
pub fn unlock_achievement(achievement_id: i32, achieved: &[Vec<i32>]) -> Vec<Vec<i32>> {
    let mut new_achieved = achieved.to_vec();

    // Add to the first group or create a new group
    if new_achieved.is_empty() {
        new_achieved.push(vec![achievement_id]);
    } else {
        new_achieved[0].push(achievement_id);
    }

    new_achieved
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_achieved() {
        let achieved = vec![vec![1, 2, 3], vec![4, 5]];

        assert!(is_achieved(1, &achieved));
        assert!(is_achieved(5, &achieved));
        assert!(!is_achieved(6, &achieved));
    }

    #[test]
    fn test_unlock_achievement() {
        let achieved = vec![vec![1, 2]];
        let new_achieved = unlock_achievement(3, &achieved);

        assert_eq!(new_achieved, vec![vec![1, 2, 3]]);
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
        let achieved: Vec<Vec<i32>> = vec![];

        let new_achievements =
            check_achievements(Opportunity::Start, &state, &achieved, &achievements);

        assert_eq!(new_achievements.len(), 1);
        assert_eq!(new_achievements[0].id, 1);
    }
}
