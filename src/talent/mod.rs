//! Talent processing module

mod processor;
mod replacer;

#[cfg(test)]
mod property_tests;

pub use processor::*;
pub use replacer::*;
