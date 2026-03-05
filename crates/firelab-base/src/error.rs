//! Error types for the BehavePlus library.

use std::fmt;

/// Top-level error type for BehavePlus calculations.
#[derive(Debug, Clone)]
pub enum BehaveError {
    /// An invalid fuel model number was provided.
    InvalidFuelModel(i32),
    /// A required input value is missing or out of range.
    InvalidInput(String),
    /// A unit conversion is not supported.
    UnitConversionError(String),
    /// Generic calculation error.
    CalculationError(String),
}

impl fmt::Display for BehaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFuelModel(n) => write!(f, "invalid fuel model number: {n}"),
            Self::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
            Self::UnitConversionError(msg) => write!(f, "unit conversion error: {msg}"),
            Self::CalculationError(msg) => write!(f, "calculation error: {msg}"),
        }
    }
}

impl std::error::Error for BehaveError {}

#[cfg(test)]
mod tests {}
