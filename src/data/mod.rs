//! Market data providers
//!
//! Abstracts different data sources (Yahoo Finance, Alpaca, etc.)

use crate::types::{MarketData, MarketSnapshot, Symbol};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

pub mod yahoo;

pub use yahoo::YahooFinanceProvider;

/// Market data provider trait
#[async_trait]
pub trait DataProvider: Send + Sync {
    /// Get latest quote for a symbol
    async fn get_quote(&self, symbol: &Symbol) -> Result<MarketData>;

    /// Get quotes for multiple symbols
    async fn get_quotes(&self, symbols: &[Symbol]) -> Result<MarketSnapshot>;

    /// Get historical data
    async fn get_historical(
        &self,
        symbol: &Symbol,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<MarketData>>;
}

/// Data provider configuration
#[derive(Debug, Clone)]
pub struct DataProviderConfig {
    pub cache_ttl_seconds: u64,
    pub retry_attempts: u32,
    pub timeout_seconds: u64,
}

impl Default for DataProviderConfig {
    fn default() -> Self {
        DataProviderConfig {
            cache_ttl_seconds: 60, // 1 minute cache
            retry_attempts: 3,     // Retry 3 times
            timeout_seconds: 10,   // 10 second timeout
        }
    }
}
