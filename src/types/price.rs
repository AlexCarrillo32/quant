//! Price and quantity types with validation

use super::ValidationError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

/// Price in dollars (newtype pattern for type safety)
///
/// Guarantees:
/// - Always positive
/// - Finite (not NaN or infinity)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Price(f64);

impl Price {
    /// Create a new price
    ///
    /// # Errors
    ///
    /// Returns `ValidationError` if price is negative, NaN, or infinite
    pub fn new(value: f64) -> Result<Self, ValidationError> {
        if value <= 0.0 {
            return Err(ValidationError::InvalidPrice(value));
        }

        if !value.is_finite() {
            return Err(ValidationError::InvalidPrice(value));
        }

        Ok(Price(value))
    }

    /// Create price without validation (use with caution)
    ///
    /// # Safety
    ///
    /// Caller must ensure value is positive and finite
    #[inline(always)]
    pub const unsafe fn new_unchecked(value: f64) -> Self {
        Price(value)
    }

    /// Get the raw value
    #[inline(always)]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Calculate percentage change
    pub fn percent_change(&self, other: Price) -> f64 {
        ((other.0 - self.0) / self.0) * 100.0
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${:.2}", self.0)
    }
}

// Arithmetic operations
impl Add for Price {
    type Output = Price;

    fn add(self, other: Price) -> Price {
        Price(self.0 + other.0)
    }
}

impl Sub for Price {
    type Output = Price;

    fn sub(self, other: Price) -> Price {
        Price(self.0 - other.0)
    }
}

impl Mul<f64> for Price {
    type Output = Price;

    fn mul(self, scalar: f64) -> Price {
        Price(self.0 * scalar)
    }
}

impl Div<f64> for Price {
    type Output = Price;

    fn div(self, scalar: f64) -> Price {
        Price(self.0 / scalar)
    }
}

/// Quantity (signed integer for buy/sell)
///
/// Positive = Buy
/// Negative = Sell
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Quantity(i64);

impl Quantity {
    /// Create a new quantity
    pub fn new(value: i64) -> Result<Self, ValidationError> {
        if value == 0 {
            return Err(ValidationError::InvalidQuantity(value));
        }
        Ok(Quantity(value))
    }

    /// Create positive (buy) quantity
    pub fn buy(value: u64) -> Self {
        Quantity(value as i64)
    }

    /// Create negative (sell) quantity
    pub fn sell(value: u64) -> Self {
        Quantity(-(value as i64))
    }

    #[inline(always)]
    pub fn value(&self) -> i64 {
        self.0
    }

    #[inline(always)]
    pub fn abs(&self) -> u64 {
        self.0.unsigned_abs()
    }

    #[inline(always)]
    pub fn is_buy(&self) -> bool {
        self.0 > 0
    }

    #[inline(always)]
    pub fn is_sell(&self) -> bool {
        self.0 < 0
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_buy() {
            write!(f, "+{}", self.0)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_creation() {
        let price = Price::new(100.50).unwrap();
        assert_eq!(price.value(), 100.50);
    }

    #[test]
    fn test_negative_price() {
        let result = Price::new(-10.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_nan_price() {
        let result = Price::new(f64::NAN);
        assert!(result.is_err());
    }

    #[test]
    fn test_price_arithmetic() {
        let p1 = Price::new(100.0).unwrap();
        let p2 = Price::new(50.0).unwrap();

        let sum = p1 + p2;
        assert_eq!(sum.value(), 150.0);

        let diff = p1 - p2;
        assert_eq!(diff.value(), 50.0);
    }

    #[test]
    fn test_percent_change() {
        let p1 = Price::new(100.0).unwrap();
        let p2 = Price::new(110.0).unwrap();

        let change = p1.percent_change(p2);
        assert_eq!(change, 10.0);
    }

    #[test]
    fn test_quantity_buy_sell() {
        let buy = Quantity::buy(100);
        assert!(buy.is_buy());
        assert!(!buy.is_sell());
        assert_eq!(buy.value(), 100);

        let sell = Quantity::sell(100);
        assert!(sell.is_sell());
        assert!(!sell.is_buy());
        assert_eq!(sell.value(), -100);
    }

    #[test]
    fn test_zero_quantity() {
        let result = Quantity::new(0);
        assert!(result.is_err());
    }
}
