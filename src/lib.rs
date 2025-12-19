//! Life Restart Core - High-performance life restart simulator engine
//!
//! This crate provides a Rust implementation of the life restart simulator
//! with Python bindings via PyO3.

use pyo3::prelude::*;

pub mod achievement;
pub mod condition;
pub mod config;
pub mod error;
pub mod event;
pub mod property;
pub mod simulator;
pub mod talent;

use crate::error::LifeRestartError;
use crate::simulator::SimulationEngine;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use pyo3::types::{PyAny, PyDict, PyDictMethods, PyList, PyListMethods, PySet, PySetMethods};
use pyo3::Py;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Global cached simulation engine
static CACHED_ENGINE: OnceCell<Arc<RwLock<SimulationEngine>>> = OnceCell::new();

/// Initialize the game configuration (call once at startup)
///
/// This caches the configuration in Rust memory, eliminating the need to
/// deserialize it on every simulation call. Call this once when your bot starts.
///
/// # Arguments
/// * `config` - Game configuration containing talents, events, ages, achievements
#[pyfunction]
fn init_config(config: &Bound<'_, PyDict>) -> PyResult<()> {
    let talents = config::deserialize_talents(config)?;
    let events = config::deserialize_events(config)?;
    let ages = config::deserialize_ages(config)?;
    let achievements = config::deserialize_achievements(config)?;
    let judge_config = config::deserialize_judge_config(config)?;

    let engine = SimulationEngine::new(talents, events, ages, achievements, judge_config);

    // If already initialized, update the engine
    if let Some(cached) = CACHED_ENGINE.get() {
        let mut guard = cached.write();
        *guard = engine;
    } else {
        let _ = CACHED_ENGINE.set(Arc::new(RwLock::new(engine)));
    }

    Ok(())
}

/// Check if config is initialized
#[pyfunction]
fn is_config_initialized() -> bool {
    CACHED_ENGINE.get().is_some()
}

/// Simulate a complete life trajectory (fast version using cached config)
///
/// # Arguments
/// * `talent_ids` - List of selected talent IDs
/// * `properties` - Initial property allocation {CHR, INT, STR, MNY}
/// * `achieved_ids` - Set of already achieved achievement IDs
///
/// # Returns
/// A dictionary containing trajectory, summary, new_achievements, triggered_events, and replacements
///
/// # Panics
/// Panics if `init_config` was not called first
#[pyfunction]
fn simulate_full_life(
    py: Python<'_>,
    talent_ids: Vec<i32>,
    properties: &Bound<'_, PyDict>,
    achieved_ids: &Bound<'_, PySet>,
) -> PyResult<Py<PyAny>> {
    // Get cached engine
    let engine_arc = CACHED_ENGINE
        .get()
        .ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Config not initialized. Call init_config() first.",
            )
        })?;

    let engine = engine_arc.read();

    // Deserialize input
    let props = deserialize_properties(properties)?;
    let achieved = deserialize_achieved_ids(achieved_ids)?;

    // Run simulation
    let result = engine
        .simulate(&talent_ids, &props, &achieved)
        .map_err(|e| LifeRestartError::from(e))?;

    // Serialize result to Python dict
    serialize_result(py, &result)
}

/// Simulate with explicit config (slower, for backwards compatibility or dynamic config)
///
/// Use this if you need to use different configs for different simulations.
/// For most cases, use `init_config` + `simulate_full_life` instead.
#[pyfunction]
fn simulate_with_config(
    py: Python<'_>,
    talent_ids: Vec<i32>,
    properties: &Bound<'_, PyDict>,
    achieved_ids: &Bound<'_, PySet>,
    config: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    // Deserialize config every time (slower)
    let talents = config::deserialize_talents(config)?;
    let events = config::deserialize_events(config)?;
    let ages = config::deserialize_ages(config)?;
    let achievements = config::deserialize_achievements(config)?;
    let judge_config = config::deserialize_judge_config(config)?;

    let engine = SimulationEngine::new(talents, events, ages, achievements, judge_config);

    // Deserialize input
    let props = deserialize_properties(properties)?;
    let achieved = deserialize_achieved_ids(achieved_ids)?;

    // Run simulation
    let result = engine
        .simulate(&talent_ids, &props, &achieved)
        .map_err(|e| LifeRestartError::from(e))?;

    serialize_result(py, &result)
}

fn deserialize_properties(dict: &Bound<'_, PyDict>) -> PyResult<HashMap<String, i32>> {
    let mut props = HashMap::new();
    for (key, value) in dict.iter() {
        let key: String = key.extract()?;
        let value: i32 = value.extract()?;
        props.insert(key, value);
    }
    Ok(props)
}

fn deserialize_achieved_ids(set: &Bound<'_, PySet>) -> PyResult<HashSet<i32>> {
    let mut achieved = HashSet::new();
    for item in set.iter() {
        let id: i32 = item.extract()?;
        achieved.insert(id);
    }
    Ok(achieved)
}

fn serialize_result(py: Python<'_>, result: &simulator::SimulationResult) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);

    // Serialize trajectory
    let trajectory = PyList::empty(py);
    for entry in &result.trajectory {
        let entry_dict = PyDict::new(py);
        entry_dict.set_item("age", entry.age)?;

        let content_list = PyList::empty(py);
        for content in &entry.content {
            let content_dict = PyDict::new(py);
            content_dict.set_item("type", &content.content_type)?;
            content_dict.set_item("description", &content.description)?;
            content_dict.set_item("grade", content.grade)?;
            if let Some(ref name) = content.name {
                content_dict.set_item("name", name)?;
            }
            content_list.append(content_dict)?;
        }
        entry_dict.set_item("content", content_list)?;
        entry_dict.set_item("is_end", entry.is_end)?;

        let props_dict = PyDict::new(py);
        for (k, v) in &entry.properties {
            props_dict.set_item(k, v)?;
        }
        entry_dict.set_item("properties", props_dict)?;

        trajectory.append(entry_dict)?;
    }
    dict.set_item("trajectory", trajectory)?;

    // Serialize summary
    let summary_dict = PyDict::new(py);
    summary_dict.set_item("total_score", result.summary.total_score)?;

    let judges_list = PyList::empty(py);
    for judge in &result.summary.judges {
        let judge_dict = PyDict::new(py);
        judge_dict.set_item("property_type", &judge.property_type)?;
        judge_dict.set_item("value", judge.value)?;
        judge_dict.set_item("grade", judge.grade)?;
        judge_dict.set_item("text", &judge.text)?;
        judge_dict.set_item("progress", judge.progress)?;
        judges_list.append(judge_dict)?;
    }
    summary_dict.set_item("judges", judges_list)?;

    let talents_list = PyList::empty(py);
    for talent in &result.summary.talents {
        let talent_dict = PyDict::new(py);
        talent_dict.set_item("id", talent.id)?;
        talent_dict.set_item("name", &talent.name)?;
        talent_dict.set_item("description", &talent.description)?;
        talent_dict.set_item("grade", talent.grade)?;
        talents_list.append(talent_dict)?;
    }
    summary_dict.set_item("talents", talents_list)?;
    dict.set_item("summary", summary_dict)?;

    // Serialize new_achievements
    let achievements_list = PyList::empty(py);
    for achievement in &result.new_achievements {
        let ach_dict = PyDict::new(py);
        ach_dict.set_item("id", achievement.id)?;
        ach_dict.set_item("name", &achievement.name)?;
        ach_dict.set_item("description", &achievement.description)?;
        ach_dict.set_item("grade", achievement.grade)?;
        achievements_list.append(ach_dict)?;
    }
    dict.set_item("new_achievements", achievements_list)?;

    // Serialize triggered_events
    let events_list = PyList::new(py, &result.triggered_events)?;
    dict.set_item("triggered_events", events_list)?;

    // Serialize replacements
    let replacements_list = PyList::empty(py);
    for replacement in &result.replacements {
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

        replacements_list.append(rep_dict)?;
    }
    dict.set_item("replacements", replacements_list)?;

    Ok(dict.into())
}

/// Python module definition
#[pymodule]
fn life_restart_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(init_config, m)?)?;
    m.add_function(wrap_pyfunction!(is_config_initialized, m)?)?;
    m.add_function(wrap_pyfunction!(simulate_full_life, m)?)?;
    m.add_function(wrap_pyfunction!(simulate_with_config, m)?)?;
    Ok(())
}
