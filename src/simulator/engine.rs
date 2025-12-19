//! Main simulation engine

use crate::achievement::{check_achievements, unlock_achievement, AchievementInfo};
use crate::config::{
    AchievementConfig, AgeConfig, EventConfig, EventEffect, JudgeLevel, Opportunity, TalentConfig,
};
use crate::error::Result;
use crate::event::{process_event, select_event};
use crate::property::PropertyState;
use crate::talent::{
    apply_replacements, apply_talent_effect, process_talents, ReplacementResult,
};
use std::collections::HashMap;

/// Content type constants
pub const CONTENT_TYPE_TALENT: &str = "TLT";
pub const CONTENT_TYPE_EVENT: &str = "EVT";

/// Year content entry
#[derive(Debug, Clone)]
pub struct YearContent {
    pub content_type: String,
    pub description: String,
    pub grade: i32,
    pub name: Option<String>,
}

/// Trajectory entry for a single year
#[derive(Debug, Clone)]
pub struct TrajectoryEntry {
    pub age: i32,
    pub content: Vec<YearContent>,
    pub is_end: bool,
    pub properties: HashMap<String, i32>,
}

/// Property judge result
#[derive(Debug, Clone)]
pub struct PropertyJudge {
    pub property_type: String,
    pub value: i32,
    pub grade: i32,
    pub text: String,
    pub progress: f64,
}

/// Talent info for summary
#[derive(Debug, Clone)]
pub struct TalentInfo {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub grade: i32,
}

/// Summary result
#[derive(Debug, Clone)]
pub struct SummaryResult {
    pub total_score: i32,
    pub judges: Vec<PropertyJudge>,
    pub talents: Vec<TalentInfo>,
}

/// Complete simulation result
#[derive(Debug, Clone)]
pub struct SimulationResult {
    pub trajectory: Vec<TrajectoryEntry>,
    pub summary: SummaryResult,
    pub new_achievements: Vec<AchievementInfo>,
    pub triggered_events: Vec<i32>,
    pub replacements: Vec<ReplacementResult>,
}

/// Main simulation engine
pub struct SimulationEngine {
    talents: HashMap<i32, TalentConfig>,
    events: HashMap<i32, EventConfig>,
    ages: HashMap<i32, AgeConfig>,
    achievements: HashMap<i32, AchievementConfig>,
    judge_config: HashMap<String, Vec<JudgeLevel>>,
}

impl SimulationEngine {
    pub fn new(
        talents: HashMap<i32, TalentConfig>,
        events: HashMap<i32, EventConfig>,
        ages: HashMap<i32, AgeConfig>,
        achievements: HashMap<i32, AchievementConfig>,
        judge_config: HashMap<String, Vec<JudgeLevel>>,
    ) -> Self {
        Self {
            talents,
            events,
            ages,
            achievements,
            judge_config,
        }
    }

    /// Run the complete life simulation
    pub fn simulate(
        &self,
        talent_ids: &[i32],
        properties: &HashMap<String, i32>,
        achieved_list: &[Vec<i32>],
    ) -> Result<SimulationResult> {
        // Apply talent replacements
        let (final_talents, replacements) = apply_replacements(talent_ids, &self.talents);

        // Create initial state
        let mut state = PropertyState::new(
            *properties.get("CHR").unwrap_or(&0),
            *properties.get("INT").unwrap_or(&0),
            *properties.get("STR").unwrap_or(&0),
            *properties.get("MNY").unwrap_or(&0),
            5, // Default SPR
            1, // Default LIF
        );

        // Set talents
        for talent_id in &final_talents {
            state.tlt.push(*talent_id);
        }

        // Talent trigger counts
        let mut trigger_counts: HashMap<i32, i32> = HashMap::new();

        // Apply initial talent effects
        self.do_talents(&mut state, &mut trigger_counts);

        // Track achievements
        let mut all_new_achievements: Vec<AchievementInfo> = Vec::new();
        let mut current_achieved = achieved_list.to_vec();

        // Check START achievements
        let start_achievements = check_achievements(
            Opportunity::Start,
            &state,
            &current_achieved,
            &self.achievements,
        );
        for achievement in start_achievements {
            current_achieved = unlock_achievement(achievement.id, &current_achieved);
            all_new_achievements.push(achievement);
        }

        // Simulate life trajectory
        let mut trajectory: Vec<TrajectoryEntry> = Vec::new();

        while !state.is_end() {
            let year_result = self.simulate_year(&mut state, &mut trigger_counts);
            trajectory.push(year_result.clone());

            // Check TRAJECTORY achievements
            let traj_achievements = check_achievements(
                Opportunity::Trajectory,
                &state,
                &current_achieved,
                &self.achievements,
            );
            for achievement in traj_achievements {
                current_achieved = unlock_achievement(achievement.id, &current_achieved);
                all_new_achievements.push(achievement);
            }

            if year_result.is_end {
                break;
            }
        }

        // Check SUMMARY achievements
        let summary_achievements = check_achievements(
            Opportunity::Summary,
            &state,
            &current_achieved,
            &self.achievements,
        );
        for achievement in summary_achievements {
            all_new_achievements.push(achievement);
        }

        // Build summary
        let judges = self.get_summary_judges(&state);
        let total_score = state.calculate_summary_score();

        let talent_infos: Vec<TalentInfo> = final_talents
            .iter()
            .filter_map(|id| {
                self.talents.get(id).map(|t| TalentInfo {
                    id: t.id,
                    name: t.name.clone(),
                    description: t.description.clone(),
                    grade: t.grade,
                })
            })
            .collect();

        let summary = SummaryResult {
            total_score,
            judges,
            talents: talent_infos,
        };

        Ok(SimulationResult {
            trajectory,
            summary,
            new_achievements: all_new_achievements,
            triggered_events: state.evt.clone(),
            replacements,
        })
    }

    fn simulate_year(
        &self,
        state: &mut PropertyState,
        trigger_counts: &mut HashMap<i32, i32>,
    ) -> TrajectoryEntry {
        // Advance age
        state.change("AGE", 1);
        let age = state.age;

        let mut content: Vec<YearContent> = Vec::new();

        // Get age config
        if let Some(age_config) = self.ages.get(&age) {
            // Add age-specific talents
            if let Some(ref talents) = age_config.talents {
                for talent_id in talents {
                    if !state.tlt.contains(talent_id) {
                        state.tlt.push(*talent_id);
                    }
                }
            }

            // Process talents
            let talent_content = self.do_talents(state, trigger_counts);
            content.extend(talent_content);

            // Process events
            if let Some(ref events) = age_config.events {
                let event_content = self.do_events(state, events);
                content.extend(event_content);
            }
        } else {
            // No age config, just process talents
            let talent_content = self.do_talents(state, trigger_counts);
            content.extend(talent_content);
        }

        let is_end = state.is_end();

        TrajectoryEntry {
            age,
            content,
            is_end,
            properties: state.get_properties_dict(),
        }
    }

    fn do_talents(
        &self,
        state: &mut PropertyState,
        trigger_counts: &mut HashMap<i32, i32>,
    ) -> Vec<YearContent> {
        let results = process_talents(state, &self.talents, trigger_counts);
        let mut content = Vec::new();

        for result in results {
            content.push(YearContent {
                content_type: CONTENT_TYPE_TALENT.to_string(),
                description: result.description,
                grade: result.grade,
                name: Some(result.name),
            });

            if let Some(ref effect) = result.effect {
                apply_talent_effect(state, effect);
            }
        }

        content
    }

    fn do_events(&self, state: &mut PropertyState, event_pool: &[(i32, f64)]) -> Vec<YearContent> {
        let mut content = Vec::new();

        if let Some(event_id) = select_event(event_pool, &self.events, state) {
            self.process_event_chain(state, event_id, &mut content);
        }

        content
    }

    fn process_event_chain(
        &self,
        state: &mut PropertyState,
        event_id: i32,
        content: &mut Vec<YearContent>,
    ) {
        if let Some(result) = process_event(event_id, &self.events, state) {
            // Record event
            if !state.evt.contains(&event_id) {
                state.evt.push(event_id);
            }

            // Build description
            let mut description = result.description;
            if let Some(ref post) = result.post_event {
                description = format!("{}{}", description, post);
            }

            content.push(YearContent {
                content_type: CONTENT_TYPE_EVENT.to_string(),
                description,
                grade: result.grade,
                name: None,
            });

            // Apply effect
            if let Some(ref effect) = result.effect {
                apply_event_effect(state, effect);
            }

            // Process chain
            if let Some(next_id) = result.next_event_id {
                self.process_event_chain(state, next_id, content);
            }
        }
    }

    fn get_summary_judges(&self, state: &PropertyState) -> Vec<PropertyJudge> {
        let mut judges = Vec::new();

        let props = [
            ("HCHR", state.hchr.max(state.chr)),
            ("HINT", state.hint.max(state.int)),
            ("HSTR", state.hstr.max(state.str_)),
            ("HMNY", state.hmny.max(state.mny)),
            ("HSPR", state.hspr.max(state.spr)),
            ("HAGE", state.hage.max(state.age)),
        ];

        for (prop, value) in props {
            if let Some(judge) = self.judge_property(prop, value) {
                judges.push(judge);
            }
        }

        // Total score
        let sum_value = state.calculate_summary_score();
        if let Some(judge) = self.judge_property("SUM", sum_value) {
            judges.push(judge);
        }

        judges
    }

    fn judge_property(&self, prop: &str, value: i32) -> Option<PropertyJudge> {
        let levels = self.judge_config.get(prop)?;

        // Find the matching level (levels should be sorted by min descending)
        for level in levels {
            if value >= level.min {
                let progress = (value.min(10).max(0) as f64) / 10.0;
                return Some(PropertyJudge {
                    property_type: prop.to_string(),
                    value,
                    grade: level.grade,
                    text: level.text.clone(),
                    progress,
                });
            }
        }

        None
    }
}

fn apply_event_effect(state: &mut PropertyState, effect: &EventEffect) {
    if effect.chr != 0 {
        state.change("CHR", effect.chr);
    }
    if effect.int != 0 {
        state.change("INT", effect.int);
    }
    if effect.str_ != 0 {
        state.change("STR", effect.str_);
    }
    if effect.mny != 0 {
        state.change("MNY", effect.mny);
    }
    if effect.spr != 0 {
        state.change("SPR", effect.spr);
    }
    if effect.lif != 0 {
        state.change("LIF", effect.lif);
    }
    if effect.age != 0 {
        state.change("AGE", effect.age);
    }
    if effect.rdm != 0 {
        state.change("RDM", effect.rdm);
    }
}
