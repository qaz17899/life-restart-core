//! Configuration module for game data structures
//!
//! This module handles deserialization of game configuration from Python dicts.

mod achievement;
mod age;
mod event;
mod judge;
mod talent;

pub use achievement::*;
pub use age::*;
pub use event::*;
pub use judge::*;
pub use talent::*;

use crate::error::LifeRestartError;
use pyo3::types::{PyAnyMethods, PyDict, PyDictMethods, PyList, PyListMethods};
use pyo3::Bound;
use std::collections::HashMap;

/// Helper to get attribute from either dict or object
fn get_attr<'py>(obj: &Bound<'py, pyo3::PyAny>, name: &str) -> pyo3::PyResult<Bound<'py, pyo3::PyAny>> {
    if let Ok(dict) = obj.downcast::<PyDict>() {
        dict.get_item(name)?
            .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err(name.to_string()))
    } else {
        obj.getattr(name)
    }
}

/// Helper to get optional attribute from either dict or object
fn get_attr_opt<'py>(obj: &Bound<'py, pyo3::PyAny>, name: &str) -> Option<Bound<'py, pyo3::PyAny>> {
    if let Ok(dict) = obj.downcast::<PyDict>() {
        dict.get_item(name).ok().flatten()
    } else {
        obj.getattr(name).ok()
    }
}

/// Deserialize talents from Python config dict
/// Expected format: {"talents": {id: TalentConfig, ...}}
pub fn deserialize_talents(
    config: &Bound<'_, PyDict>,
) -> pyo3::PyResult<HashMap<i32, TalentConfig>> {
    let talents_dict = config
        .get_item("talents")?
        .ok_or_else(|| LifeRestartError::DeserializationError("talents not found".to_string()))?;

    let talents_dict: Bound<'_, PyDict> = talents_dict.extract()?;
    let mut talents = HashMap::new();

    for (key, value) in talents_dict.iter() {
        // Support both string and integer keys
        let id: i32 = if let Ok(id) = key.extract::<i32>() {
            id
        } else {
            let key_str: String = key.extract()?;
            key_str.parse().map_err(|_| {
                pyo3::exceptions::PyValueError::new_err(format!("Invalid talent id: {}", key_str))
            })?
        };
        let talent = extract_talent(&value)?;
        talents.insert(id, talent);
    }

    Ok(talents)
}

fn extract_talent(obj: &Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<TalentConfig> {
    let id: i32 = get_attr(obj, "id")?.extract()?;
    let name: String = get_attr(obj, "name")?.extract()?;
    let description: String = get_attr(obj, "description")?.extract()?;
    let grade: i32 = get_attr_opt(obj, "grade").and_then(|v| v.extract().ok()).unwrap_or(0);
    let max_triggers: i32 = get_attr_opt(obj, "max_triggers").and_then(|v| v.extract().ok()).unwrap_or(1);
    let condition: Option<String> = get_attr_opt(obj, "condition").and_then(|v| v.extract().ok());
    let exclusive: bool = get_attr_opt(obj, "exclusive").and_then(|v| v.extract().ok()).unwrap_or(false);
    let status: i32 = get_attr_opt(obj, "status").and_then(|v| v.extract().ok()).unwrap_or(0);

    // Extract effect
    let effect = if let Some(effect_obj) = get_attr_opt(obj, "effect") {
        if !effect_obj.is_none() {
            Some(extract_talent_effect(&effect_obj)?)
        } else {
            None
        }
    } else {
        None
    };

    // Extract exclude list
    let exclude = if let Some(exclude_obj) = get_attr_opt(obj, "exclude") {
        if !exclude_obj.is_none() {
            let list: Vec<i32> = exclude_obj.extract()?;
            if list.is_empty() { None } else { Some(list) }
        } else {
            None
        }
    } else {
        None
    };

    // Extract replacement
    let replacement = if let Some(repl_obj) = get_attr_opt(obj, "replacement") {
        if !repl_obj.is_none() {
            Some(extract_talent_replacement(&repl_obj)?)
        } else {
            None
        }
    } else {
        None
    };

    Ok(TalentConfig {
        id,
        name,
        description,
        grade,
        max_triggers,
        condition,
        effect,
        exclusive,
        exclude,
        replacement,
        status,
    })
}

fn extract_talent_effect(obj: &Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<TalentEffect> {
    Ok(TalentEffect {
        chr: get_attr_opt(obj, "CHR").and_then(|v| v.extract().ok()).unwrap_or(0),
        int: get_attr_opt(obj, "INT").and_then(|v| v.extract().ok()).unwrap_or(0),
        str_: get_attr_opt(obj, "STR").and_then(|v| v.extract().ok()).unwrap_or(0),
        mny: get_attr_opt(obj, "MNY").and_then(|v| v.extract().ok()).unwrap_or(0),
        spr: get_attr_opt(obj, "SPR").and_then(|v| v.extract().ok()).unwrap_or(0),
        lif: get_attr_opt(obj, "LIF").and_then(|v| v.extract().ok()).unwrap_or(0),
        age: get_attr_opt(obj, "AGE").and_then(|v| v.extract().ok()).unwrap_or(0),
        rdm: get_attr_opt(obj, "RDM").and_then(|v| v.extract().ok()).unwrap_or(0),
    })
}

fn extract_talent_replacement(obj: &Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<TalentReplacement> {
    let grade = if let Some(grade_obj) = get_attr_opt(obj, "grade") {
        if !grade_obj.is_none() {
            let dict: HashMap<String, f64> = grade_obj.extract()?;
            if dict.is_empty() { None } else { Some(dict) }
        } else {
            None
        }
    } else {
        None
    };

    let talent = if let Some(talent_obj) = get_attr_opt(obj, "talent") {
        if !talent_obj.is_none() {
            let dict: HashMap<String, f64> = talent_obj.extract()?;
            if dict.is_empty() { None } else { Some(dict) }
        } else {
            None
        }
    } else {
        None
    };

    Ok(TalentReplacement { grade, talent })
}

/// Deserialize events from Python config dict
pub fn deserialize_events(
    config: &Bound<'_, PyDict>,
) -> pyo3::PyResult<HashMap<i32, EventConfig>> {
    let events_dict = config
        .get_item("events")?
        .ok_or_else(|| LifeRestartError::DeserializationError("events not found".to_string()))?;

    let events_dict: Bound<'_, PyDict> = events_dict.extract()?;
    let mut events = HashMap::new();

    for (key, value) in events_dict.iter() {
        // Support both string and integer keys
        let id: i32 = if let Ok(id) = key.extract::<i32>() {
            id
        } else {
            let key_str: String = key.extract()?;
            key_str.parse().map_err(|_| {
                pyo3::exceptions::PyValueError::new_err(format!("Invalid event id: {}", key_str))
            })?
        };
        let event = extract_event(&value)?;
        events.insert(id, event);
    }

    Ok(events)
}

fn extract_event(obj: &Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<EventConfig> {
    let id: i32 = get_attr(obj, "id")?.extract()?;
    let event: String = get_attr(obj, "event")?.extract()?;
    let grade: i32 = get_attr_opt(obj, "grade").and_then(|v| v.extract().ok()).unwrap_or(0);
    // Support both "no_random" and "NoRandom" field names
    let no_random: bool = get_attr_opt(obj, "no_random")
        .or_else(|| get_attr_opt(obj, "NoRandom"))
        .and_then(|v| v.extract().ok())
        .unwrap_or(false);
    let include: Option<String> = get_attr_opt(obj, "include").and_then(|v| v.extract().ok());
    let exclude: Option<String> = get_attr_opt(obj, "exclude").and_then(|v| v.extract().ok());
    // Support both "post_event" and "postEvent" field names
    let post_event: Option<String> = get_attr_opt(obj, "post_event")
        .or_else(|| get_attr_opt(obj, "postEvent"))
        .and_then(|v| v.extract().ok());

    // Extract effect
    let effect = if let Some(effect_obj) = get_attr_opt(obj, "effect") {
        if !effect_obj.is_none() {
            Some(extract_event_effect(&effect_obj)?)
        } else {
            None
        }
    } else {
        None
    };

    // Extract branch list
    let branch = if let Some(branch_obj) = get_attr_opt(obj, "branch") {
        if !branch_obj.is_none() {
            let list: Bound<'_, PyList> = branch_obj.extract()?;
            let mut branches = Vec::new();
            for item in list.iter() {
                branches.push(extract_event_branch(&item)?);
            }
            if branches.is_empty() { None } else { Some(branches) }
        } else {
            None
        }
    } else {
        None
    };

    Ok(EventConfig {
        id,
        event,
        grade,
        no_random,
        include,
        exclude,
        effect,
        branch,
        post_event,
    })
}

fn extract_event_effect(obj: &Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<EventEffect> {
    Ok(EventEffect {
        chr: get_attr_opt(obj, "CHR").and_then(|v| v.extract().ok()).unwrap_or(0),
        int: get_attr_opt(obj, "INT").and_then(|v| v.extract().ok()).unwrap_or(0),
        str_: get_attr_opt(obj, "STR").and_then(|v| v.extract().ok()).unwrap_or(0),
        mny: get_attr_opt(obj, "MNY").and_then(|v| v.extract().ok()).unwrap_or(0),
        spr: get_attr_opt(obj, "SPR").and_then(|v| v.extract().ok()).unwrap_or(0),
        lif: get_attr_opt(obj, "LIF").and_then(|v| v.extract().ok()).unwrap_or(0),
        age: get_attr_opt(obj, "AGE").and_then(|v| v.extract().ok()).unwrap_or(0),
        rdm: get_attr_opt(obj, "RDM").and_then(|v| v.extract().ok()).unwrap_or(0),
    })
}

fn extract_event_branch(obj: &Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<EventBranch> {
    let condition: String = get_attr(obj, "condition")?.extract()?;
    // Support both "event_id" and "eventId" field names
    let event_id: i32 = get_attr(obj, "event_id")
        .or_else(|_| get_attr(obj, "eventId"))?
        .extract()?;
    Ok(EventBranch { condition, event_id })
}

/// Deserialize age configs from Python config dict
pub fn deserialize_ages(config: &Bound<'_, PyDict>) -> pyo3::PyResult<HashMap<i32, AgeConfig>> {
    let ages_dict = config
        .get_item("ages")?
        .ok_or_else(|| LifeRestartError::DeserializationError("ages not found".to_string()))?;

    let ages_dict: Bound<'_, PyDict> = ages_dict.extract()?;
    let mut ages = HashMap::new();

    for (key, value) in ages_dict.iter() {
        // Support both string and integer keys
        let age: i32 = if let Ok(age) = key.extract::<i32>() {
            age
        } else {
            let key_str: String = key.extract()?;
            key_str.parse().map_err(|_| {
                pyo3::exceptions::PyValueError::new_err(format!("Invalid age: {}", key_str))
            })?
        };
        let age_config = extract_age(&value)?;
        ages.insert(age, age_config);
    }

    Ok(ages)
}

fn extract_age(obj: &Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<AgeConfig> {
    let age: i32 = get_attr(obj, "age")?.extract()?;

    // Extract talents list
    let talents = if let Some(talents_obj) = get_attr_opt(obj, "talents") {
        if !talents_obj.is_none() {
            let list: Vec<i32> = talents_obj.extract()?;
            if list.is_empty() { None } else { Some(list) }
        } else {
            None
        }
    } else {
        None
    };

    // Extract events list [(event_id, weight), ...]
    // Support both tuples and lists for each event entry
    let events = if let Some(events_obj) = get_attr_opt(obj, "events") {
        if !events_obj.is_none() {
            let list: Bound<'_, PyList> = events_obj.extract()?;
            let mut events_vec = Vec::new();
            for item in list.iter() {
                // Try to extract as tuple first, then as list
                let (event_id, weight): (i32, f64) = if let Ok(tuple) = item.extract::<(i32, f64)>() {
                    tuple
                } else {
                    // Try extracting as a list [event_id, weight]
                    let inner_list: Vec<pyo3::PyObject> = item.extract()?;
                    if inner_list.len() >= 2 {
                        let py = item.py();
                        let event_id: i32 = inner_list[0].bind(py).extract()?;
                        let weight: f64 = inner_list[1].bind(py).extract()?;
                        (event_id, weight)
                    } else {
                        return Err(pyo3::exceptions::PyValueError::new_err(
                            "Event entry must have at least 2 elements [event_id, weight]"
                        ));
                    }
                };
                events_vec.push((event_id, weight));
            }
            if events_vec.is_empty() { None } else { Some(events_vec) }
        } else {
            None
        }
    } else {
        None
    };

    Ok(AgeConfig { age, talents, events })
}

/// Deserialize achievements from Python config dict
pub fn deserialize_achievements(
    config: &Bound<'_, PyDict>,
) -> pyo3::PyResult<HashMap<i32, AchievementConfig>> {
    let achievements_dict = config.get_item("achievements")?.ok_or_else(|| {
        LifeRestartError::DeserializationError("achievements not found".to_string())
    })?;

    let achievements_dict: Bound<'_, PyDict> = achievements_dict.extract()?;
    let mut achievements = HashMap::new();

    for (key, value) in achievements_dict.iter() {
        // Support both string and integer keys
        let id: i32 = if let Ok(id) = key.extract::<i32>() {
            id
        } else {
            let key_str: String = key.extract()?;
            key_str.parse().map_err(|_| {
                pyo3::exceptions::PyValueError::new_err(format!("Invalid achievement id: {}", key_str))
            })?
        };
        let achievement = extract_achievement(&value)?;
        achievements.insert(id, achievement);
    }

    Ok(achievements)
}

fn extract_achievement(obj: &Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<AchievementConfig> {
    let id: i32 = get_attr(obj, "id")?.extract()?;
    let name: String = get_attr(obj, "name")?.extract()?;
    let description: String = get_attr(obj, "description")?.extract()?;
    let grade: i32 = get_attr_opt(obj, "grade").and_then(|v| v.extract().ok()).unwrap_or(0);
    let opportunity: String = get_attr(obj, "opportunity")?.extract()?;
    let condition: String = get_attr(obj, "condition")?.extract()?;

    Ok(AchievementConfig {
        id,
        name,
        description,
        grade,
        opportunity,
        condition,
    })
}

/// Deserialize judge config from Python config dict
pub fn deserialize_judge_config(
    config: &Bound<'_, PyDict>,
) -> pyo3::PyResult<HashMap<String, Vec<JudgeLevel>>> {
    let judge_dict = config
        .get_item("judge")?
        .ok_or_else(|| LifeRestartError::DeserializationError("judge not found".to_string()))?;

    let judge_dict: Bound<'_, PyDict> = judge_dict.extract()?;
    let mut judge_config = HashMap::new();

    for (key, value) in judge_dict.iter() {
        let prop: String = key.extract()?;
        let levels_list: Bound<'_, PyList> = value.extract()?;
        let mut levels = Vec::new();
        for item in levels_list.iter() {
            levels.push(extract_judge_level(&item)?);
        }
        // Sort by min descending for O(1) early-return lookup
        levels.sort_by(|a, b| b.min.cmp(&a.min));
        judge_config.insert(prop, levels);
    }

    Ok(judge_config)
}

fn extract_judge_level(obj: &Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<JudgeLevel> {
    let min: i32 = get_attr(obj, "min")?.extract()?;
    let grade: i32 = get_attr(obj, "grade")?.extract()?;
    let text: String = get_attr(obj, "text")?.extract()?;

    Ok(JudgeLevel { min, grade, text })
}
