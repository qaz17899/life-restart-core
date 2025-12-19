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
use pyo3::types::{PyAny, PyDict, PyDictMethods, PyList, PyListMethods};
use pyo3::Py;
use std::collections::HashMap;

/// Simulate a complete life trajectory
///
/// # Arguments
/// * `talent_ids` - List of selected talent IDs
/// * `properties` - Initial property allocation {CHR, INT, STR, MNY}
/// * `achieved_list` - List of already achieved achievement IDs
/// * `config` - Game configuration containing talents, events, ages, achievements
///
/// # Returns
/// A dictionary containing trajectory, summary, new_achievements, triggered_events, and replacements
#[pyfunction]
fn simulate_full_life(
    py: Python<'_>,
    talent_ids: Vec<i32>,
    properties: &Bound<'_, PyDict>,
    achieved_list: &Bound<'_, PyList>,
    config: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    // Deserialize config
    let talents = config::deserialize_talents(config)?;
    let events = config::deserialize_events(config)?;
    let ages = config::deserialize_ages(config)?;
    let achievements = config::deserialize_achievements(config)?;
    let judge_config = config::deserialize_judge_config(config)?;

    // Create simulation engine
    let engine = SimulationEngine::new(talents, events, ages, achievements, judge_config);

    // Deserialize input
    let props = deserialize_properties(properties)?;
    let achieved = deserialize_achieved_list(achieved_list)?;

    // Run simulation
    let result = engine
        .simulate(&talent_ids, &props, &achieved)
        .map_err(|e| LifeRestartError::from(e))?;

    // Serialize result to Python dict
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

fn deserialize_achieved_list(list: &Bound<'_, PyList>) -> PyResult<Vec<Vec<i32>>> {
    let mut achieved = Vec::new();
    for item in list.iter() {
        let inner: Vec<i32> = item.extract()?;
        achieved.push(inner);
    }
    Ok(achieved)
}

fn serialize_result(
    py: Python<'_>,
    result: &simulator::SimulationResult,
) -> PyResult<Py<PyAny>> {
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
        source_dict.set_item("id", replacement.source_id)?;
        source_dict.set_item("name", &replacement.source_name)?;
        rep_dict.set_item("source", source_dict)?;

        let target_dict = PyDict::new(py);
        target_dict.set_item("id", replacement.target_id)?;
        target_dict.set_item("name", &replacement.target_name)?;
        rep_dict.set_item("target", target_dict)?;

        replacements_list.append(rep_dict)?;
    }
    dict.set_item("replacements", replacements_list)?;

    Ok(dict.into())
}

/// Python module definition
#[pymodule]
fn life_restart_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(simulate_full_life, m)?)?;
    Ok(())
}
