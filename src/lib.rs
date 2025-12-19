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
use crate::simulator::{default_emoji_map, GameSession, SimulationEngine};
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use pyo3::types::PyDict;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

// ============================================================================
// Cached Configuration
// ============================================================================

/// Cached configuration containing engine and emoji map
struct CachedConfig {
    engine: SimulationEngine,
    emoji_map: Arc<HashMap<i32, String>>,
}

/// Global cached configuration
static CACHED_CONFIG: OnceCell<Arc<RwLock<CachedConfig>>> = OnceCell::new();

// ============================================================================
// Helper Functions
// ============================================================================

/// Deserialize emoji map from Python dict
fn deserialize_emoji_map(dict: &Bound<'_, PyDict>) -> PyResult<HashMap<i32, String>> {
    let mut map = HashMap::new();
    for (key, value) in dict.iter() {
        let grade: i32 = key.extract()?;
        let emoji: String = value.extract()?;
        map.insert(grade, emoji);
    }
    Ok(map)
}

// ============================================================================
// Python Functions
// ============================================================================

/// Initialize the game configuration (call once at startup)
///
/// This caches the configuration in Rust memory, eliminating the need to
/// deserialize it on every simulation call. Call this once when your bot starts.
///
/// # Arguments
/// * `config` - Game configuration containing talents, events, ages, achievements
/// * `emoji_map` - Optional emoji map for grade-to-emoji conversion (default: {0: "⚪", 1: "🔵", 2: "🟣", 3: "🟠"})
#[pyfunction]
#[pyo3(signature = (config, emoji_map=None))]
fn init_config(config: &Bound<'_, PyDict>, emoji_map: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
    let talents = config::deserialize_talents(config)?;
    let events = config::deserialize_events(config)?;
    let ages = config::deserialize_ages(config)?;
    let achievements = config::deserialize_achievements(config)?;
    let judge_config = config::deserialize_judge_config(config)?;

    let engine = SimulationEngine::new(talents, events, ages, achievements, judge_config);

    // Parse emoji map or use defaults
    let emoji = if let Some(map) = emoji_map {
        deserialize_emoji_map(map)?
    } else {
        default_emoji_map()
    };

    let cached = CachedConfig {
        engine,
        emoji_map: Arc::new(emoji),
    };

    // If already initialized, update the config
    if let Some(existing) = CACHED_CONFIG.get() {
        let mut guard = existing.write();
        *guard = cached;
    } else {
        let _ = CACHED_CONFIG.set(Arc::new(RwLock::new(cached)));
    }

    Ok(())
}

/// Check if config is initialized
#[pyfunction]
fn is_config_initialized() -> bool {
    CACHED_CONFIG.get().is_some()
}

/// Simulate a complete life trajectory (fast version using cached config)
///
/// # Arguments
/// * `talent_ids` - List of selected talent IDs
/// * `properties` - Initial property allocation {CHR, INT, STR, MNY}
/// * `achieved_ids` - Set of already achieved achievement IDs
///
/// # Returns
/// A GameSession object containing the simulation results
///
/// # Raises
/// RuntimeError if `init_config` was not called first
#[pyfunction]
fn simulate_full_life(
    talent_ids: Vec<i32>,
    properties: HashMap<String, i32>,
    achieved_ids: HashSet<i32>,
) -> PyResult<GameSession> {
    // Get cached config
    let config_arc = CACHED_CONFIG
        .get()
        .ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Config not initialized. Call init_config() first.",
            )
        })?
        .clone();

    let config = config_arc.read();

    // Run simulation
    let result = config
        .engine
        .simulate(&talent_ids, &properties, &achieved_ids)
        .map_err(LifeRestartError::from)?;

    // Wrap in GameSession with pre-rendering
    Ok(GameSession::new(result, config.emoji_map.clone()))
}

/// Simulate a complete life trajectory asynchronously
///
/// This function runs the simulation in a background thread using Tokio's
/// spawn_blocking, allowing Python's asyncio event loop to remain responsive.
/// The GIL is automatically released during the CPU-intensive simulation.
///
/// # Arguments
/// * `py` - Python interpreter token
/// * `talent_ids` - List of selected talent IDs
/// * `properties` - Initial property allocation {CHR, INT, STR, MNY}
/// * `achieved_ids` - Set of already achieved achievement IDs
///
/// # Returns
/// A Python awaitable that resolves to a GameSession object
///
/// # Raises
/// RuntimeError if `init_config` was not called first
///
/// # Example (Python)
/// ```python
/// session = await simulate_async([1, 2, 3], {"CHR": 5, "INT": 5}, set())
/// print(session.total_years)
/// ```
#[pyfunction]
fn simulate_async<'py>(
    py: Python<'py>,
    talent_ids: Vec<i32>,
    properties: HashMap<String, i32>,
    achieved_ids: HashSet<i32>,
) -> PyResult<Bound<'py, PyAny>> {
    // Get cached config before entering async context
    let config_arc = CACHED_CONFIG
        .get()
        .ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Config not initialized. Call init_config() first.",
            )
        })?
        .clone();

    // Use pyo3-async-runtimes to convert Rust Future to Python awaitable
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        // Run CPU-intensive simulation in a blocking thread
        // GIL is automatically released in spawn_blocking (pyo3-async-runtimes 0.27)
        let result = tokio::task::spawn_blocking(move || {
            let config = config_arc.read();
            
            // Run simulation
            let sim_result = config
                .engine
                .simulate(&talent_ids, &properties, &achieved_ids)
                .map_err(LifeRestartError::from)?;

            // Wrap in GameSession with pre-rendering
            Ok::<GameSession, PyErr>(GameSession::new(sim_result, config.emoji_map.clone()))
        })
        .await
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Simulation task panicked: {}",
                e
            ))
        })??;

        Ok(result)
    })
}

// ============================================================================
// Python Module Definition
// ============================================================================

/// Python module definition
#[pymodule]
fn life_restart_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(init_config, m)?)?;
    m.add_function(wrap_pyfunction!(is_config_initialized, m)?)?;
    m.add_function(wrap_pyfunction!(simulate_full_life, m)?)?;
    m.add_function(wrap_pyfunction!(simulate_async, m)?)?;
    m.add_class::<GameSession>()?;
    Ok(())
}
