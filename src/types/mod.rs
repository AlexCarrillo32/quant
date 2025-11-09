//! Core domain types for trading engine
//!
//! These types enforce invariants at compile time and runtime.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

mod market_data;
mod price;
mod signal;

pub use market_data::{MarketData, MarketSnapshot, OrderBookLevel, Quote};
pub use price::{Price, Quantity};
pub use signal::{Confidence, Signal, SignalAction};

/// Symbol identifier (e.g., "AAPL", "GOOGL")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol(String);

impl Symbol {
    pub fn new(s: impl Into<String>) -> Result<Self, ValidationError> {
        let s = s.into();

        // Validate: non-empty, uppercase alphanumeric
        if s.is_empty() {
            return Err(ValidationError::EmptySymbol);
        }

        if s.len() > 10 {
            return Err(ValidationError::SymbolTooLong(s.len()));
        }

        if !s.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(ValidationError::InvalidSymbolCharacters);
        }

        Ok(Symbol(s.to_uppercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Validation errors
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Symbol cannot be empty")]
    EmptySymbol,

    #[error("Symbol too long: {0} characters (max 10)")]
    SymbolTooLong(usize),

    #[error("Symbol contains invalid characters (alphanumeric only)")]
    InvalidSymbolCharacters,

    #[error("Price must be positive, got {0}")]
    InvalidPrice(f64),

    #[error("Quantity must be positive, got {0}")]
    InvalidQuantity(i64),

    #[error("Confidence must be between 0 and 1, got {0}")]
    InvalidConfidence(f64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_symbol() {
        let sym = Symbol::new("AAPL").unwrap();
        assert_eq!(sym.as_str(), "AAPL");
    }

    #[test]
    fn test_symbol_uppercase() {
        let sym = Symbol::new("aapl").unwrap();
        assert_eq!(sym.as_str(), "AAPL");
    }

    #[test]
    fn test_empty_symbol() {
        let result = Symbol::new("");
        assert!(result.is_err());
    }

    #[test]
    fn test_symbol_too_long() {
        let result = Symbol::new("VERYLONGSYMBOL");
        assert!(result.is_err());
    }
}
