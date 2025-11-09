//! Market data structures

use super::{Price, Quantity, Symbol};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Order book price level
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: Price,
    pub quantity: Quantity,
}

/// Quote (bid/ask)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Quote {
    pub bid: Price,
    pub ask: Price,
    pub bid_size: Quantity,
    pub ask_size: Quantity,
    pub timestamp: DateTime<Utc>,
}

impl Quote {
    /// Calculate bid-ask spread
    pub fn spread(&self) -> f64 {
        self.ask.value() - self.bid.value()
    }

    /// Calculate mid price
    pub fn mid(&self) -> Price {
        Price::new((self.bid.value() + self.ask.value()) / 2.0)
            .expect("Mid price calculation failed")
    }

    /// Calculate spread as percentage of mid
    pub fn spread_pct(&self) -> f64 {
        (self.spread() / self.mid().value()) * 100.0
    }
}

/// Market data snapshot for a symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub symbol: Symbol,
    pub quote: Quote,
    pub last_price: Price,
    pub volume: u64,
    pub timestamp: DateTime<Utc>,

    // Optional fields
    pub open: Option<Price>,
    pub high: Option<Price>,
    pub low: Option<Price>,
    pub prev_close: Option<Price>,

    // For behavioral analysis
    pub vix: Option<f64>,            // Volatility index
    pub put_call_ratio: Option<f64>, // Options sentiment
}

impl MarketData {
    /// Calculate intraday change percentage
    pub fn intraday_change_pct(&self) -> Option<f64> {
        let prev_close = self.prev_close?;
        Some(prev_close.percent_change(self.last_price))
    }

    /// Check if price is at session high
    pub fn at_high(&self) -> bool {
        if let Some(high) = self.high {
            (self.last_price.value() - high.value()).abs() < 0.01
        } else {
            false
        }
    }

    /// Check if price is at session low
    pub fn at_low(&self) -> bool {
        if let Some(low) = self.low {
            (self.last_price.value() - low.value()).abs() < 0.01
        } else {
            false
        }
    }
}

/// Market snapshot for multiple symbols
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarketSnapshot {
    pub data: HashMap<Symbol, MarketData>,
    pub timestamp: DateTime<Utc>,
}

impl MarketSnapshot {
    pub fn new() -> Self {
        MarketSnapshot {
            data: HashMap::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn insert(&mut self, symbol: Symbol, data: MarketData) {
        self.data.insert(symbol, data);
    }

    pub fn get(&self, symbol: &Symbol) -> Option<&MarketData> {
        self.data.get(symbol)
    }

    pub fn symbols(&self) -> Vec<&Symbol> {
        self.data.keys().collect()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_quote() -> Quote {
        Quote {
            bid: Price::new(100.0).unwrap(),
            ask: Price::new(100.10).unwrap(),
            bid_size: Quantity::buy(1000),
            ask_size: Quantity::sell(1000),
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_quote_spread() {
        let quote = create_test_quote();
        let spread = quote.spread();
        assert!(
            (spread - 0.10).abs() < 0.001,
            "Expected ~0.10, got {}",
            spread
        );
    }

    #[test]
    fn test_quote_mid() {
        let quote = create_test_quote();
        assert_eq!(quote.mid().value(), 100.05);
    }

    #[test]
    fn test_quote_spread_pct() {
        let quote = create_test_quote();
        let spread_pct = quote.spread_pct();
        assert!((spread_pct - 0.0999).abs() < 0.001); // ~0.1%
    }

    #[test]
    fn test_market_snapshot() {
        let mut snapshot = MarketSnapshot::new();

        let symbol = Symbol::new("AAPL").unwrap();
        let data = MarketData {
            symbol: symbol.clone(),
            quote: create_test_quote(),
            last_price: Price::new(100.05).unwrap(),
            volume: 1_000_000,
            timestamp: Utc::now(),
            open: Some(Price::new(99.0).unwrap()),
            high: Some(Price::new(101.0).unwrap()),
            low: Some(Price::new(98.0).unwrap()),
            prev_close: Some(Price::new(99.5).unwrap()),
            vix: None,
            put_call_ratio: None,
        };

        snapshot.insert(symbol.clone(), data);

        assert_eq!(snapshot.len(), 1);
        assert!(snapshot.get(&symbol).is_some());
    }

    #[test]
    fn test_intraday_change() {
        let symbol = Symbol::new("AAPL").unwrap();
        let data = MarketData {
            symbol,
            quote: create_test_quote(),
            last_price: Price::new(105.0).unwrap(),
            volume: 1_000_000,
            timestamp: Utc::now(),
            open: None,
            high: None,
            low: None,
            prev_close: Some(Price::new(100.0).unwrap()),
            vix: None,
            put_call_ratio: None,
        };

        let change = data.intraday_change_pct().unwrap();
        assert_eq!(change, 5.0); // 5% increase
    }
}
