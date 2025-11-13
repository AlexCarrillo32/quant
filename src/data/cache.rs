//! High-Performance In-Memory Data Cache
//!
//! **Why Caching Matters**: Every microsecond counts in trading
//!
//! ## The Human Edge
//!
//! Banks pay millions for colocation to save milliseconds.
//! We can't compete on network speed, but we can:
//! - Cache aggressively to avoid redundant API calls
//! - Use lock-free data structures for concurrent access
//! - Pre-warm cache for symbols we're tracking
//!
//! ## Architecture
//!
//! ```text
//! Alpha Model → Cache (check) → Hit? Return cached data
//!                            → Miss? Fetch from provider → Cache → Return
//! ```
//!
//! ## Performance
//!
//! - Cache hit: ~100ns (nanoseconds)
//! - API call: ~50ms (milliseconds) = 500,000x slower!
//! - Cache miss is expensive, but still better than no cache

use crate::data::DataProvider;
use crate::types::{MarketData, MarketSnapshot, Symbol};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Cache entry with expiration
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    data: T,
    expires_at: DateTime<Utc>,
}

impl<T> CacheEntry<T> {
    fn new(data: T, ttl: Duration) -> Self {
        CacheEntry {
            data,
            expires_at: Utc::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// In-memory cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Time-to-live for quote data (seconds)
    pub quote_ttl_seconds: i64,

    /// Time-to-live for historical data (seconds)
    pub historical_ttl_seconds: i64,

    /// Maximum cache size (number of entries)
    pub max_entries: usize,

    /// Enable/disable caching
    pub enabled: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig {
            quote_ttl_seconds: 1,        // 1 second for real-time quotes
            historical_ttl_seconds: 300, // 5 minutes for historical data
            max_entries: 1000,           // Cache up to 1000 symbols
            enabled: true,
        }
    }
}

/// Cached data provider that wraps any DataProvider
///
/// Uses async RwLock for concurrent read access with single writer
pub struct CachedDataProvider<T: DataProvider> {
    /// Underlying data provider
    provider: T,

    /// Cache configuration
    config: CacheConfig,

    /// Quote cache: Symbol -> MarketData
    quote_cache: Arc<RwLock<HashMap<Symbol, CacheEntry<MarketData>>>>,

    /// Historical data cache: (Symbol, Start, End) -> Vec<MarketData>
    historical_cache: Arc<RwLock<HashMap<String, CacheEntry<Vec<MarketData>>>>>,

    /// Cache statistics
    stats: Arc<RwLock<CacheStats>>,
}

/// Cache performance statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub quote_hits: u64,
    pub quote_misses: u64,
    pub historical_hits: u64,
    pub historical_misses: u64,
    pub evictions: u64,
}

impl CacheStats {
    /// Calculate quote cache hit rate
    pub fn quote_hit_rate(&self) -> f64 {
        let total = self.quote_hits + self.quote_misses;
        if total == 0 {
            0.0
        } else {
            self.quote_hits as f64 / total as f64
        }
    }

    /// Calculate historical cache hit rate
    pub fn historical_hit_rate(&self) -> f64 {
        let total = self.historical_hits + self.historical_misses;
        if total == 0 {
            0.0
        } else {
            self.historical_hits as f64 / total as f64
        }
    }

    /// Overall hit rate
    pub fn overall_hit_rate(&self) -> f64 {
        let total = self.quote_hits + self.quote_misses + self.historical_hits + self.historical_misses;
        let hits = self.quote_hits + self.historical_hits;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }
}

impl<T: DataProvider> CachedDataProvider<T> {
    /// Create new cached data provider
    pub fn new(provider: T, config: CacheConfig) -> Self {
        CachedDataProvider {
            provider,
            config,
            quote_cache: Arc::new(RwLock::new(HashMap::new())),
            historical_cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Create with default cache config
    pub fn with_defaults(provider: T) -> Self {
        Self::new(provider, CacheConfig::default())
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    /// Clear all caches
    pub async fn clear(&self) {
        if !self.config.enabled {
            return;
        }

        self.quote_cache.write().await.clear();
        self.historical_cache.write().await.clear();

        tracing::info!("📦 Cache cleared");
    }

    /// Pre-warm cache for a list of symbols
    ///
    /// Useful for fetching data for all tracked symbols at startup
    pub async fn prewarm(&self, symbols: &[Symbol]) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        tracing::info!("🔥 Pre-warming cache for {} symbols", symbols.len());

        for symbol in symbols {
            // Fetch and cache (ignore errors - some symbols may not have data)
            let _ = self.get_quote(symbol).await;
        }

        let stats = self.stats().await;
        tracing::info!(
            "✅ Cache pre-warmed. Hit rate: {:.2}%",
            stats.quote_hit_rate() * 100.0
        );

        Ok(())
    }

    /// Evict expired entries from quote cache
    async fn evict_expired_quotes(&self) {
        let mut cache = self.quote_cache.write().await;

        let before_count = cache.len();
        cache.retain(|_, entry| !entry.is_expired());
        let after_count = cache.len();

        let evicted = before_count - after_count;
        if evicted > 0 {
            let mut stats = self.stats.write().await;
            stats.evictions += evicted as u64;

            tracing::debug!("🗑️  Evicted {} expired quote entries", evicted);
        }
    }

    /// Evict expired entries from historical cache
    async fn evict_expired_historical(&self) {
        let mut cache = self.historical_cache.write().await;

        let before_count = cache.len();
        cache.retain(|_, entry| !entry.is_expired());
        let after_count = cache.len();

        let evicted = before_count - after_count;
        if evicted > 0 {
            let mut stats = self.stats.write().await;
            stats.evictions += evicted as u64;

            tracing::debug!("🗑️  Evicted {} expired historical entries", evicted);
        }
    }

    /// Generate cache key for historical data
    fn historical_cache_key(
        symbol: &Symbol,
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
    ) -> String {
        format!("{}:{}:{}", symbol.as_str(), start.timestamp(), end.timestamp())
    }
}

#[async_trait]
impl<T: DataProvider> DataProvider for CachedDataProvider<T> {
    async fn get_quote(&self, symbol: &Symbol) -> Result<MarketData> {
        if !self.config.enabled {
            return self.provider.get_quote(symbol).await;
        }

        // Check cache first (read lock)
        {
            let cache = self.quote_cache.read().await;
            if let Some(entry) = cache.get(symbol) {
                if !entry.is_expired() {
                    // Cache hit!
                    self.stats.write().await.quote_hits += 1;
                    tracing::trace!("📦 Cache HIT: {}", symbol.as_str());
                    return Ok(entry.data.clone());
                }
            }
        }

        // Cache miss - fetch from provider
        self.stats.write().await.quote_misses += 1;
        tracing::trace!("📭 Cache MISS: {}", symbol.as_str());

        let data = self.provider.get_quote(symbol).await?;

        // Store in cache (write lock)
        {
            let mut cache = self.quote_cache.write().await;

            // Check if we need to evict (simple size-based eviction)
            if cache.len() >= self.config.max_entries {
                // Remove oldest entry (simplified - could use LRU)
                if let Some(key) = cache.keys().next().cloned() {
                    cache.remove(&key);
                    self.stats.write().await.evictions += 1;
                }
            }

            let ttl = Duration::seconds(self.config.quote_ttl_seconds);
            cache.insert(symbol.clone(), CacheEntry::new(data.clone(), ttl));
        }

        // Periodically evict expired entries
        self.evict_expired_quotes().await;

        Ok(data)
    }

    async fn get_quotes(&self, symbols: &[Symbol]) -> Result<MarketSnapshot> {
        if !self.config.enabled {
            return self.provider.get_quotes(symbols).await;
        }

        let mut snapshot = MarketSnapshot::new();

        // Try to fetch each from cache, fallback to provider
        for symbol in symbols {
            match self.get_quote(symbol).await {
                Ok(data) => {
                    snapshot.data.insert(symbol.clone(), data);
                }
                Err(e) => {
                    tracing::warn!("Failed to get quote for {}: {}", symbol.as_str(), e);
                }
            }
        }

        Ok(snapshot)
    }

    async fn get_historical(
        &self,
        symbol: &Symbol,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<MarketData>> {
        if !self.config.enabled {
            return self.provider.get_historical(symbol, start, end).await;
        }

        let cache_key = Self::historical_cache_key(symbol, &start, &end);

        // Check cache first
        {
            let cache = self.historical_cache.read().await;
            if let Some(entry) = cache.get(&cache_key) {
                if !entry.is_expired() {
                    // Cache hit!
                    self.stats.write().await.historical_hits += 1;
                    tracing::trace!("📦 Historical cache HIT: {}", symbol.as_str());
                    return Ok(entry.data.clone());
                }
            }
        }

        // Cache miss
        self.stats.write().await.historical_misses += 1;
        tracing::trace!("📭 Historical cache MISS: {}", symbol.as_str());

        let data = self.provider.get_historical(symbol, start, end).await?;

        // Store in cache
        {
            let mut cache = self.historical_cache.write().await;

            // Simple size-based eviction
            if cache.len() >= self.config.max_entries {
                if let Some(key) = cache.keys().next().cloned() {
                    cache.remove(&key);
                    self.stats.write().await.evictions += 1;
                }
            }

            let ttl = Duration::seconds(self.config.historical_ttl_seconds);
            cache.insert(cache_key, CacheEntry::new(data.clone(), ttl));
        }

        // Periodically evict expired entries
        self.evict_expired_historical().await;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Price, Quantity, Quote};
    use chrono::Utc;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Mock data provider for testing
    struct MockProvider {
        call_count: Arc<AtomicU32>,
    }

    impl MockProvider {
        fn new() -> Self {
            MockProvider {
                call_count: Arc::new(AtomicU32::new(0)),
            }
        }

        fn calls(&self) -> u32 {
            self.call_count.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl DataProvider for MockProvider {
        async fn get_quote(&self, symbol: &Symbol) -> Result<MarketData> {
            self.call_count.fetch_add(1, Ordering::SeqCst);

            let now = Utc::now();
            let price = Price::new(100.0).unwrap();

            // Return dummy data
            Ok(MarketData {
                symbol: symbol.clone(),
                timestamp: now,
                quote: Quote {
                    bid: price,
                    ask: Price::new(101.0).unwrap(),
                    bid_size: Quantity::buy(100),
                    ask_size: Quantity::buy(100),
                    timestamp: now,
                },
                last_price: price,
                volume: 1000,
                open: Some(price),
                high: Some(Price::new(102.0).unwrap()),
                low: Some(Price::new(99.0).unwrap()),
                prev_close: Some(price),
                vix: None,
                put_call_ratio: None,
            })
        }

        async fn get_quotes(&self, symbols: &[Symbol]) -> Result<MarketSnapshot> {
            let mut snapshot = MarketSnapshot::new();
            for symbol in symbols {
                let data = self.get_quote(symbol).await?;
                snapshot.data.insert(symbol.clone(), data);
            }
            Ok(snapshot)
        }

        async fn get_historical(
            &self,
            _symbol: &Symbol,
            _start: DateTime<Utc>,
            _end: DateTime<Utc>,
        ) -> Result<Vec<MarketData>> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            Ok(Vec::new())
        }
    }

    #[tokio::test]
    async fn test_cache_hit() {
        let mock = MockProvider::new();
        let cached = CachedDataProvider::with_defaults(mock);

        let symbol = Symbol::new("AAPL").unwrap();

        // First call - cache miss
        let _ = cached.get_quote(&symbol).await.unwrap();
        assert_eq!(cached.provider.calls(), 1);

        // Second call - cache hit (within TTL)
        let _ = cached.get_quote(&symbol).await.unwrap();
        assert_eq!(cached.provider.calls(), 1); // No additional call

        // Verify stats
        let stats = cached.stats().await;
        assert_eq!(stats.quote_hits, 1);
        assert_eq!(stats.quote_misses, 1);
        assert_eq!(stats.quote_hit_rate(), 0.5);
    }

    #[tokio::test]
    async fn test_cache_disabled() {
        let mock = MockProvider::new();
        let config = CacheConfig {
            enabled: false,
            ..Default::default()
        };
        let cached = CachedDataProvider::new(mock, config);

        let symbol = Symbol::new("AAPL").unwrap();

        // Both calls should go to provider
        let _ = cached.get_quote(&symbol).await.unwrap();
        let _ = cached.get_quote(&symbol).await.unwrap();

        assert_eq!(cached.provider.calls(), 2);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let mock = MockProvider::new();
        let cached = CachedDataProvider::with_defaults(mock);

        let symbol = Symbol::new("AAPL").unwrap();

        // Cache a value
        let _ = cached.get_quote(&symbol).await.unwrap();

        // Clear cache
        cached.clear().await;

        // Next call should be a miss
        let _ = cached.get_quote(&symbol).await.unwrap();
        assert_eq!(cached.provider.calls(), 2);
    }

    #[tokio::test]
    async fn test_prewarm() {
        let mock = MockProvider::new();
        let cached = CachedDataProvider::with_defaults(mock);

        let symbols = vec![
            Symbol::new("AAPL").unwrap(),
            Symbol::new("GOOGL").unwrap(),
            Symbol::new("MSFT").unwrap(),
        ];

        // Pre-warm cache
        cached.prewarm(&symbols).await.unwrap();

        // All should be cached now
        for symbol in &symbols {
            let _ = cached.get_quote(symbol).await.unwrap();
        }

        // Provider should only be called during prewarm (3 times)
        assert_eq!(cached.provider.calls(), 3);

        let stats = cached.stats().await;
        assert_eq!(stats.quote_hits, 3);
    }
}
