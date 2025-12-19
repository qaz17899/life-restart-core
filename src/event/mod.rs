//! Event processing module

mod processor;
pub mod selector;

#[cfg(test)]
mod property_tests;

pub use processor::*;
pub use selector::*;
