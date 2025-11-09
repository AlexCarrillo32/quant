//! Order Manager - Turns Signals into Trades
//!
//! **Critical Component**: This is where market data → signals → actual trades
//!
//! Strategy: Small profits (0.5-2%), high win rate (70%+)
//! - Calculate position size based on risk
//! - Track open positions
//! - Monitor stop loss / take profit
//! - Calculate P&L
//! - Risk checks before every trade

use crate::core::risk_manager::{RiskCheckResult, RiskManager, RiskManagerConfig};
use crate::types::*;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Order status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,   // Created but not filled
    Filled,    // Executed successfully
    Cancelled, // Cancelled before fill
    Rejected,  // Rejected by risk checks
}

/// Order side
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Order type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Market,     // Execute at market price
    Limit,      // Execute at specific price or better
    StopLoss,   // Trigger when price hits stop
    TakeProfit, // Trigger when price hits target
}

/// Trading order
#[derive(Debug, Clone)]
pub struct Order {
    pub id: String,
    pub symbol: Symbol,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: Quantity,
    pub price: Option<Price>, // For limit orders
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub filled_at: Option<DateTime<Utc>>,
    pub filled_price: Option<Price>,
}

impl Order {
    /// Create new market order
    pub fn market(symbol: Symbol, side: OrderSide, quantity: Quantity) -> Self {
        Order {
            id: uuid::Uuid::new_v4().to_string(),
            symbol,
            side,
            order_type: OrderType::Market,
            quantity,
            price: None,
            status: OrderStatus::Pending,
            created_at: Utc::now(),
            filled_at: None,
            filled_price: None,
        }
    }

    /// Create limit order
    pub fn limit(symbol: Symbol, side: OrderSide, quantity: Quantity, price: Price) -> Self {
        Order {
            id: uuid::Uuid::new_v4().to_string(),
            symbol,
            side,
            order_type: OrderType::Limit,
            quantity,
            price: Some(price),
            status: OrderStatus::Pending,
            created_at: Utc::now(),
            filled_at: None,
            filled_price: None,
        }
    }

    /// Mark order as filled
    pub fn fill(&mut self, price: Price) {
        self.status = OrderStatus::Filled;
        self.filled_at = Some(Utc::now());
        self.filled_price = Some(price);
    }
}

/// Open position
#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: Symbol,
    pub quantity: Quantity,
    pub entry_price: Price,
    pub current_price: Price,
    pub stop_loss: Option<Price>,
    pub take_profit: Option<Price>,
    pub opened_at: DateTime<Utc>,
    pub source_signal: String, // Which alpha generated this
}

impl Position {
    /// Calculate unrealized P&L
    pub fn unrealized_pnl(&self) -> f64 {
        let price_diff = self.current_price.value() - self.entry_price.value();
        price_diff * self.quantity.value() as f64
    }

    /// Calculate unrealized P&L percentage
    pub fn unrealized_pnl_pct(&self) -> f64 {
        (self.current_price.value() / self.entry_price.value() - 1.0) * 100.0
    }

    /// Check if stop loss hit
    pub fn stop_loss_hit(&self) -> bool {
        if let Some(stop) = self.stop_loss {
            self.current_price <= stop
        } else {
            false
        }
    }

    /// Check if take profit hit
    pub fn take_profit_hit(&self) -> bool {
        if let Some(target) = self.take_profit {
            self.current_price >= target
        } else {
            false
        }
    }

    /// Update current price
    pub fn update_price(&mut self, price: Price) {
        self.current_price = price;
    }
}

/// Position sizing calculator
///
/// Strategy: Risk 0.3-0.5% per trade for small profits
pub struct PositionSizer {
    portfolio_value: f64,
    max_risk_per_trade_pct: f64, // 0.5% = $50 on $10k portfolio
}

impl PositionSizer {
    pub fn new(portfolio_value: f64, max_risk_pct: f64) -> Self {
        PositionSizer {
            portfolio_value,
            max_risk_per_trade_pct: max_risk_pct,
        }
    }

    /// Calculate position size based on risk
    ///
    /// Formula: shares = (portfolio × risk%) / (entry - stop_loss)
    ///
    /// Example:
    /// - Portfolio: $10,000
    /// - Risk: 0.5% = $50
    /// - Entry: $670.00
    /// - Stop: $666.50 (0.5% below)
    /// - Distance: $3.50
    /// - Shares: $50 / $3.50 = 14 shares
    pub fn calculate_size(&self, entry_price: Price, stop_loss: Option<Price>) -> Result<Quantity> {
        let stop = stop_loss.context("Stop loss required for position sizing")?;

        // Calculate risk amount in dollars
        let risk_dollars = self.portfolio_value * (self.max_risk_per_trade_pct / 100.0);

        // Calculate price distance to stop
        let stop_distance = (entry_price.value() - stop.value()).abs();

        if stop_distance < 0.01 {
            return Err(anyhow::anyhow!(
                "Stop loss too close to entry: ${:.2}",
                stop_distance
            ));
        }

        // Calculate number of shares
        let shares = (risk_dollars / stop_distance).floor() as i32;

        if shares <= 0 {
            return Err(anyhow::anyhow!(
                "Position size too small: {} shares",
                shares
            ));
        }

        Ok(Quantity::buy(shares as u64))
    }
}

/// Order Manager
///
/// Responsibilities:
/// - Execute orders (paper trading mode)
/// - Track open positions
/// - Monitor stop loss / take profit
/// - Calculate P&L
/// - Risk management
pub struct OrderManager {
    /// Portfolio cash balance
    cash: f64,

    /// Open positions
    positions: HashMap<Symbol, Position>,

    /// Completed trades history
    trade_history: Vec<Trade>,

    /// Position sizer
    sizer: PositionSizer,

    /// Risk manager
    risk_manager: RiskManager,

    /// Configuration
    config: OrderManagerConfig,
}

#[derive(Debug, Clone)]
pub struct OrderManagerConfig {
    /// Maximum number of open positions
    pub max_positions: usize,

    /// Maximum risk per trade (%)
    pub max_risk_per_trade_pct: f64,

    /// Paper trading mode (no real money)
    pub paper_trading: bool,
}

impl Default for OrderManagerConfig {
    fn default() -> Self {
        OrderManagerConfig {
            max_positions: 10,
            max_risk_per_trade_pct: 0.5, // 0.5% risk per trade
            paper_trading: true,         // Safe default
        }
    }
}

/// Completed trade
#[derive(Debug, Clone)]
pub struct Trade {
    pub symbol: Symbol,
    pub side: OrderSide,
    pub quantity: Quantity,
    pub entry_price: Price,
    pub exit_price: Price,
    pub pnl: f64,
    pub pnl_pct: f64,
    pub opened_at: DateTime<Utc>,
    pub closed_at: DateTime<Utc>,
    pub close_reason: CloseReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseReason {
    StopLoss,
    TakeProfit,
    Manual,
    Timeout,
}

impl OrderManager {
    /// Create new order manager
    pub fn new(initial_cash: f64, config: OrderManagerConfig) -> Self {
        let sizer = PositionSizer::new(initial_cash, config.max_risk_per_trade_pct);

        // Create risk manager with matching config
        let risk_config = RiskManagerConfig {
            max_risk_per_trade_pct: config.max_risk_per_trade_pct,
            max_daily_drawdown_pct: 5.0,        // Stop at -5% daily loss
            max_correlation_exposure_pct: 50.0, // Max 50% in correlated assets
            max_consecutive_losses: 3,          // Stop after 3 losses in a row
            emergency_stop_value: initial_cash * 0.5, // Emergency stop at 50% loss
        };
        let risk_manager = RiskManager::new(risk_config, initial_cash);

        OrderManager {
            cash: initial_cash,
            positions: HashMap::new(),
            trade_history: Vec::new(),
            sizer,
            risk_manager,
            config,
        }
    }

    /// Reset daily risk counters (call at start of each trading day)
    pub fn reset_daily(&mut self) {
        self.risk_manager.reset_daily();
    }

    /// Get current portfolio value
    pub fn portfolio_value(&self) -> f64 {
        let positions_value: f64 = self
            .positions
            .values()
            .map(|p| p.current_price.value() * p.quantity.value() as f64)
            .sum();

        self.cash + positions_value
    }

    /// Get total unrealized P&L
    pub fn unrealized_pnl(&self) -> f64 {
        self.positions.values().map(|p| p.unrealized_pnl()).sum()
    }

    /// Get total realized P&L
    pub fn realized_pnl(&self) -> f64 {
        self.trade_history.iter().map(|t| t.pnl).sum()
    }

    /// Get number of open positions
    pub fn position_count(&self) -> usize {
        self.positions.len()
    }

    /// Check if we can open new position
    pub fn can_open_position(&self) -> bool {
        self.positions.len() < self.config.max_positions
    }

    /// Execute signal (create and fill order)
    pub fn execute_signal(&mut self, signal: &Signal, current_price: Price) -> Result<Order> {
        // Check if we already have position in this symbol
        if self.positions.contains_key(&signal.symbol) {
            return Err(anyhow::anyhow!(
                "Already have position in {}",
                signal.symbol
            ));
        }

        // Calculate position size
        let quantity = self.sizer.calculate_size(current_price, signal.stop_loss)?;

        // Calculate position value and risk
        let position_value = current_price.value() * quantity.value() as f64;
        let stop_loss = signal
            .stop_loss
            .context("Stop loss required for risk check")?;
        let risk_amount =
            (current_price.value() - stop_loss.value()).abs() * quantity.value() as f64;

        // Build position values map for correlation check
        let position_values: HashMap<Symbol, f64> = self
            .positions
            .iter()
            .map(|(sym, pos)| {
                let value = pos.current_price.value() * pos.quantity.value() as f64;
                (sym.clone(), value)
            })
            .collect();

        // Update risk manager with current portfolio value
        self.risk_manager
            .update_portfolio_value(self.portfolio_value());

        // Risk check
        let risk_result = self.risk_manager.check_trade(
            &signal.symbol,
            position_value,
            risk_amount,
            &position_values,
            self.config.max_positions,
            self.cash,
        );

        match risk_result {
            RiskCheckResult::Rejected(violation) => {
                return Err(anyhow::anyhow!("Risk check failed: {}", violation));
            }
            RiskCheckResult::Approved => {
                // Continue with trade
            }
        }

        // Create order
        let side = match signal.action {
            SignalAction::Buy => OrderSide::Buy,
            SignalAction::Sell => OrderSide::Sell,
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid signal action: {:?}",
                    signal.action
                ))
            }
        };

        let mut order = Order::market(signal.symbol.clone(), side, quantity);

        // Simulate order fill (paper trading)
        if self.config.paper_trading {
            order.fill(current_price);

            // Create position
            let position = Position {
                symbol: signal.symbol.clone(),
                quantity,
                entry_price: current_price,
                current_price,
                stop_loss: signal.stop_loss,
                take_profit: signal.take_profit,
                opened_at: Utc::now(),
                source_signal: signal.source.clone(),
            };

            // Update cash
            let cost = current_price.value() * quantity.value() as f64;
            self.cash -= cost;

            // Add position
            self.positions.insert(signal.symbol.clone(), position);

            tracing::info!(
                "✅ Opened position: {} {} shares @ ${:.2} (stop: ${:.2}, target: ${:.2})",
                signal.symbol,
                quantity.value(),
                current_price.value(),
                signal.stop_loss.map(|p| p.value()).unwrap_or(0.0),
                signal.take_profit.map(|p| p.value()).unwrap_or(0.0)
            );
        }

        Ok(order)
    }

    /// Update positions with current market data
    pub fn update_positions(&mut self, snapshot: &MarketSnapshot) {
        for (symbol, position) in &mut self.positions {
            if let Some(market_data) = snapshot.get(symbol) {
                position.update_price(market_data.last_price);
            }
        }
    }

    /// Check and close positions that hit stop/target
    pub fn check_exits(&mut self) -> Vec<Trade> {
        let mut closed_trades = Vec::new();
        let mut to_close = Vec::new();

        for (symbol, position) in &self.positions {
            if position.stop_loss_hit() {
                to_close.push((symbol.clone(), CloseReason::StopLoss));
            } else if position.take_profit_hit() {
                to_close.push((symbol.clone(), CloseReason::TakeProfit));
            }
        }

        // Close positions
        for (symbol, reason) in to_close {
            if let Some(trade) = self.close_position(&symbol, reason) {
                closed_trades.push(trade);
            }
        }

        closed_trades
    }

    /// Close position
    fn close_position(&mut self, symbol: &Symbol, reason: CloseReason) -> Option<Trade> {
        let position = self.positions.remove(symbol)?;

        // Calculate P&L
        let pnl = position.unrealized_pnl();
        let pnl_pct = position.unrealized_pnl_pct();

        // Record trade with risk manager
        self.risk_manager.record_trade(pnl);

        // Update cash
        let proceeds = position.current_price.value() * position.quantity.value() as f64;
        self.cash += proceeds;

        // Update risk manager with new portfolio value
        self.risk_manager
            .update_portfolio_value(self.portfolio_value());

        let trade = Trade {
            symbol: symbol.clone(),
            side: OrderSide::Buy, // Assuming long-only for now
            quantity: position.quantity,
            entry_price: position.entry_price,
            exit_price: position.current_price,
            pnl,
            pnl_pct,
            opened_at: position.opened_at,
            closed_at: Utc::now(),
            close_reason: reason,
        };

        let emoji = if pnl > 0.0 { "💰" } else { "📉" };
        tracing::info!(
            "{} Closed position: {} - P&L: ${:.2} ({:+.2}%) - Reason: {:?}",
            emoji,
            symbol,
            pnl,
            pnl_pct,
            reason
        );

        self.trade_history.push(trade.clone());

        Some(trade)
    }

    /// Get risk statistics
    pub fn risk_stats(&self) -> crate::core::risk_manager::RiskStats {
        self.risk_manager.stats()
    }

    /// Get statistics
    pub fn stats(&self) -> OrderManagerStats {
        let total_trades = self.trade_history.len();
        let winning_trades = self.trade_history.iter().filter(|t| t.pnl > 0.0).count();
        let losing_trades = total_trades - winning_trades;

        let win_rate = if total_trades > 0 {
            (winning_trades as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        let avg_win = if winning_trades > 0 {
            self.trade_history
                .iter()
                .filter(|t| t.pnl > 0.0)
                .map(|t| t.pnl)
                .sum::<f64>()
                / winning_trades as f64
        } else {
            0.0
        };

        let avg_loss = if losing_trades > 0 {
            self.trade_history
                .iter()
                .filter(|t| t.pnl < 0.0)
                .map(|t| t.pnl)
                .sum::<f64>()
                / losing_trades as f64
        } else {
            0.0
        };

        OrderManagerStats {
            total_trades,
            winning_trades,
            losing_trades,
            win_rate,
            avg_win,
            avg_loss,
            total_pnl: self.realized_pnl(),
            unrealized_pnl: self.unrealized_pnl(),
            portfolio_value: self.portfolio_value(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrderManagerStats {
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub total_pnl: f64,
    pub unrealized_pnl: f64,
    pub portfolio_value: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_sizer() {
        let sizer = PositionSizer::new(10_000.0, 0.5); // $10k, 0.5% risk

        let entry = Price::new(670.0).unwrap();
        let stop = Price::new(666.50).unwrap(); // 0.52% below

        let size = sizer.calculate_size(entry, Some(stop)).unwrap();

        // Risk: $10k × 0.5% = $50
        // Distance: $670 - $666.50 = $3.50
        // Shares: $50 / $3.50 = 14.28 → 14 shares
        assert_eq!(size.value(), 14);
    }

    #[test]
    fn test_order_manager_creation() {
        let config = OrderManagerConfig::default();
        let manager = OrderManager::new(10_000.0, config);

        assert_eq!(manager.cash, 10_000.0);
        assert_eq!(manager.position_count(), 0);
        assert!(manager.can_open_position());
    }

    #[test]
    fn test_position_pnl() {
        let mut position = Position {
            symbol: Symbol::new("SPY").unwrap(),
            quantity: Quantity::buy(10),
            entry_price: Price::new(670.0).unwrap(),
            current_price: Price::new(680.0).unwrap(), // +$10/share
            stop_loss: None,
            take_profit: None,
            opened_at: Utc::now(),
            source_signal: "test".to_string(),
        };

        // P&L: +$10 × 10 shares = +$100
        assert_eq!(position.unrealized_pnl(), 100.0);

        // Update price
        position.update_price(Price::new(665.0).unwrap()); // -$5/share
        assert_eq!(position.unrealized_pnl(), -50.0);
    }
}
