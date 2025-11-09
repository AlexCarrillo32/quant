//! Yahoo Finance data provider
//!
//! Fetches real market data from Yahoo Finance API

use super::{DataProvider, DataProviderConfig};
use crate::types::*;
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use time::OffsetDateTime;
use yahoo_finance_api as yahoo;

/// Yahoo Finance data provider
pub struct YahooFinanceProvider {
    provider: yahoo::YahooConnector,
    config: DataProviderConfig,
}

impl YahooFinanceProvider {
    /// Create new Yahoo Finance provider
    pub fn new() -> Self {
        YahooFinanceProvider {
            provider: yahoo::YahooConnector::new().unwrap(),
            config: DataProviderConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: DataProviderConfig) -> Self {
        YahooFinanceProvider {
            provider: yahoo::YahooConnector::new().unwrap(),
            config,
        }
    }

    /// Fetch VIX (volatility index) for market sentiment
    async fn fetch_vix(&self) -> Result<f64> {
        let response = self
            .provider
            .get_latest_quotes("^VIX", "1d")
            .await
            .context("Failed to fetch VIX")?;

        let quote = response.last_quote().context("No VIX quote available")?;

        Ok(quote.close)
    }

    /// Convert Yahoo quote to our MarketData
    fn convert_quote(
        &self,
        symbol: Symbol,
        yahoo_quote: &yahoo::Quote,
        vix: Option<f64>,
    ) -> Result<MarketData> {
        // Create quote (bid/ask)
        let quote = Quote {
            bid: Price::new(yahoo_quote.close * 0.999)?, // Approximate bid
            ask: Price::new(yahoo_quote.close * 1.001)?, // Approximate ask
            bid_size: Quantity::buy(1000),               // Yahoo doesn't provide
            ask_size: Quantity::sell(1000),              // Yahoo doesn't provide
            timestamp: DateTime::from_timestamp(yahoo_quote.timestamp as i64, 0)
                .unwrap_or_else(Utc::now),
        };

        // Build market data
        let market_data = MarketData {
            symbol,
            quote,
            last_price: Price::new(yahoo_quote.close)?,
            volume: yahoo_quote.volume,
            timestamp: DateTime::from_timestamp(yahoo_quote.timestamp as i64, 0)
                .unwrap_or_else(Utc::now),
            open: Some(Price::new(yahoo_quote.open)?),
            high: Some(Price::new(yahoo_quote.high)?),
            low: Some(Price::new(yahoo_quote.low)?),
            prev_close: Some(Price::new(yahoo_quote.adjclose)?),
            vix,
            put_call_ratio: None, // Yahoo doesn't provide this
        };

        Ok(market_data)
    }
}

impl Default for YahooFinanceProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DataProvider for YahooFinanceProvider {
    async fn get_quote(&self, symbol: &Symbol) -> Result<MarketData> {
        // Fetch quote from Yahoo
        let response = self
            .provider
            .get_latest_quotes(symbol.as_str(), "1d")
            .await
            .context(format!("Failed to fetch quote for {}", symbol))?;

        let yahoo_quote = response
            .last_quote()
            .context(format!("No quote available for {}", symbol))?;

        // Fetch VIX for market sentiment
        let vix = self.fetch_vix().await.ok();

        // Convert to our format
        self.convert_quote(symbol.clone(), &yahoo_quote, vix)
    }

    async fn get_quotes(&self, symbols: &[Symbol]) -> Result<MarketSnapshot> {
        let mut snapshot = MarketSnapshot::new();

        // Fetch VIX once for all symbols
        let vix = self.fetch_vix().await.ok();

        // Fetch quotes sequentially (Yahoo API has rate limits anyway)
        for symbol in symbols {
            match self.provider.get_latest_quotes(symbol.as_str(), "1d").await {
                Ok(response) => match response.last_quote() {
                    Ok(yahoo_quote) => {
                        match self.convert_quote(symbol.clone(), &yahoo_quote, vix) {
                            Ok(market_data) => {
                                snapshot.insert(symbol.clone(), market_data);
                            }
                            Err(e) => {
                                tracing::warn!("Failed to convert quote for {}: {}", symbol, e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("No quote for {}: {}", symbol, e);
                    }
                },
                Err(e) => {
                    tracing::warn!("Failed to fetch {}: {}", symbol, e);
                }
            }
        }

        if snapshot.is_empty() {
            return Err(anyhow!("Failed to fetch any quotes"));
        }

        Ok(snapshot)
    }

    async fn get_historical(
        &self,
        symbol: &Symbol,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<MarketData>> {
        // Convert chrono DateTime to time OffsetDateTime
        let start_ts = OffsetDateTime::from_unix_timestamp(start.timestamp())
            .context("Invalid start timestamp")?;
        let end_ts = OffsetDateTime::from_unix_timestamp(end.timestamp())
            .context("Invalid end timestamp")?;

        // Fetch historical data
        let response = self
            .provider
            .get_quote_history_interval(symbol.as_str(), start_ts, end_ts, "1d")
            .await
            .context(format!("Failed to fetch historical data for {}", symbol))?;

        let quotes = response
            .quotes()
            .context("Failed to parse historical quotes")?;

        // Convert to our format
        let mut result = Vec::new();
        for yahoo_quote in quotes {
            match self.convert_quote(symbol.clone(), &yahoo_quote, None) {
                Ok(market_data) => result.push(market_data),
                Err(e) => {
                    tracing::warn!("Failed to convert historical quote: {}", e);
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[tokio::test]
    #[ignore] // Requires internet connection
    async fn test_fetch_real_quote() {
        let provider = YahooFinanceProvider::new();
        let symbol = Symbol::new("AAPL").unwrap();

        let result = provider.get_quote(&symbol).await;
        assert!(result.is_ok(), "Failed to fetch AAPL: {:?}", result.err());

        let market_data = result.unwrap();
        assert_eq!(market_data.symbol, symbol);
        assert!(market_data.last_price.value() > 0.0);
        assert!(market_data.volume > 0);

        println!("AAPL Quote: ${:.2}", market_data.last_price.value());
        println!("Volume: {}", market_data.volume);
        if let Some(vix) = market_data.vix {
            println!("VIX: {:.2}", vix);
        }
    }

    #[tokio::test]
    #[ignore] // Requires internet connection
    async fn test_fetch_multiple_quotes() {
        let provider = YahooFinanceProvider::new();
        let symbols = vec![
            Symbol::new("AAPL").unwrap(),
            Symbol::new("GOOGL").unwrap(),
            Symbol::new("MSFT").unwrap(),
        ];

        let result = provider.get_quotes(&symbols).await;
        assert!(result.is_ok(), "Failed to fetch quotes: {:?}", result.err());

        let snapshot = result.unwrap();
        assert!(snapshot.len() >= 2, "Expected at least 2 quotes");

        for symbol in symbols {
            if let Some(data) = snapshot.get(&symbol) {
                println!(
                    "{}: ${:.2} (vol: {})",
                    symbol,
                    data.last_price.value(),
                    data.volume
                );
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires internet connection
    async fn test_fetch_historical_data() {
        let provider = YahooFinanceProvider::new();
        let symbol = Symbol::new("SPY").unwrap();

        let end = Utc::now();
        let start = end - Duration::days(30);

        let result = provider.get_historical(&symbol, start, end).await;
        assert!(
            result.is_ok(),
            "Failed to fetch historical: {:?}",
            result.err()
        );

        let history = result.unwrap();
        assert!(!history.is_empty(), "Expected historical data");
        assert!(history.len() <= 30, "Expected ~30 days of data");

        println!("Fetched {} days of {} history", history.len(), symbol);
        if let Some(first) = history.first() {
            println!("First: ${:.2}", first.last_price.value());
        }
        if let Some(last) = history.last() {
            println!("Last: ${:.2}", last.last_price.value());
        }
    }

    #[tokio::test]
    #[ignore] // Requires internet connection
    async fn test_vix_fetch() {
        let provider = YahooFinanceProvider::new();
        let vix = provider.fetch_vix().await;

        assert!(vix.is_ok(), "Failed to fetch VIX: {:?}", vix.err());

        let vix_value = vix.unwrap();
        assert!(vix_value > 0.0);
        assert!(vix_value < 100.0); // VIX typically 10-80

        println!("Current VIX: {:.2}", vix_value);
    }
}
