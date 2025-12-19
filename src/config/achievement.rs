//! Achievement configuration structures

use serde::Deserialize;

/// Achievement configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AchievementConfig {
    pub id: i32,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub grade: i32,
    /// Opportunity: "START", "TRAJECTORY", "SUMMARY"
    pub opportunity: String,
    pub condition: String,
}

/// Achievement opportunity timing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opportunity {
    Start,
    Trajectory,
    Summary,
}

impl Opportunity {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "START" => Some(Opportunity::Start),
            "TRAJECTORY" => Some(Opportunity::Trajectory),
            "SUMMARY" => Some(Opportunity::Summary),
            _ => None,
        }
    }
}
