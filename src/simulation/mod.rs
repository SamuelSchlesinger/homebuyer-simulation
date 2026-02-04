//! Core simulation types and logic.

pub mod engine;
pub mod mortgage;
pub mod parameters;
pub mod results;

pub use engine::SimulationEngine;
pub use mortgage::MortgageCalculator;
pub use parameters::*;
pub use results::*;
