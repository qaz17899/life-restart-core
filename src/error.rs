//! Error types for the life restart core engine

use pyo3::exceptions::{PyKeyError, PyRuntimeError, PyValueError};
use pyo3::PyErr;
use thiserror::Error;

/// Main error type for the life restart core engine
#[derive(Error, Debug)]
pub enum LifeRestartError {
    #[error("Invalid condition: {0}")]
    InvalidCondition(String),

    #[error("Talent not found: {0}")]
    TalentNotFound(i32),

    #[error("Event not found: {0}")]
    EventNotFound(i32),

    #[error("Age config not found: {0}")]
    AgeConfigNotFound(i32),

    #[error("Achievement not found: {0}")]
    AchievementNotFound(i32),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Invalid property: {0}")]
    InvalidProperty(String),

    #[error("Simulation error: {0}")]
    SimulationError(String),
}

impl From<LifeRestartError> for PyErr {
    fn from(err: LifeRestartError) -> PyErr {
        match err {
            LifeRestartError::InvalidCondition(msg) => {
                PyValueError::new_err(format!("Invalid condition: {}", msg))
            }
            LifeRestartError::TalentNotFound(id) => {
                PyKeyError::new_err(format!("Talent not found: {}", id))
            }
            LifeRestartError::EventNotFound(id) => {
                PyKeyError::new_err(format!("Event not found: {}", id))
            }
            LifeRestartError::AgeConfigNotFound(id) => {
                PyKeyError::new_err(format!("Age config not found: {}", id))
            }
            LifeRestartError::AchievementNotFound(id) => {
                PyKeyError::new_err(format!("Achievement not found: {}", id))
            }
            LifeRestartError::DeserializationError(msg) => {
                PyValueError::new_err(format!("Deserialization error: {}", msg))
            }
            LifeRestartError::InvalidProperty(msg) => {
                PyValueError::new_err(format!("Invalid property: {}", msg))
            }
            LifeRestartError::SimulationError(msg) => {
                PyRuntimeError::new_err(format!("Simulation error: {}", msg))
            }
        }
    }
}

/// Result type alias for the life restart core engine
pub type Result<T> = std::result::Result<T, LifeRestartError>;
