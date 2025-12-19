//! Event configuration structures

use serde::Deserialize;

/// Event configuration
#[derive(Debug, Clone, Deserialize)]
pub struct EventConfig {
    pub id: i32,
    pub event: String,
    #[serde(default)]
    pub grade: i32,
    #[serde(default)]
    pub no_random: bool,
    pub include: Option<String>,
    pub exclude: Option<String>,
    pub effect: Option<EventEffect>,
    pub branch: Option<Vec<EventBranch>>,
    pub post_event: Option<String>,
}

/// Event effect on properties
#[derive(Debug, Clone, Default, Deserialize)]
pub struct EventEffect {
    #[serde(default, rename = "CHR")]
    pub chr: i32,
    #[serde(default, rename = "INT")]
    pub int: i32,
    #[serde(default, rename = "STR")]
    pub str_: i32,
    #[serde(default, rename = "MNY")]
    pub mny: i32,
    #[serde(default, rename = "SPR")]
    pub spr: i32,
    #[serde(default, rename = "LIF")]
    pub lif: i32,
    #[serde(default, rename = "AGE")]
    pub age: i32,
    #[serde(default, rename = "RDM")]
    pub rdm: i32,
}

/// Event branch for conditional branching
#[derive(Debug, Clone, Deserialize)]
pub struct EventBranch {
    pub condition: String,
    pub event_id: i32,
}
