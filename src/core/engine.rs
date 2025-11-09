//! Trading Engine Core
//!
//! The main event loop that orchestrates:
//! 1. Fetching market data
//! 2. Running alpha models
//! 3. Aggregating signals
//! 4. Managing orders

use crate::alphas::BoxedAlpha;
use crate::core::order_manager::{OrderManager, OrderManagerConfig};
use crate::core::signal_aggregator::{AggregationStrategy, SignalAggregator};
use crate::data::DataProvider;
use crate::types::{MarketSnapshot, Signal, Symbol};
use anyhow::{Context, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{debug, error, info, warn};

/// Configuration for the trading engine
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// How often to fetch market data and run alphas
    pub update_interval: Duration,

    /// Symbols to monitor
    pub symbols: Vec<Symbol>,

    /// Minimum confidence to act on signals
    pub min_confidence: f64,

    /// Strategy for aggregating signals
    pub aggregation_strategy: AggregationStrategy,

    /// Maximum number of concurrent positions
    pub max_positions: usize,

    /// Enable paper trading mode (no real orders)
    pub paper_trading: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        // STRATEGY: High-frequency small profits (0.5-2% per trade)
        // See STRATEGY.md for full details
        EngineConfig {
            update_interval: Duration::from_secs(60), // 1 minute (balance speed vs noise)
            symbols: Vec::new(),
            min_confidence: 0.7, // 70% minimum (allows 70%+ win rate)
            aggregation_strategy: AggregationStrategy::WeightedAverage,
            max_positions: 10,   // Diversify across 10 small trades
            paper_trading: true, // Safe default
        }
    }
}

/// The Human Edge Trading Engine
pub struct TradingEngine {
    config: EngineConfig,
    data_provider: Arc<dyn DataProvider>,
    alphas: Vec<BoxedAlpha>,
    signal_aggregator: SignalAggregator,
    order_manager: OrderManager,
    stats: EngineStats,
}

/// Engine performance statistics
#[derive(Debug, Clone, Default)]
pub struct EngineStats {
    pub cycles_completed: u64,
    pub signals_generated: u64,
    pub signals_aggregated: u64,
    pub errors_encountered: u64,
    pub total_runtime_ms: u64,
}

impl TradingEngine {
    /// Create new trading engine
    pub fn new(
        config: EngineConfig,
        data_provider: Arc<dyn DataProvider>,
        initial_capital: f64,
    ) -> Self {
        let signal_aggregator = SignalAggregator::new(config.aggregation_strategy)
            .with_min_confidence(config.min_confidence);

        let order_config = OrderManagerConfig {
            max_positions: config.max_positions,
            max_risk_per_trade_pct: 0.5, // 0.5% risk per trade
            paper_trading: config.paper_trading,
        };

        let order_manager = OrderManager::new(initial_capital, order_config);

        TradingEngine {
            config,
            data_provider,
            alphas: Vec::new(),
            signal_aggregator,
            order_manager,
            stats: EngineStats::default(),
        }
    }

    /// Add an alpha model to the engine
    pub fn add_alpha(&mut self, alpha: BoxedAlpha) {
        info!("Added alpha: {}", alpha.name());
        self.alphas.push(alpha);
    }

    /// Get current engine statistics
    pub fn stats(&self) -> &EngineStats {
        &self.stats
    }

    /// Main event loop
    ///
    /// This is the heart of the engine:
    /// 1. Fetch market data at regular intervals
    /// 2. Update all alpha models with new data
    /// 3. Collect signals from all alphas
    /// 4. Aggregate signals using configured strategy
    /// 5. Execute orders (TODO: order manager)
    pub async fn run(mut self) -> Result<()> {
        info!("🚀 The Human Edge Engine starting...");
        info!("   Monitoring {} symbols", self.config.symbols.len());
        info!("   Running {} alpha models", self.alphas.len());
        info!("   Update interval: {:?}", self.config.update_interval);
        info!(
            "   Paper trading: {}",
            if self.config.paper_trading {
                "YES"
            } else {
                "NO"
            }
        );

        if self.alphas.is_empty() {
            warn!("⚠️  No alpha models configured!");
        }

        if self.config.symbols.is_empty() {
            return Err(anyhow::anyhow!("No symbols configured to monitor"));
        }

        let mut interval = time::interval(self.config.update_interval);
        let start_time = std::time::Instant::now();

        loop {
            interval.tick().await;

            let cycle_start = std::time::Instant::now();
            debug!("Starting new engine cycle");

            match self.run_cycle().await {
                Ok(_) => {
                    self.stats.cycles_completed += 1;
                    let cycle_duration = cycle_start.elapsed();
                    debug!(
                        "Cycle {} completed in {:?}",
                        self.stats.cycles_completed, cycle_duration
                    );
                }
                Err(e) => {
                    error!("Engine cycle failed: {}", e);
                    self.stats.errors_encountered += 1;
                }
            }

            self.stats.total_runtime_ms = start_time.elapsed().as_millis() as u64;
        }
    }

    /// Run a single engine cycle
    async fn run_cycle(&mut self) -> Result<()> {
        // Step 1: Fetch market data
        let snapshot = self
            .fetch_market_data()
            .await
            .context("Failed to fetch market data")?;

        debug!("Fetched data for {} symbols", snapshot.len());

        // Step 2: Update order manager positions with new market data
        self.order_manager.update_positions(&snapshot);

        // Step 3: Check and close positions that hit stop loss / take profit
        let closed_trades = self.order_manager.check_exits();
        for trade in closed_trades {
            info!(
                "💰 Closed: {} - P&L: ${:.2} ({:+.2}%) - {:?}",
                trade.symbol, trade.pnl, trade.pnl_pct, trade.close_reason
            );
        }

        // Step 4: Update all alphas with new data
        self.update_alphas(&snapshot);

        // Step 5: Collect signals from all alphas
        let all_signals = self.collect_signals().await;

        debug!("Collected {} raw signals", all_signals.len());
        self.stats.signals_generated += all_signals.len() as u64;

        // Step 6: Aggregate signals
        let aggregated_signals = self.signal_aggregator.aggregate(all_signals);

        debug!(
            "Aggregated to {} actionable signals",
            aggregated_signals.len()
        );
        self.stats.signals_aggregated += aggregated_signals.len() as u64;

        // Step 7: Execute signals
        for signal in &aggregated_signals {
            info!(
                "📊 Signal: {} {:?} @ ${:.2} (confidence: {:.1}%)",
                signal.symbol,
                signal.action,
                signal.target_price.map(|p| p.value()).unwrap_or(0.0),
                signal.confidence.as_percent()
            );
            info!("   Reason: {}", signal.reason);

            // Get current market price for this symbol
            if let Some(market_data) = snapshot.get(&signal.symbol) {
                let current_price = market_data.last_price;

                // Execute the signal
                match self.order_manager.execute_signal(signal, current_price) {
                    Ok(order) => {
                        info!(
                            "✅ Order executed: {} {} @ ${:.2}",
                            order.symbol,
                            order.quantity.value(),
                            current_price.value()
                        );
                    }
                    Err(e) => {
                        warn!("⚠️  Order rejected: {} - {}", signal.symbol, e);
                    }
                }
            } else {
                warn!("⚠️  No market data for signal symbol: {}", signal.symbol);
            }
        }

        // Step 8: Log portfolio status
        let om_stats = self.order_manager.stats();
        let risk_stats = self.order_manager.risk_stats();

        info!(
            "💼 Portfolio: ${:.2} | Open: {} | P&L: ${:.2} (realized) + ${:.2} (unrealized)",
            om_stats.portfolio_value,
            self.order_manager.position_count(),
            om_stats.total_pnl,
            om_stats.unrealized_pnl
        );

        // Log risk status
        if !risk_stats.is_healthy() {
            warn!("⚠️  Risk Alert: {}", risk_stats.status_message());
        } else {
            debug!("🛡️  Risk Status: {}", risk_stats.status_message());
        }

        Ok(())
    }

    /// Fetch current market data for all configured symbols
    async fn fetch_market_data(&self) -> Result<MarketSnapshot> {
        self.data_provider
            .get_quotes(&self.config.symbols)
            .await
            .context("Data provider failed")
    }

    /// Update all alpha models with new market data
    fn update_alphas(&mut self, snapshot: &MarketSnapshot) {
        for alpha in &mut self.alphas {
            alpha.update(snapshot);
        }
    }

    /// Collect signals from all alpha models
    async fn collect_signals(&self) -> Vec<Signal> {
        let mut all_signals = Vec::new();

        for alpha in &self.alphas {
            match alpha.generate_signals().await {
                signals => {
                    debug!(
                        "Alpha '{}' generated {} signals",
                        alpha.name(),
                        signals.len()
                    );
                    all_signals.extend(signals);
                }
            }
        }

        all_signals
    }
}

/// Builder for TradingEngine with performance optimizations
pub struct TradingEngineBuilder {
    config: EngineConfig,
    data_provider: Option<Arc<dyn DataProvider>>,
    initial_capital: f64,
    cpu_cores: Vec<usize>,
    lockfree: bool,
    preallocated_memory: usize,
}

impl TradingEngineBuilder {
    pub fn new() -> Self {
        TradingEngineBuilder {
            config: EngineConfig::default(),
            data_provider: None,
            initial_capital: 10_000.0, // Default $10k
            cpu_cores: vec![],
            lockfree: false,
            preallocated_memory: 0,
        }
    }

    pub fn with_initial_capital(mut self, capital: f64) -> Self {
        self.initial_capital = capital;
        self
    }

    pub fn with_config(mut self, config: EngineConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_data_provider(mut self, provider: Arc<dyn DataProvider>) -> Self {
        self.data_provider = Some(provider);
        self
    }

    pub fn with_symbols(mut self, symbols: Vec<Symbol>) -> Self {
        self.config.symbols = symbols;
        self
    }

    pub fn with_update_interval(mut self, interval: Duration) -> Self {
        self.config.update_interval = interval;
        self
    }

    pub fn with_min_confidence(mut self, confidence: f64) -> Self {
        self.config.min_confidence = confidence;
        self
    }

    pub fn with_aggregation_strategy(mut self, strategy: AggregationStrategy) -> Self {
        self.config.aggregation_strategy = strategy;
        self
    }

    pub fn with_paper_trading(mut self, enabled: bool) -> Self {
        self.config.paper_trading = enabled;
        self
    }

    // Performance optimization methods
    pub fn with_cpu_pinning(mut self, cores: Vec<usize>) -> Self {
        self.cpu_cores = cores;
        self
    }

    pub fn with_lockfree_structures(mut self) -> Self {
        self.lockfree = true;
        self
    }

    pub fn with_preallocated_memory(mut self, size: usize) -> Self {
        self.preallocated_memory = size;
        self
    }

    pub fn build(self) -> Result<TradingEngine> {
        let data_provider = self
            .data_provider
            .ok_or_else(|| anyhow::anyhow!("Data provider is required"))?;

        // TODO: Apply performance optimizations
        if !self.cpu_cores.is_empty() {
            info!("CPU pinning requested but not yet implemented");
        }
        if self.lockfree {
            info!("Lock-free structures requested but not yet implemented");
        }
        if self.preallocated_memory > 0 {
            info!(
                "Memory pre-allocation ({} bytes) requested but not yet implemented",
                self.preallocated_memory
            );
        }

        Ok(TradingEngine::new(
            self.config,
            data_provider,
            self.initial_capital,
        ))
    }
}

impl Default for TradingEngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TradingEngine {
    pub fn builder() -> TradingEngineBuilder {
        TradingEngineBuilder::new()
    }
}
