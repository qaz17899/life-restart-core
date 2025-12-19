//! Age configuration structures

use serde::Deserialize;

/// Age configuration for each year
#[derive(Debug, Clone, Deserialize)]
pub struct AgeConfig {
    pub age: i32,
    /// Talents to add at this age
    pub talents: Option<Vec<i32>>,
    /// Event pool for this age: [(event_id, weight), ...]
    pub events: Option<Vec<(i32, f64)>>,
}
