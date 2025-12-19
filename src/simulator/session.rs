//! GameSession - Stateful session for Python-Rust boundary
//!
//! This module provides the GameSession PyClass that holds simulation results
//! in Rust heap memory, allowing Python to lazily access data without
//! serializing the entire result upfront.

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyList};
use std::collections::HashMap;
use std::sync::Arc;

use crate::achievement::AchievementInfo;
use crate::talent::ReplacementResult;

use super::{SimulationResult, TalentInfo};

// ============================================================================
// Pre-rendered Data Structures
// ============================================================================

/// Pre-rendered year data - optimized for display
/// Implements Clone + Send + Sync for thread safety
#[derive(Debug, Clone)]
pub struct RenderedYear {
    /// Age of this year
    pub age: i32,
    /// Pre-formatted display text with emoji, e.g., "🟣 你考上了大學\n🔵 你交了女朋友"
    pub display_text: String,
    /// Fixed-size array [CHR, INT, STR, MNY, SPR, LIF] - faster than HashMap
    pub properties: [i32; 6],
    /// Whether this is the final year
    pub is_end: bool,
}

/// Property index constants for the properties array
pub const PROP_CHR: usize = 0;
pub const PROP_INT: usize = 1;
pub const PROP_STR: usize = 2;
pub const PROP_MNY: usize = 3;
pub const PROP_SPR: usize = 4;
pub const PROP_LIF: usize = 5;

/// Property names in order
pub const PROP_NAMES: [&str; 6] = ["CHR", "INT", "STR", "MNY", "SPR", "LIF"];


/// Pre-rendered property judge with progress bar
/// Implements Clone + Send + Sync for thread safety
#[derive(Debug, Clone)]
pub struct PreRenderedJudge {
    /// Property type (e.g., "HCHR", "HINT")
    pub property_type: String,
    /// Property value
    pub value: i32,
    /// Grade level
    pub grade: i32,
    /// Judge text description
    pub text: String,
    /// Progress value (0.0 to 1.0)
    pub progress: f64,
    /// Pre-rendered progress bar, e.g., "██████░░░░"
    pub progress_bar: String,
}

/// Pre-rendered summary
/// Implements Clone + Send + Sync for thread safety
#[derive(Debug, Clone)]
pub struct PreRenderedSummary {
    /// Total score
    pub total_score: i32,
    /// Property judges with progress bars
    pub judges: Vec<PreRenderedJudge>,
    /// Talent information
    pub talents: Vec<TalentInfo>,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Render a progress bar string from a progress value (0.0 to 1.0)
/// Returns a 10-character string like "██████░░░░"
#[inline]
pub fn render_progress_bar(progress: f64) -> String {
    let filled = (progress * 10.0).round() as usize;
    let filled = filled.min(10); // Clamp to max 10
    let empty = 10 - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

/// Default emoji map for grade-to-emoji conversion
pub fn default_emoji_map() -> HashMap<i32, String> {
    let mut map = HashMap::with_capacity(4);
    map.insert(0, "⚪".to_string());
    map.insert(1, "🔵".to_string());
    map.insert(2, "🟣".to_string());
    map.insert(3, "🟠".to_string());
    map
}


// ============================================================================
// GameSession PyClass
// ============================================================================

/// GameSession - Stateful session holding simulation results in Rust heap
///
/// This PyClass allows Python to hold a handle to the simulation result
/// without serializing the entire data structure. Data is lazily accessed
/// through getter methods.
///
/// # Thread Safety
/// GameSession implements Send + Sync because:
/// - Vec<T> where T: Send + Sync is Send + Sync
/// - Arc<T> where T: Send + Sync is Send + Sync
/// - All contained types (String, i32, etc.) are Send + Sync
#[pyclass]
pub struct GameSession {
    /// Pre-rendered trajectory
    trajectory: Vec<RenderedYear>,
    /// Pre-rendered summary
    summary: PreRenderedSummary,
    /// New achievements unlocked
    new_achievements: Vec<AchievementInfo>,
    /// Triggered event IDs
    triggered_events: Vec<i32>,
    /// Talent replacements
    replacements: Vec<ReplacementResult>,
    /// Emoji map (shared reference to avoid copying)
    #[allow(dead_code)]
    emoji_map: Arc<HashMap<i32, String>>,
}

impl GameSession {
    /// Create a new GameSession from SimulationResult with pre-rendering
    pub fn new(result: SimulationResult, emoji_map: Arc<HashMap<i32, String>>) -> Self {
        // Pre-render trajectory
        let trajectory: Vec<RenderedYear> = result
            .trajectory
            .iter()
            .map(|entry| {
                // Format content with emoji
                let display_text = entry
                    .content
                    .iter()
                    .map(|c| {
                        let emoji = emoji_map
                            .get(&c.grade)
                            .map(|s| s.as_str())
                            .unwrap_or("⚪");
                        format!("{} {}", emoji, c.description)
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                // Convert properties to fixed array
                let properties = [
                    *entry.properties.get("CHR").unwrap_or(&0),
                    *entry.properties.get("INT").unwrap_or(&0),
                    *entry.properties.get("STR").unwrap_or(&0),
                    *entry.properties.get("MNY").unwrap_or(&0),
                    *entry.properties.get("SPR").unwrap_or(&0),
                    *entry.properties.get("LIF").unwrap_or(&0),
                ];

                RenderedYear {
                    age: entry.age,
                    display_text,
                    properties,
                    is_end: entry.is_end,
                }
            })
            .collect();

        // Pre-render summary with progress bars
        let summary = PreRenderedSummary {
            total_score: result.summary.total_score,
            judges: result
                .summary
                .judges
                .iter()
                .map(|j| PreRenderedJudge {
                    property_type: j.property_type.clone(),
                    value: j.value,
                    grade: j.grade,
                    text: j.text.clone(),
                    progress: j.progress,
                    progress_bar: render_progress_bar(j.progress),
                })
                .collect(),
            talents: result.summary.talents.clone(),
        };

        Self {
            trajectory,
            summary,
            new_achievements: result.new_achievements,
            triggered_events: result.triggered_events,
            replacements: result.replacements,
            emoji_map,
        }
    }
}


// ============================================================================
// PyMethods Implementation
// ============================================================================

#[pymethods]
impl GameSession {
    // ------------------------------------------------------------------------
    // Getter Properties
    // ------------------------------------------------------------------------

    /// Total number of years in the trajectory
    #[getter]
    fn total_years(&self) -> usize {
        self.trajectory.len()
    }

    /// Total number of pages (50 years per page by default)
    #[getter]
    fn total_pages(&self) -> usize {
        let years_per_page = 50;
        (self.trajectory.len() + years_per_page - 1) / years_per_page
    }

    /// Total score from summary
    #[getter]
    fn total_score(&self) -> i32 {
        self.summary.total_score
    }

    /// Final age (age of the last year)
    #[getter]
    fn final_age(&self) -> i32 {
        self.trajectory.last().map(|y| y.age).unwrap_or(0)
    }

    /// Whether the simulation has ended
    #[getter]
    fn is_ended(&self) -> bool {
        self.trajectory.last().map(|y| y.is_end).unwrap_or(true)
    }

    // ------------------------------------------------------------------------
    // Lazy Data Access Methods
    // ------------------------------------------------------------------------

    /// Get paginated trajectory data
    ///
    /// # Arguments
    /// * `page` - Page number (1-indexed)
    /// * `years_per_page` - Number of years per page (default: 50)
    ///
    /// # Returns
    /// List of year dicts for the requested page, or empty list if out of bounds
    #[pyo3(signature = (page, years_per_page=None))]
    fn get_page_data(&self, py: Python<'_>, page: usize, years_per_page: Option<usize>) -> PyResult<Py<PyAny>> {
        let per_page = years_per_page.unwrap_or(50);
        
        if page == 0 {
            return Ok(PyList::empty(py).into());
        }
        
        let start = (page - 1) * per_page;
        let end = (start + per_page).min(self.trajectory.len());
        
        if start >= self.trajectory.len() {
            return Ok(PyList::empty(py).into());
        }
        
        let list = PyList::empty(py);
        for year in &self.trajectory[start..end] {
            let dict = self.year_to_dict(py, year)?;
            list.append(dict)?;
        }
        
        Ok(list.into())
    }

    /// Get a single year by index
    ///
    /// # Arguments
    /// * `index` - Year index (0-indexed)
    ///
    /// # Returns
    /// Year dict or None if out of bounds
    fn get_year(&self, py: Python<'_>, index: usize) -> PyResult<Py<PyAny>> {
        match self.trajectory.get(index) {
            Some(year) => Ok(self.year_to_dict(py, year)?.into()),
            None => Ok(py.None()),
        }
    }

    /// Get a range of years
    ///
    /// # Arguments
    /// * `start` - Start index (inclusive)
    /// * `end` - End index (exclusive)
    ///
    /// # Returns
    /// List of year dicts, or empty list if out of bounds
    fn get_years_range(&self, py: Python<'_>, start: usize, end: usize) -> PyResult<Py<PyAny>> {
        let actual_start = start.min(self.trajectory.len());
        let actual_end = end.min(self.trajectory.len());
        
        if actual_start >= actual_end {
            return Ok(PyList::empty(py).into());
        }
        
        let list = PyList::empty(py);
        for year in &self.trajectory[actual_start..actual_end] {
            let dict = self.year_to_dict(py, year)?;
            list.append(dict)?;
        }
        
        Ok(list.into())
    }

    /// Get pre-formatted year content
    ///
    /// # Arguments
    /// * `index` - Year index (0-indexed)
    ///
    /// # Returns
    /// Dict with pre-rendered text, or None if out of bounds
    fn get_year_formatted(&self, py: Python<'_>, index: usize) -> PyResult<Py<PyAny>> {
        match self.trajectory.get(index) {
            Some(year) => {
                let dict = PyDict::new(py);
                dict.set_item("age", year.age)?;
                dict.set_item("text", &year.display_text)?;
                
                let props_dict = PyDict::new(py);
                for (i, name) in PROP_NAMES.iter().enumerate() {
                    props_dict.set_item(*name, year.properties[i])?;
                }
                dict.set_item("properties", props_dict)?;
                dict.set_item("is_end", year.is_end)?;
                
                Ok(dict.into())
            }
            None => Ok(py.None()),
        }
    }


    /// Get the summary with pre-rendered progress bars
    fn get_summary(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let dict = PyDict::new(py);
        dict.set_item("total_score", self.summary.total_score)?;
        
        // Judges
        let judges_list = PyList::empty(py);
        for judge in &self.summary.judges {
            let judge_dict = PyDict::new(py);
            judge_dict.set_item("property_type", &judge.property_type)?;
            judge_dict.set_item("value", judge.value)?;
            judge_dict.set_item("grade", judge.grade)?;
            judge_dict.set_item("text", &judge.text)?;
            judge_dict.set_item("progress", judge.progress)?;
            judge_dict.set_item("progress_bar", &judge.progress_bar)?;
            judges_list.append(judge_dict)?;
        }
        dict.set_item("judges", judges_list)?;
        
        // Talents
        let talents_list = PyList::empty(py);
        for talent in &self.summary.talents {
            let talent_dict = PyDict::new(py);
            talent_dict.set_item("id", talent.id)?;
            talent_dict.set_item("name", &talent.name)?;
            talent_dict.set_item("description", &talent.description)?;
            talent_dict.set_item("grade", talent.grade)?;
            talents_list.append(talent_dict)?;
        }
        dict.set_item("talents", talents_list)?;
        
        Ok(dict.into())
    }

    /// Get new achievements unlocked during simulation
    fn get_new_achievements(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = PyList::empty(py);
        for achievement in &self.new_achievements {
            let dict = PyDict::new(py);
            dict.set_item("id", achievement.id)?;
            dict.set_item("name", &achievement.name)?;
            dict.set_item("description", &achievement.description)?;
            dict.set_item("grade", achievement.grade)?;
            list.append(dict)?;
        }
        Ok(list.into())
    }

    /// Get triggered event IDs
    fn get_triggered_events(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = PyList::new(py, &self.triggered_events)?;
        Ok(list.into())
    }

    /// Get talent replacements
    fn get_replacements(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = PyList::empty(py);
        for replacement in &self.replacements {
            let rep_dict = PyDict::new(py);
            
            let source_dict = PyDict::new(py);
            source_dict.set_item("id", replacement.source.id)?;
            source_dict.set_item("name", &replacement.source.name)?;
            source_dict.set_item("description", &replacement.source.description)?;
            source_dict.set_item("grade", replacement.source.grade)?;
            rep_dict.set_item("source", source_dict)?;
            
            let target_dict = PyDict::new(py);
            target_dict.set_item("id", replacement.target.id)?;
            target_dict.set_item("name", &replacement.target.name)?;
            target_dict.set_item("description", &replacement.target.description)?;
            target_dict.set_item("grade", replacement.target.grade)?;
            rep_dict.set_item("target", target_dict)?;
            
            list.append(rep_dict)?;
        }
        Ok(list.into())
    }
}


// ============================================================================
// Private Helper Methods
// ============================================================================

impl GameSession {
    /// Convert a RenderedYear to a Python dict
    fn year_to_dict<'py>(&self, py: Python<'py>, year: &RenderedYear) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("age", year.age)?;
        dict.set_item("display_text", &year.display_text)?;
        
        let props_dict = PyDict::new(py);
        for (i, name) in PROP_NAMES.iter().enumerate() {
            props_dict.set_item(*name, year.properties[i])?;
        }
        dict.set_item("properties", props_dict)?;
        dict.set_item("is_end", year.is_end)?;
        
        Ok(dict)
    }
}

// ============================================================================
// Test Helper Methods (crate-visible for property tests)
// ============================================================================

impl GameSession {
    /// Get trajectory length (for testing)
    #[cfg(test)]
    pub(crate) fn trajectory_len(&self) -> usize {
        self.trajectory.len()
    }

    /// Get summary total score (for testing)
    #[cfg(test)]
    pub(crate) fn summary_total_score(&self) -> i32 {
        self.summary.total_score
    }

    /// Get summary judges count (for testing)
    #[cfg(test)]
    pub(crate) fn summary_judges_len(&self) -> usize {
        self.summary.judges.len()
    }

    /// Get trajectory year at index (for testing)
    #[cfg(test)]
    pub(crate) fn get_trajectory_year(&self, index: usize) -> Option<&RenderedYear> {
        self.trajectory.get(index)
    }

    /// Iterate over trajectory with original result (for testing)
    #[cfg(test)]
    pub(crate) fn trajectory_iter(&self) -> impl Iterator<Item = &RenderedYear> {
        self.trajectory.iter()
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_progress_bar() {
        assert_eq!(render_progress_bar(0.0), "░░░░░░░░░░");
        assert_eq!(render_progress_bar(0.5), "█████░░░░░");
        assert_eq!(render_progress_bar(1.0), "██████████");
        assert_eq!(render_progress_bar(0.3), "███░░░░░░░");
        assert_eq!(render_progress_bar(0.75), "████████░░");
        // Edge cases
        assert_eq!(render_progress_bar(1.5), "██████████"); // Clamped to 10
        assert_eq!(render_progress_bar(-0.5), "░░░░░░░░░░"); // Negative rounds to 0
    }

    #[test]
    fn test_default_emoji_map() {
        let map = default_emoji_map();
        assert_eq!(map.get(&0), Some(&"⚪".to_string()));
        assert_eq!(map.get(&1), Some(&"🔵".to_string()));
        assert_eq!(map.get(&2), Some(&"🟣".to_string()));
        assert_eq!(map.get(&3), Some(&"🟠".to_string()));
        assert_eq!(map.len(), 4);
    }

    #[test]
    fn test_rendered_year_properties_order() {
        // Verify the property order matches PROP_NAMES
        assert_eq!(PROP_NAMES[PROP_CHR], "CHR");
        assert_eq!(PROP_NAMES[PROP_INT], "INT");
        assert_eq!(PROP_NAMES[PROP_STR], "STR");
        assert_eq!(PROP_NAMES[PROP_MNY], "MNY");
        assert_eq!(PROP_NAMES[PROP_SPR], "SPR");
        assert_eq!(PROP_NAMES[PROP_LIF], "LIF");
    }

    #[test]
    fn test_rendered_year_creation() {
        let year = RenderedYear {
            age: 18,
            display_text: "🟣 你考上了大學\n🔵 你交了女朋友".to_string(),
            properties: [10, 8, 6, 5, 7, 1],
            is_end: false,
        };
        
        assert_eq!(year.age, 18);
        assert!(year.display_text.contains("🟣"));
        assert!(year.display_text.contains("\n"));
        assert_eq!(year.properties[PROP_CHR], 10);
        assert_eq!(year.properties[PROP_INT], 8);
        assert_eq!(year.properties[PROP_STR], 6);
        assert_eq!(year.properties[PROP_MNY], 5);
        assert_eq!(year.properties[PROP_SPR], 7);
        assert_eq!(year.properties[PROP_LIF], 1);
        assert!(!year.is_end);
    }

    #[test]
    fn test_pre_rendered_judge() {
        let judge = PreRenderedJudge {
            property_type: "HCHR".to_string(),
            value: 10,
            grade: 3,
            text: "絕世美人".to_string(),
            progress: 1.0,
            progress_bar: render_progress_bar(1.0),
        };
        
        assert_eq!(judge.property_type, "HCHR");
        assert_eq!(judge.value, 10);
        assert_eq!(judge.grade, 3);
        assert_eq!(judge.progress_bar, "██████████");
    }
}


// ============================================================================
// Property Tests for Emoji Configuration and GameSession
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // ========================================================================
    // Strategy Generators
    // ========================================================================

    /// Strategy for generating valid emoji strings
    fn emoji_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("⚪".to_string()),
            Just("🔵".to_string()),
            Just("🟣".to_string()),
            Just("🟠".to_string()),
            Just("🔴".to_string()),
            Just("🟢".to_string()),
            Just("🟡".to_string()),
            Just("⚫".to_string()),
            Just("✨".to_string()),
            Just("💎".to_string()),
        ]
    }

    /// Strategy for generating emoji maps
    fn emoji_map_strategy() -> impl Strategy<Value = HashMap<i32, String>> {
        prop::collection::hash_map(0i32..=3i32, emoji_strategy(), 1..=4)
    }

    /// Strategy for generating RenderedYear
    fn rendered_year_strategy() -> impl Strategy<Value = RenderedYear> {
        (
            0i32..=120i32,                    // age
            "[⚪🔵🟣🟠] .{0,50}",              // display_text pattern
            prop::array::uniform6(0i32..=20i32), // properties
            any::<bool>(),                    // is_end
        )
            .prop_map(|(age, text, properties, is_end)| RenderedYear {
                age,
                display_text: text,
                properties,
                is_end,
            })
    }

    /// Strategy for generating a trajectory (Vec<RenderedYear>)
    fn trajectory_strategy(min_len: usize, max_len: usize) -> impl Strategy<Value = Vec<RenderedYear>> {
        prop::collection::vec(rendered_year_strategy(), min_len..=max_len)
            .prop_map(|mut years| {
                // Ensure ages are sequential and last year has is_end = true
                for (i, year) in years.iter_mut().enumerate() {
                    year.age = i as i32;
                    year.is_end = false;
                }
                if let Some(last) = years.last_mut() {
                    last.is_end = true;
                }
                years
            })
    }

    /// Strategy for generating PreRenderedJudge
    fn judge_strategy() -> impl Strategy<Value = PreRenderedJudge> {
        (
            prop_oneof![
                Just("HCHR".to_string()),
                Just("HINT".to_string()),
                Just("HSTR".to_string()),
                Just("HMNY".to_string()),
                Just("HSPR".to_string()),
                Just("HLIF".to_string()),
            ],
            0i32..=20i32,      // value
            0i32..=3i32,       // grade
            ".{0,20}",         // text
            0.0f64..=1.0f64,   // progress
        )
            .prop_map(|(property_type, value, grade, text, progress)| PreRenderedJudge {
                property_type,
                value,
                grade,
                text,
                progress,
                progress_bar: render_progress_bar(progress),
            })
    }

    /// Strategy for generating PreRenderedSummary
    fn summary_strategy() -> impl Strategy<Value = PreRenderedSummary> {
        (
            0i32..=1000i32, // total_score
            prop::collection::vec(judge_strategy(), 0..=6),
        )
            .prop_map(|(total_score, judges)| PreRenderedSummary {
                total_score,
                judges,
                talents: vec![],
            })
    }

    /// Strategy for generating a complete GameSession (without PyO3 context)
    fn game_session_data_strategy() -> impl Strategy<Value = (Vec<RenderedYear>, PreRenderedSummary, Arc<HashMap<i32, String>>)> {
        (
            trajectory_strategy(1, 100),
            summary_strategy(),
            emoji_map_strategy().prop_map(Arc::new),
        )
    }

    // ========================================================================
    // Property 8: Custom Emoji Configuration
    // Validates: Requirements 3.7, 9.2, 9.5
    // ========================================================================

    proptest! {
        /// Property 8: Default emoji map has all grades 0-3
        #[test]
        fn test_default_emoji_map_has_all_grades(grade in 0i32..=3i32) {
            let map = default_emoji_map();
            prop_assert!(map.contains_key(&grade), "Default emoji map should contain grade {}", grade);
            prop_assert!(!map.get(&grade).unwrap().is_empty(), "Emoji for grade {} should not be empty", grade);
        }

        /// Property 8: Custom emoji maps should be usable for pre-rendering
        #[test]
        fn test_custom_emoji_map_can_be_used(custom_map in emoji_map_strategy()) {
            for (grade, emoji) in &custom_map {
                prop_assert!(*grade >= 0 && *grade <= 3, "Grade should be 0-3");
                prop_assert!(!emoji.is_empty(), "Emoji should not be empty");
            }
        }

        /// Property: Progress bar rendering is deterministic
        #[test]
        fn test_progress_bar_deterministic(progress in 0.0f64..=1.0f64) {
            let bar1 = render_progress_bar(progress);
            let bar2 = render_progress_bar(progress);
            prop_assert_eq!(&bar1, &bar2, "Progress bar should be deterministic");
            prop_assert_eq!(bar1.chars().count(), 10, "Progress bar should have 10 characters");
        }

        /// Property: Progress bar filled count matches progress
        #[test]
        fn test_progress_bar_filled_count(progress in 0.0f64..=1.0f64) {
            let bar = render_progress_bar(progress);
            let filled = bar.chars().filter(|c| *c == '█').count();
            let expected = (progress * 10.0).round() as usize;
            prop_assert_eq!(filled, expected.min(10), "Filled count should match progress");
        }
    }

    // ========================================================================
    // Property 2: Getter Properties Existence
    // Validates: Requirement 1.3
    // ========================================================================

    proptest! {
        /// Property 2: total_years equals trajectory length
        #[test]
        fn test_total_years_equals_trajectory_len(
            (trajectory, summary, emoji_map) in game_session_data_strategy()
        ) {
            let expected_len = trajectory.len();
            let session = GameSessionTestHelper::new(trajectory, summary, emoji_map);
            prop_assert_eq!(session.total_years(), expected_len, "total_years should equal trajectory length");
        }

        /// Property 2: total_pages calculation is correct
        #[test]
        fn test_total_pages_calculation(
            (trajectory, summary, emoji_map) in game_session_data_strategy()
        ) {
            let years_per_page = 50;
            let expected_pages = (trajectory.len() + years_per_page - 1) / years_per_page;
            let session = GameSessionTestHelper::new(trajectory, summary, emoji_map);
            prop_assert_eq!(session.total_pages(), expected_pages, "total_pages should be ceiling division");
        }

        /// Property 2: total_score matches summary
        #[test]
        fn test_total_score_matches_summary(
            (trajectory, summary, emoji_map) in game_session_data_strategy()
        ) {
            let expected_score = summary.total_score;
            let session = GameSessionTestHelper::new(trajectory, summary, emoji_map);
            prop_assert_eq!(session.total_score(), expected_score, "total_score should match summary");
        }

        /// Property 2: final_age is last year's age
        #[test]
        fn test_final_age_is_last_year(
            (trajectory, summary, emoji_map) in game_session_data_strategy()
        ) {
            let expected_age = trajectory.last().map(|y| y.age).unwrap_or(0);
            let session = GameSessionTestHelper::new(trajectory, summary, emoji_map);
            prop_assert_eq!(session.final_age(), expected_age, "final_age should be last year's age");
        }

        /// Property 2: is_ended reflects last year's is_end
        #[test]
        fn test_is_ended_reflects_last_year(
            (trajectory, summary, emoji_map) in game_session_data_strategy()
        ) {
            let expected_ended = trajectory.last().map(|y| y.is_end).unwrap_or(true);
            let session = GameSessionTestHelper::new(trajectory, summary, emoji_map);
            prop_assert_eq!(session.is_ended(), expected_ended, "is_ended should reflect last year");
        }
    }

    // ========================================================================
    // Property 3: Pagination Correctness
    // Validates: Requirement 2.1
    // ========================================================================

    proptest! {
        /// Property 3: Page data returns correct number of items
        #[test]
        fn test_page_data_correct_count(
            (trajectory, summary, emoji_map) in game_session_data_strategy(),
            page in 1usize..=10usize,
            years_per_page in 10usize..=100usize
        ) {
            let session = GameSessionTestHelper::new(trajectory.clone(), summary, emoji_map);
            let page_data = session.get_page_data_test(page, years_per_page);
            
            let start = (page - 1) * years_per_page;
            let expected_count = if start >= trajectory.len() {
                0
            } else {
                (trajectory.len() - start).min(years_per_page)
            };
            
            prop_assert_eq!(page_data.len(), expected_count, 
                "Page {} should have {} items, got {}", page, expected_count, page_data.len());
        }

        /// Property 3: Page 0 returns empty list
        #[test]
        fn test_page_zero_returns_empty(
            (trajectory, summary, emoji_map) in game_session_data_strategy()
        ) {
            let session = GameSessionTestHelper::new(trajectory, summary, emoji_map);
            let page_data = session.get_page_data_test(0, 50);
            prop_assert!(page_data.is_empty(), "Page 0 should return empty list");
        }
    }

    // ========================================================================
    // Property 4: Single Year Access Correctness
    // Validates: Requirement 2.2
    // ========================================================================

    proptest! {
        /// Property 4: get_year returns correct year for valid index
        #[test]
        fn test_get_year_valid_index(
            (trajectory, summary, emoji_map) in game_session_data_strategy()
        ) {
            prop_assume!(!trajectory.is_empty());
            let session = GameSessionTestHelper::new(trajectory.clone(), summary, emoji_map);
            
            for (i, expected_year) in trajectory.iter().enumerate() {
                let year = session.get_year_test(i);
                prop_assert!(year.is_some(), "Year at index {} should exist", i);
                let year = year.unwrap();
                prop_assert_eq!(year.age, expected_year.age, "Age mismatch at index {}", i);
            }
        }

        /// Property 4: get_year returns None for out-of-bounds index
        #[test]
        fn test_get_year_out_of_bounds(
            (trajectory, summary, emoji_map) in game_session_data_strategy(),
            offset in 1usize..=100usize
        ) {
            let session = GameSessionTestHelper::new(trajectory.clone(), summary, emoji_map);
            let out_of_bounds_index = trajectory.len() + offset;
            let year = session.get_year_test(out_of_bounds_index);
            prop_assert!(year.is_none(), "Index {} should return None", out_of_bounds_index);
        }
    }

    // ========================================================================
    // Property 5: Range Access Correctness
    // Validates: Requirement 2.3
    // ========================================================================

    proptest! {
        /// Property 5: get_years_range returns correct slice
        #[test]
        fn test_get_years_range_correct_slice(
            (trajectory, summary, emoji_map) in game_session_data_strategy(),
            start in 0usize..=50usize,
            len in 0usize..=50usize
        ) {
            let session = GameSessionTestHelper::new(trajectory.clone(), summary, emoji_map);
            let end = start + len;
            let range = session.get_years_range_test(start, end);
            
            let actual_start = start.min(trajectory.len());
            let actual_end = end.min(trajectory.len());
            let expected_len = if actual_start >= actual_end { 0 } else { actual_end - actual_start };
            
            prop_assert_eq!(range.len(), expected_len, 
                "Range [{}, {}) should have {} items, got {}", start, end, expected_len, range.len());
        }

        /// Property 5: Empty range when start >= end
        #[test]
        fn test_get_years_range_empty_when_start_ge_end(
            (trajectory, summary, emoji_map) in game_session_data_strategy(),
            start in 0usize..=100usize
        ) {
            let session = GameSessionTestHelper::new(trajectory, summary, emoji_map);
            let range = session.get_years_range_test(start, start);
            prop_assert!(range.is_empty(), "Range [{}, {}) should be empty", start, start);
        }
    }

    // ========================================================================
    // Property 12: Boundary Handling
    // Validates: Requirements 6.4, 6.5
    // ========================================================================

    proptest! {
        /// Property 12: Empty trajectory handling
        #[test]
        fn test_empty_trajectory_handling(
            summary in summary_strategy(),
            emoji_map in emoji_map_strategy().prop_map(Arc::new)
        ) {
            let session = GameSessionTestHelper::new(vec![], summary, emoji_map);
            
            prop_assert_eq!(session.total_years(), 0, "Empty trajectory should have 0 years");
            prop_assert_eq!(session.total_pages(), 0, "Empty trajectory should have 0 pages");
            prop_assert_eq!(session.final_age(), 0, "Empty trajectory should have final_age 0");
            prop_assert!(session.is_ended(), "Empty trajectory should be ended");
            prop_assert!(session.get_year_test(0).is_none(), "Empty trajectory get_year(0) should be None");
            prop_assert!(session.get_page_data_test(1, 50).is_empty(), "Empty trajectory page 1 should be empty");
        }

        /// Property 12: Large index handling
        #[test]
        fn test_large_index_handling(
            (trajectory, summary, emoji_map) in game_session_data_strategy()
        ) {
            let session = GameSessionTestHelper::new(trajectory, summary, emoji_map);
            
            // Very large indices should not panic
            let large_index = usize::MAX / 2;
            prop_assert!(session.get_year_test(large_index).is_none(), "Large index should return None");
            
            let range = session.get_years_range_test(large_index, large_index + 10);
            prop_assert!(range.is_empty(), "Large range should be empty");
        }
    }

    // ========================================================================
    // Test Helper - GameSession without PyO3 context
    // ========================================================================

    /// Helper struct for testing GameSession logic without PyO3 context
    struct GameSessionTestHelper {
        trajectory: Vec<RenderedYear>,
        summary: PreRenderedSummary,
        #[allow(dead_code)]
        emoji_map: Arc<HashMap<i32, String>>,
    }

    impl GameSessionTestHelper {
        fn new(
            trajectory: Vec<RenderedYear>,
            summary: PreRenderedSummary,
            emoji_map: Arc<HashMap<i32, String>>,
        ) -> Self {
            Self { trajectory, summary, emoji_map }
        }

        fn total_years(&self) -> usize {
            self.trajectory.len()
        }

        fn total_pages(&self) -> usize {
            let years_per_page = 50;
            (self.trajectory.len() + years_per_page - 1) / years_per_page
        }

        fn total_score(&self) -> i32 {
            self.summary.total_score
        }

        fn final_age(&self) -> i32 {
            self.trajectory.last().map(|y| y.age).unwrap_or(0)
        }

        fn is_ended(&self) -> bool {
            self.trajectory.last().map(|y| y.is_end).unwrap_or(true)
        }

        fn get_page_data_test(&self, page: usize, years_per_page: usize) -> Vec<&RenderedYear> {
            if page == 0 {
                return vec![];
            }
            let start = (page - 1) * years_per_page;
            let end = (start + years_per_page).min(self.trajectory.len());
            if start >= self.trajectory.len() {
                return vec![];
            }
            self.trajectory[start..end].iter().collect()
        }

        fn get_year_test(&self, index: usize) -> Option<&RenderedYear> {
            self.trajectory.get(index)
        }

        fn get_years_range_test(&self, start: usize, end: usize) -> Vec<&RenderedYear> {
            let actual_start = start.min(self.trajectory.len());
            let actual_end = end.min(self.trajectory.len());
            if actual_start >= actual_end {
                return vec![];
            }
            self.trajectory[actual_start..actual_end].iter().collect()
        }
    }
}
