//! Kelly Criterion Position Sizing
//!
//! **Mathematical Optimization**: Size positions to maximize geometric growth
//!
//! ## The Kelly Formula
//!
//! ```text
//! Kelly % = (Win Rate * Avg Win - Loss Rate * Avg Loss) / Avg Win
//!         = (p * b - q) / b
//!
//! Where:
//!   p = win probability
//!   q = loss probability (1 - p)
//!   b = win/loss ratio (avg win / avg loss)
//! ```
//!
//! ## Why Kelly?
//!
//! - **Maximizes geometric growth**: Optimal long-term capital growth
//! - **Prevents ruin**: Never risks 100% (unlike fixed %)
//! - **Adapts to performance**: Increases size when winning, reduces when losing
//! - **Mathematically proven**: Optimal for repeated bets with known odds
//!
//! ## Practical Adjustments
//!
//! Pure Kelly can be aggressive. We use:
//! - **Half Kelly** (default): Kelly% / 2 for safety
//! - **Quarter Kelly**: Kelly% / 4 for very conservative
//! - **Caps**: Never exceed max position size (e.g., 10% of portfolio)
//!
//! ## Human Insight
//!
//! Banks use sophisticated risk models, but Kelly is SIMPLE and PROVEN.
//! It automatically:
//! - Reduces size during losing streaks (preserves capital)
//! - Increases size during winning streaks (maximizes gains)
//! - Accounts for win rate AND win/loss ratio

use crate::types::{Confidence, Price};
use serde::{Deserialize, Serialize};

/// Kelly Criterion configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KellyConfig {
    /// Kelly fraction multiplier (0.5 = Half Kelly, 0.25 = Quarter Kelly)
    pub kelly_fraction: f64,

    /// Minimum win rate to use Kelly (below this, use fixed %)
    pub min_win_rate: f64,

    /// Maximum position size as % of portfolio (cap)
    pub max_position_pct: f64,

    /// Minimum position size as % of portfolio (floor)
    pub min_position_pct: f64,

    /// Minimum number of trades before using Kelly (use fixed % until then)
    pub min_trades_for_kelly: usize,

    /// Default position size when not enough history (% of portfolio)
    pub default_position_pct: f64,
}

impl Default for KellyConfig {
    fn default() -> Self {
        KellyConfig {
            kelly_fraction: 0.5,       // Half Kelly (conservative)
            min_win_rate: 0.40,        // Need at least 40% win rate
            max_position_pct: 10.0,    // Never more than 10% per position
            min_position_pct: 0.5,     // Never less than 0.5%
            min_trades_for_kelly: 20,  // Need 20 trades minimum for stats
            default_position_pct: 2.0, // Use 2% until we have history
        }
    }
}

/// Trading statistics for Kelly calculation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TradingStats {
    /// Total number of trades
    pub total_trades: usize,

    /// Number of winning trades
    pub wins: usize,

    /// Number of losing trades
    pub losses: usize,

    /// Sum of all winning trade P&Ls
    pub total_win_amount: f64,

    /// Sum of all losing trade P&Ls (absolute value)
    pub total_loss_amount: f64,
}

impl TradingStats {
    /// Create new empty stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a trade result
    pub fn record_trade(&mut self, pnl: f64) {
        self.total_trades += 1;

        if pnl > 0.0 {
            self.wins += 1;
            self.total_win_amount += pnl;
        } else if pnl < 0.0 {
            self.losses += 1;
            self.total_loss_amount += pnl.abs();
        }
        // Breakeven trades (pnl == 0) don't count as wins or losses
    }

    /// Calculate win rate (0.0 to 1.0)
    pub fn win_rate(&self) -> Option<f64> {
        if self.total_trades == 0 {
            return None;
        }
        Some(self.wins as f64 / self.total_trades as f64)
    }

    /// Calculate average win
    pub fn avg_win(&self) -> Option<f64> {
        if self.wins == 0 {
            return None;
        }
        Some(self.total_win_amount / self.wins as f64)
    }

    /// Calculate average loss
    pub fn avg_loss(&self) -> Option<f64> {
        if self.losses == 0 {
            return None;
        }
        Some(self.total_loss_amount / self.losses as f64)
    }

    /// Calculate win/loss ratio (b in Kelly formula)
    pub fn win_loss_ratio(&self) -> Option<f64> {
        let avg_win = self.avg_win()?;
        let avg_loss = self.avg_loss()?;

        if avg_loss == 0.0 {
            return None;
        }

        Some(avg_win / avg_loss)
    }

    /// Check if we have enough data for Kelly
    pub fn has_sufficient_data(&self, min_trades: usize) -> bool {
        self.total_trades >= min_trades && self.wins > 0 && self.losses > 0
    }
}

/// Kelly Criterion calculator
pub struct KellyCriterion {
    config: KellyConfig,
    stats: TradingStats,
}

impl KellyCriterion {
    /// Create new Kelly calculator with default config
    pub fn new() -> Self {
        Self::with_config(KellyConfig::default())
    }

    /// Create with custom config
    pub fn with_config(config: KellyConfig) -> Self {
        KellyCriterion {
            config,
            stats: TradingStats::new(),
        }
    }

    /// Record a trade result
    pub fn record_trade(&mut self, pnl: f64) {
        self.stats.record_trade(pnl);
    }

    /// Get current trading stats
    pub fn stats(&self) -> &TradingStats {
        &self.stats
    }

    /// Calculate Kelly percentage
    ///
    /// Returns None if insufficient data
    pub fn kelly_pct(&self) -> Option<f64> {
        // Need sufficient trading history
        if !self
            .stats
            .has_sufficient_data(self.config.min_trades_for_kelly)
        {
            return None;
        }

        let win_rate = self.stats.win_rate()?;
        let loss_rate = 1.0 - win_rate;
        let win_loss_ratio = self.stats.win_loss_ratio()?;

        // Need minimum win rate
        if win_rate < self.config.min_win_rate {
            return None;
        }

        // Kelly formula: (p * b - q) / b
        let kelly_raw = (win_rate * win_loss_ratio - loss_rate) / win_loss_ratio;

        // Apply Kelly fraction (e.g., half Kelly)
        let kelly_adjusted = kelly_raw * self.config.kelly_fraction;

        // Ensure positive
        if kelly_adjusted <= 0.0 {
            return None;
        }

        Some(kelly_adjusted * 100.0) // Convert to percentage
    }

    /// Calculate position size in dollars
    ///
    /// # Arguments
    ///
    /// * `portfolio_value` - Current portfolio value
    /// * `signal_confidence` - Signal confidence (0.0 to 1.0)
    ///
    /// # Returns
    ///
    /// Position size in dollars, capped by config limits
    pub fn position_size(
        &self,
        portfolio_value: f64,
        signal_confidence: Option<Confidence>,
    ) -> f64 {
        // Get base position size
        let base_pct = self.kelly_pct().unwrap_or(self.config.default_position_pct);

        // Adjust for signal confidence (scale down if low confidence)
        let adjusted_pct = if let Some(conf) = signal_confidence {
            // Scale Kelly by confidence
            // High confidence (0.9) → use full Kelly
            // Medium confidence (0.6) → use 60% of Kelly
            // Low confidence (0.4) → use 40% of Kelly
            base_pct * conf.value()
        } else {
            base_pct
        };

        // Apply caps
        let capped_pct = adjusted_pct
            .max(self.config.min_position_pct)
            .min(self.config.max_position_pct);

        // Convert to dollars
        (capped_pct / 100.0) * portfolio_value
    }

    /// Calculate quantity to trade
    ///
    /// # Arguments
    ///
    /// * `portfolio_value` - Current portfolio value
    /// * `current_price` - Current price of the asset
    /// * `signal_confidence` - Optional signal confidence
    ///
    /// # Returns
    ///
    /// Quantity to buy (always positive, use Quantity::buy())
    pub fn calculate_quantity(
        &self,
        portfolio_value: f64,
        current_price: Price,
        signal_confidence: Option<Confidence>,
    ) -> u64 {
        let position_value = self.position_size(portfolio_value, signal_confidence);
        let shares = position_value / current_price.value();

        // Round down to avoid over-allocation
        shares.floor() as u64
    }

    /// Get recommended position size as a summary
    pub fn position_summary(
        &self,
        portfolio_value: f64,
        current_price: Price,
        signal_confidence: Option<Confidence>,
    ) -> PositionSummary {
        let position_value = self.position_size(portfolio_value, signal_confidence);
        let quantity = self.calculate_quantity(portfolio_value, current_price, signal_confidence);
        let position_pct = (position_value / portfolio_value) * 100.0;

        let sizing_method = if self.kelly_pct().is_some() {
            SizingMethod::Kelly
        } else {
            SizingMethod::Fixed
        };

        PositionSummary {
            quantity,
            position_value,
            position_pct,
            sizing_method,
            kelly_pct: self.kelly_pct(),
            trades_completed: self.stats.total_trades,
            win_rate: self.stats.win_rate(),
        }
    }
}

impl Default for KellyCriterion {
    fn default() -> Self {
        Self::new()
    }
}

/// Position sizing method used
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizingMethod {
    /// Using Kelly Criterion (sufficient history)
    Kelly,
    /// Using fixed percentage (insufficient history)
    Fixed,
}

/// Summary of recommended position
#[derive(Debug, Clone)]
pub struct PositionSummary {
    /// Number of shares to buy
    pub quantity: u64,

    /// Total position value in dollars
    pub position_value: f64,

    /// Position as % of portfolio
    pub position_pct: f64,

    /// Sizing method used
    pub sizing_method: SizingMethod,

    /// Kelly percentage (if available)
    pub kelly_pct: Option<f64>,

    /// Number of trades in history
    pub trades_completed: usize,

    /// Win rate (if available)
    pub win_rate: Option<f64>,
}

impl std::fmt::Display for PositionSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Qty: {} shares (${:.2}, {:.2}% of portfolio) ",
            self.quantity, self.position_value, self.position_pct
        )?;

        match self.sizing_method {
            SizingMethod::Kelly => {
                write!(f, "[Kelly: {:.2}%]", self.kelly_pct.unwrap_or(0.0))?;
                if let Some(wr) = self.win_rate {
                    write!(f, " WinRate: {:.1}%", wr * 100.0)?;
                }
            }
            SizingMethod::Fixed => {
                write!(
                    f,
                    "[Fixed] (Need {} trades for Kelly)",
                    self.trades_completed
                )?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trading_stats_basic() {
        let mut stats = TradingStats::new();

        // Record some trades
        stats.record_trade(100.0); // Win
        stats.record_trade(-50.0); // Loss
        stats.record_trade(150.0); // Win

        assert_eq!(stats.total_trades, 3);
        assert_eq!(stats.wins, 2);
        assert_eq!(stats.losses, 1);
        assert_eq!(stats.win_rate(), Some(2.0 / 3.0));
        assert_eq!(stats.avg_win(), Some(125.0)); // (100 + 150) / 2
        assert_eq!(stats.avg_loss(), Some(50.0));
    }

    #[test]
    fn test_win_loss_ratio() {
        let mut stats = TradingStats::new();

        stats.record_trade(200.0); // Win
        stats.record_trade(-100.0); // Loss

        // Win/Loss ratio = 200 / 100 = 2.0
        assert_eq!(stats.win_loss_ratio(), Some(2.0));
    }

    #[test]
    fn test_kelly_calculation() {
        let config = KellyConfig {
            min_trades_for_kelly: 5,
            ..Default::default()
        };
        let mut kelly = KellyCriterion::with_config(config);

        // Simulate profitable strategy: 60% win rate, 2:1 win/loss
        kelly.record_trade(200.0); // Win
        kelly.record_trade(-100.0); // Loss
        kelly.record_trade(200.0); // Win
        kelly.record_trade(200.0); // Win
        kelly.record_trade(-100.0); // Loss

        // Kelly = (p * b - q) / b
        // p = 0.6, q = 0.4, b = 2.0
        // Kelly = (0.6 * 2.0 - 0.4) / 2.0 = 0.8 / 2.0 = 0.4 = 40%
        // Half Kelly = 20%

        let kelly_pct = kelly.kelly_pct();
        assert!(kelly_pct.is_some());

        let kelly_val = kelly_pct.unwrap();
        // Should be around 20% (half of 40%)
        assert!((kelly_val - 20.0).abs() < 1.0, "Kelly: {}", kelly_val);
    }

    #[test]
    fn test_insufficient_trades() {
        let kelly = KellyCriterion::new();

        // No trades yet
        assert!(kelly.kelly_pct().is_none());

        // Should use default position size
        let size = kelly.position_size(10_000.0, None);
        // Default is 2% of 10k = $200
        assert_eq!(size, 200.0);
    }

    #[test]
    fn test_position_size_with_confidence() {
        let config = KellyConfig {
            min_trades_for_kelly: 5,
            max_position_pct: 50.0, // High cap so we don't hit it
            ..Default::default()
        };
        let mut kelly = KellyCriterion::with_config(config);

        // Build history
        for _ in 0..3 {
            kelly.record_trade(200.0); // Win
        }
        for _ in 0..2 {
            kelly.record_trade(-100.0); // Loss
        }

        let portfolio = 10_000.0;

        // High confidence signal (0.9)
        let high_conf = Confidence::new(0.9).unwrap();
        let size_high = kelly.position_size(portfolio, Some(high_conf));

        // Medium confidence signal (0.6)
        let med_conf = Confidence::new(0.6).unwrap();
        let size_med = kelly.position_size(portfolio, Some(med_conf));

        // Low confidence signal (0.4)
        let low_conf = Confidence::new(0.4).unwrap();
        let size_low = kelly.position_size(portfolio, Some(low_conf));

        // High confidence should get larger size than medium
        // Sizes scale with confidence: 0.9x Kelly > 0.6x Kelly > 0.4x Kelly
        assert!(
            size_high > size_med,
            "High conf ({}) should > med conf ({})",
            size_high,
            size_med
        );
        assert!(
            size_med > size_low,
            "Med conf ({}) should > low conf ({})",
            size_med,
            size_low
        );
    }

    #[test]
    fn test_quantity_calculation() {
        let config = KellyConfig {
            default_position_pct: 5.0, // 5% default
            ..Default::default()
        };
        let kelly = KellyCriterion::with_config(config);

        let portfolio = 10_000.0;
        let price = Price::new(100.0).unwrap();

        // 5% of $10k = $500
        // $500 / $100/share = 5 shares
        let qty = kelly.calculate_quantity(portfolio, price, None);
        assert_eq!(qty, 5);
    }

    #[test]
    fn test_position_caps() {
        let config = KellyConfig {
            max_position_pct: 5.0, // Cap at 5%
            min_trades_for_kelly: 2,
            ..Default::default()
        };
        let mut kelly = KellyCriterion::with_config(config);

        // Extremely profitable strategy (would normally suggest huge Kelly)
        kelly.record_trade(1000.0);
        kelly.record_trade(-10.0);
        kelly.record_trade(1000.0);

        let portfolio = 10_000.0;

        // Even with great stats, should cap at 5%
        let size = kelly.position_size(portfolio, None);
        let size_pct = (size / portfolio) * 100.0;

        assert!(size_pct <= 5.1); // Allow small rounding
    }

    #[test]
    fn test_negative_kelly() {
        let config = KellyConfig {
            min_trades_for_kelly: 5,
            min_win_rate: 0.3,
            ..Default::default()
        };
        let mut kelly = KellyCriterion::with_config(config);

        // Losing strategy: 20% win rate, 1:1 ratio
        kelly.record_trade(100.0); // Win
        kelly.record_trade(-100.0); // Loss
        kelly.record_trade(-100.0); // Loss
        kelly.record_trade(-100.0); // Loss
        kelly.record_trade(-100.0); // Loss

        // Kelly would be negative → should return None
        assert!(kelly.kelly_pct().is_none());

        // Should fallback to default
        let size = kelly.position_size(10_000.0, None);
        assert_eq!(size, 200.0); // 2% default
    }

    #[test]
    fn test_position_summary() {
        let mut kelly = KellyCriterion::new();

        let portfolio = 10_000.0;
        let price = Price::new(50.0).unwrap();
        let confidence = Confidence::new(0.8).unwrap();

        let summary = kelly.position_summary(portfolio, price, Some(confidence));

        assert_eq!(summary.sizing_method, SizingMethod::Fixed);
        assert!(summary.kelly_pct.is_none());
        assert_eq!(summary.trades_completed, 0);

        // Add some history
        for _ in 0..30 {
            kelly.record_trade(100.0);
        }
        for _ in 0..20 {
            kelly.record_trade(-80.0);
        }

        let summary2 = kelly.position_summary(portfolio, price, Some(confidence));
        assert_eq!(summary2.sizing_method, SizingMethod::Kelly);
        assert!(summary2.kelly_pct.is_some());
    }
}
