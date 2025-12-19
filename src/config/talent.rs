//! Talent configuration structures

use serde::Deserialize;
use std::collections::HashMap;

/// Talent configuration
#[derive(Debug, Clone, Deserialize)]
pub struct TalentConfig {
    pub id: i32,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub grade: i32,
    #[serde(default = "default_max_triggers")]
    pub max_triggers: i32,
    pub condition: Option<String>,
    pub effect: Option<TalentEffect>,
    #[serde(default)]
    pub exclusive: bool,
    pub exclude: Option<Vec<i32>>,
    pub replacement: Option<TalentReplacement>,
    #[serde(default)]
    pub status: i32,
}

fn default_max_triggers() -> i32 {
    1
}

/// Talent effect on properties
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TalentEffect {
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

/// Talent replacement rules
#[derive(Debug, Clone, Deserialize)]
pub struct TalentReplacement {
    /// Replace by grade: {"0": 1.0, "1": 2.0, ...}
    pub grade: Option<HashMap<String, f64>>,
    /// Replace by specific talent: {"1001": 1.0, "1002": 2.0, ...}
    pub talent: Option<HashMap<String, f64>>,
}
