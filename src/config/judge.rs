//! Judge configuration structures for property evaluation

use serde::Deserialize;

/// Judge level for property evaluation
#[derive(Debug, Clone, Deserialize)]
pub struct JudgeLevel {
    /// Minimum value for this level
    pub min: i32,
    /// Grade/tier of this level
    pub grade: i32,
    /// Display text for this level
    pub text: String,
}
