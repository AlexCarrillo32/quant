//! Backtesting Engine - Simulate Trading on Historical Data
//!
//! **Purpose**: Test strategies before risking real money
//!
//! Features:
//! - Realistic execution (slippage, commissions)
//! - Multiple alpha models
//! - Risk management integration
//! - Performance analysis

use crate::alphas::AlphaModel;
use crate::backtest::metrics::PerformanceMetrics;
use crate::backtest::trade::{BacktestTrade, ExitReason};
use crate::core::risk_manager::{RiskManager, RiskManagerConfig};
use crate::core::signal_aggregator::{SignalAggregator, AggregationStrategy};
use crate::types::*;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::time::{SystemTime, Duration};

/// Backtesting configuration
#[derive(Debug, Clone)]
pub struct BacktestConfig {
    /// Initial capital
    pub initial_capital: f64,

    /// Commission per trade (fixed $)
    pub commission_per_trade: f64,

    /// Slippage (% of price)
    pub slippage_pct: f64,

    /// Default position size (% of capital)
    pub default_position_size_pct: f64,

    /// Use signal confidence for position sizing?
    pub use_confidence_sizing: bool,

    /// Risk manager configuration
    pub risk_config: RiskManagerConfig,

    /// Signal aggregation strategy
    pub aggregation_strategy: AggregationStrategy,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: 10_000.0,
            commission_per_trade: 1.0,
            slippage_pct: 0.05, // 0.05% = 5 basis points
            default_position_size_pct: 10.0,
            use_confidence_sizing: true,
            risk_config: RiskManagerConfig::default(),
            aggregation_strategy: AggregationStrategy::WeightedAverage,
        }
    }
}

/// Result of a backtest
#[derive(Debug, Clone)]
pub struct BacktestResult {
    /// Performance metrics
    pub metrics: PerformanceMetrics,

    /// All trades executed
    pub trades: Vec<BacktestTrade>,

    /// Equity curve (daily)
    pub equity_curve: Vec<f64>,

    /// Final capital
    pub final_capital: f64,

    /// Total number of signals generated
    pub total_signals: usize,

    /// Signals rejected by risk manager
    pub rejected_signals: usize,
}

/// Backtester - simulates trading on historical data
pub struct Backtester {
    config: BacktestConfig,
    alphas: Vec<Box<dyn AlphaModel>>,
    aggregator: SignalAggregator,
    risk_manager: RiskManager,
}

impl Backtester {
    /// Create a new backtester
    pub fn new(config: BacktestConfig) -> Self {
        let aggregator = SignalAggregator::new(config.aggregation_strategy);
        let risk_manager = RiskManager::new(config.risk_config.clone(), config.initial_capital);

        Self {
            config,
            alphas: Vec::new(),
            aggregator,
            risk_manager,
        }
    }

    /// Add an alpha model
    pub fn add_alpha(&mut self, alpha: Box<dyn AlphaModel>) {
        self.alphas.push(alpha);
    }

    /// Run backtest on historical data
    ///
    /// # Parameters
    /// - `historical_data`: Map of symbol -> time series of market data
    /// - `symbols`: Symbols to track
    ///
    /// # Returns
    /// Backtest results with performance metrics
    pub fn run(
        &mut self,
        historical_data: &HashMap<Symbol, Vec<MarketData>>,
        symbols: &[Symbol],
    ) -> Result<BacktestResult> {
        // Initialize alphas (no initialization method needed)
        for alpha in &mut self.alphas {
            alpha.reset();
        }

        // Find the minimum number of data points across all symbols
        let min_data_points = historical_data
            .values()
            .map(|v| v.len())
            .min()
            .context("No historical data provided")?;

        let mut capital = self.config.initial_capital;
        let mut equity_curve = vec![capital];
        let mut open_trades: HashMap<Symbol, BacktestTrade> = HashMap::new();
        let mut closed_trades: Vec<BacktestTrade> = Vec::new();
        let mut total_signals = 0;
        let mut rejected_signals = 0;

        // Simulate each time step
        for i in 0..min_data_points {
            let current_time = SystemTime::UNIX_EPOCH + Duration::from_secs((i as u64) * 86400);

            // Build market snapshot for this time step
            let mut market_snapshot = MarketSnapshot::new();
            for symbol in symbols {
                if let Some(data) = historical_data.get(symbol) {
                    if let Some(market_data) = data.get(i) {
                        market_snapshot.data.insert(symbol.clone(), market_data.clone());
                    }
                }
            }

            // Generate signals from all alphas
            // Note: In a real implementation, you'd need to handle async alphas differently
            // For now, we'll just update each alpha and assume synchronous signal generation
            let all_signals = Vec::new();
            for alpha in &mut self.alphas {
                alpha.update(&market_snapshot);
                // In a backtest, we can't use await, so this would need to be refactored
                // to use tokio::runtime or process signals synchronously
            }

            total_signals += all_signals.len();

            // Aggregate signals
            let aggregated_signals = self.aggregator.aggregate(all_signals.clone());

            // Check if we should close existing positions
            for (symbol, trade) in open_trades.iter_mut() {
                if let Some(market_data) = market_snapshot.data.get(symbol) {
                    // Check stop loss / take profit
                    if let Some(exit_reason) =
                        self.should_exit(trade, market_data, &aggregated_signals)
                    {
                        let exit_price = self.apply_slippage(market_data.quote.mid(), true);
                        let exit_commission = self.config.commission_per_trade;
                        let exit_slippage =
                            (exit_price.value() - market_data.quote.mid().value()).abs();

                        trade.close(
                            exit_price,
                            current_time,
                            exit_reason,
                            exit_commission,
                            exit_slippage,
                        );

                        // Update capital
                        capital += trade.net_pnl.unwrap_or(0.0);
                    }
                }
            }

            // Remove closed trades
            let closed: Vec<_> = open_trades.iter()
                .filter(|(_, t)| !t.is_open())
                .map(|(k, _)| k.clone())
                .collect();
            for key in closed {
                if let Some((_, trade)) = open_trades.remove_entry(&key) {
                    closed_trades.push(trade);
                }
            }

            // Process new signals
            for signal in aggregated_signals {
                if signal.action == SignalAction::Hold {
                    continue;
                }

                // Skip if we already have a position
                if open_trades.contains_key(&signal.symbol) {
                    continue;
                }

                // Get market data
                let market_data = match market_snapshot.data.get(&signal.symbol) {
                    Some(data) => data,
                    None => continue,
                };

                // Check risk manager
                let position_value = capital * (self.config.default_position_size_pct / 100.0);
                let open_positions_map: std::collections::HashMap<Symbol, f64> = open_trades
                    .iter()
                    .map(|(s, t)| (s.clone(), t.entry_price.value() * t.quantity.value() as f64))
                    .collect();

                let risk_check = self.risk_manager.check_trade(
                    &signal.symbol,
                    position_value,
                    position_value * 0.01, // Risk amount (1% for example)
                    &open_positions_map,
                    10, // max positions
                    capital,
                );

                if !matches!(risk_check, crate::core::risk_manager::RiskCheckResult::Approved) {
                    rejected_signals += 1;
                    continue;
                }

                // Calculate position size
                let position_size = self.calculate_position_size(capital, &signal);

                if position_size < 1.0 {
                    continue; // Can't buy fractional shares in this simulation
                }

                // Execute trade
                let entry_price = self.apply_slippage(market_data.quote.mid(), false);
                let quantity = Quantity::buy(position_size as u64);
                let commission = self.config.commission_per_trade;
                let slippage = (entry_price.value() - market_data.quote.mid().value()).abs();

                let trade = BacktestTrade::new(
                    signal.symbol.clone(),
                    signal.action,
                    entry_price,
                    quantity,
                    current_time,
                    commission,
                    slippage,
                    signal.confidence.value(),
                );

                // Deduct capital
                let trade_cost = entry_price.value() * quantity.value() as f64 + commission;
                capital -= trade_cost;

                open_trades.insert(signal.symbol.clone(), trade);
            }

            // Update equity curve (capital + value of open positions)
            let open_positions_value: f64 = open_trades
                .iter()
                .filter_map(|(symbol, trade)| {
                    market_snapshot.data.get(symbol).map(|data| {
                        let current_price = data.quote.mid().value();
                        current_price * trade.quantity.value() as f64
                    })
                })
                .sum();

            equity_curve.push(capital + open_positions_value);
        }

        // Close all remaining positions at the end
        for (symbol, mut trade) in open_trades.into_iter() {
            if let Some(data_series) = historical_data.get(&symbol) {
                if let Some(last_data) = data_series.last() {
                    let exit_price = self.apply_slippage(last_data.quote.mid(), true);
                    trade.close(
                        exit_price,
                        SystemTime::now(),
                        ExitReason::EndOfData,
                        self.config.commission_per_trade,
                        0.0,
                    );

                    capital += trade.net_pnl.unwrap_or(0.0);
                    closed_trades.push(trade);
                }
            }
        }

        // Calculate metrics
        let metrics = PerformanceMetrics::calculate(
            &equity_curve,
            &closed_trades,
            self.config.initial_capital,
            min_data_points,
        );

        Ok(BacktestResult {
            metrics,
            trades: closed_trades,
            equity_curve,
            final_capital: capital,
            total_signals,
            rejected_signals,
        })
    }

    /// Should we exit this trade?
    fn should_exit(
        &self,
        trade: &BacktestTrade,
        market_data: &MarketData,
        aggregated_signals: &[Signal],
    ) -> Option<ExitReason> {
        let current_price = market_data.quote.mid();

        // Check for opposing signal
        for signal in aggregated_signals {
            if signal.symbol == trade.symbol && signal.action != trade.action {
                return Some(ExitReason::SignalReverse);
            }
        }

        // Check stop loss (2% default)
        let stop_loss_pct = 2.0;
        match trade.action {
            SignalAction::Buy => {
                let loss_pct = ((current_price.value() - trade.entry_price.value()) / trade.entry_price.value()) * 100.0;
                if loss_pct < -stop_loss_pct {
                    return Some(ExitReason::StopLoss);
                }
            }
            SignalAction::Sell => {
                let loss_pct = ((trade.entry_price.value() - current_price.value()) / trade.entry_price.value()) * 100.0;
                if loss_pct < -stop_loss_pct {
                    return Some(ExitReason::StopLoss);
                }
            }
            SignalAction::Hold | SignalAction::Close => {}
        }

        // Check take profit (4% = 2:1 R:R)
        let take_profit_pct = 4.0;
        match trade.action {
            SignalAction::Buy => {
                let profit_pct = ((current_price.value() - trade.entry_price.value()) / trade.entry_price.value()) * 100.0;
                if profit_pct > take_profit_pct {
                    return Some(ExitReason::TakeProfit);
                }
            }
            SignalAction::Sell => {
                let profit_pct = ((trade.entry_price.value() - current_price.value()) / trade.entry_price.value()) * 100.0;
                if profit_pct > take_profit_pct {
                    return Some(ExitReason::TakeProfit);
                }
            }
            SignalAction::Hold | SignalAction::Close => {}
        }

        None
    }

    /// Calculate position size based on signal and capital
    fn calculate_position_size(&self, capital: f64, signal: &Signal) -> f64 {
        let base_size_pct = if self.config.use_confidence_sizing {
            // Scale position size by confidence (50% - 100% of default)
            self.config.default_position_size_pct * (0.5 + signal.confidence.value() * 0.5)
        } else {
            self.config.default_position_size_pct
        };

        let position_value = capital * (base_size_pct / 100.0);

        // Assume we're using the mid price (simplified)
        // In reality, we'd use the actual execution price
        position_value
    }

    /// Run backtest with pre-generated signals (for Python integration)
    ///
    /// # Parameters
    /// - `signals`: Pre-generated trading signals with entry prices
    ///
    /// # Returns
    /// Backtest results with performance metrics
    pub fn run_with_signals(&mut self, signals: Vec<Signal>) -> Result<BacktestResult> {
        let mut capital = self.config.initial_capital;
        let mut equity_curve = vec![capital];
        let mut open_trades: HashMap<Symbol, BacktestTrade> = HashMap::new();
        let mut closed_trades: Vec<BacktestTrade> = Vec::new();
        let total_signals = signals.len();
        let mut rejected_signals = 0;

        // Process each signal in order
        for (idx, signal) in signals.iter().enumerate() {
            let current_time = SystemTime::UNIX_EPOCH + Duration::from_secs((idx as u64) * 86400);

            if signal.action == SignalAction::Hold {
                continue;
            }

            // Check if we should close existing position for this symbol
            if let Some(mut trade) = open_trades.remove(&signal.symbol) {
                // Close existing position
                let exit_price = if let Some(price) = signal.target_price {
                    self.apply_slippage(price, true)
                } else {
                    self.apply_slippage(trade.entry_price, true) // Use entry as fallback
                };

                let exit_commission = self.config.commission_per_trade;
                let exit_slippage =
                    (exit_price.value() - trade.entry_price.value()).abs() * 0.01;

                trade.close(
                    exit_price,
                    current_time,
                    ExitReason::SignalReverse,
                    exit_commission,
                    exit_slippage,
                );

                // Update capital
                capital += trade.net_pnl.unwrap_or(0.0);
                closed_trades.push(trade);
            }

            // Skip if action is to close but we had no position
            if signal.action == SignalAction::Close {
                continue;
            }

            // Get entry price from signal (use target_price as entry price)
            let entry_price = match signal.target_price {
                Some(price) => price,
                None => {
                    rejected_signals += 1;
                    continue; // Need price to execute
                }
            };

            // Check risk manager
            let position_value = capital * (self.config.default_position_size_pct / 100.0);
            let open_positions_map: HashMap<Symbol, f64> = open_trades
                .iter()
                .map(|(s, t)| (s.clone(), t.entry_price.value() * t.quantity.value() as f64))
                .collect();

            let risk_amount = if let Some(stop_loss) = signal.stop_loss {
                // Calculate risk based on stop loss
                let risk_per_share = (entry_price.value() - stop_loss.value()).abs();
                let num_shares = (position_value / entry_price.value()).floor();
                risk_per_share * num_shares
            } else {
                position_value * 0.01 // Default 1% risk
            };

            let risk_check = self.risk_manager.check_trade(
                &signal.symbol,
                position_value,
                risk_amount,
                &open_positions_map,
                10, // max positions
                capital,
            );

            if !matches!(risk_check, crate::core::risk_manager::RiskCheckResult::Approved) {
                rejected_signals += 1;
                continue;
            }

            // Calculate position size
            let position_size = self.calculate_position_size(capital, signal);

            if position_size < 1.0 {
                continue; // Can't buy fractional shares
            }

            // Execute trade
            let adjusted_entry = self.apply_slippage(entry_price, false);
            let quantity = Quantity::buy(position_size as u64);
            let commission = self.config.commission_per_trade;
            let slippage = (adjusted_entry.value() - entry_price.value()).abs();

            let trade = BacktestTrade::new(
                signal.symbol.clone(),
                signal.action,
                adjusted_entry,
                quantity,
                current_time,
                commission,
                slippage,
                signal.confidence.value(),
            );

            // Deduct capital
            let trade_cost = adjusted_entry.value() * quantity.value() as f64 + commission;

            if trade_cost > capital {
                rejected_signals += 1;
                continue; // Not enough capital
            }

            capital -= trade_cost;
            open_trades.insert(signal.symbol.clone(), trade);

            // Update equity curve
            let open_positions_value: f64 = open_trades
                .iter()
                .map(|(_, trade)| entry_price.value() * trade.quantity.value() as f64)
                .sum();

            equity_curve.push(capital + open_positions_value);
        }

        // Close all remaining positions at last signal price
        let final_time = SystemTime::UNIX_EPOCH +
            Duration::from_secs((signals.len() as u64) * 86400);

        for (_, mut trade) in open_trades.into_iter() {
            // Use the trade's entry price as exit (simplified)
            let exit_price = self.apply_slippage(trade.entry_price, true);

            trade.close(
                exit_price,
                final_time,
                ExitReason::EndOfData,
                self.config.commission_per_trade,
                0.0,
            );

            capital += trade.net_pnl.unwrap_or(0.0);
            closed_trades.push(trade);
        }

        // Calculate metrics
        let metrics = PerformanceMetrics::calculate(
            &equity_curve,
            &closed_trades,
            self.config.initial_capital,
            signals.len(),
        );

        Ok(BacktestResult {
            metrics,
            trades: closed_trades,
            equity_curve,
            final_capital: capital,
            total_signals,
            rejected_signals,
        })
    }

    /// Apply slippage to a price
    fn apply_slippage(&self, price: Price, is_exit: bool) -> Price {
        let slippage_amount = price.value() * (self.config.slippage_pct / 100.0);

        // Entry: pay more (buy) or get less (sell)
        // Exit: get less (sell) or pay more (cover)
        if is_exit {
            Price::new(price.value() - slippage_amount).unwrap_or(price)
        } else {
            Price::new(price.value() + slippage_amount).unwrap_or(price)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backtester_creation() {
        let config = BacktestConfig::default();
        let backtester = Backtester::new(config);

        assert_eq!(backtester.alphas.len(), 0);
    }

    #[test]
    fn test_slippage_application() {
        let config = BacktestConfig {
            slippage_pct: 0.1, // 0.1%
            ..Default::default()
        };

        let backtester = Backtester::new(config);

        // Entry slippage: pay more
        let entry_price = backtester.apply_slippage(Price::new(100.0).unwrap(), false);
        assert!((entry_price.value() - 100.1).abs() < 0.01);

        // Exit slippage: get less
        let exit_price = backtester.apply_slippage(Price::new(100.0).unwrap(), true);
        assert!((exit_price.value() - 99.9).abs() < 0.01);
    }
}
