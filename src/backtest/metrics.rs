//! Performance Metrics - Measure Strategy Quality
//!
//! **Key Metrics**:
//! - Sharpe Ratio: Risk-adjusted returns
//! - Sortino Ratio: Downside risk-adjusted returns
//! - Maximum Drawdown: Worst peak-to-trough decline
//! - Win Rate: % of profitable trades
//! - Profit Factor: Gross profit / gross loss
//! - Expectancy: Average $ per trade

use crate::backtest::trade::{BacktestTrade, TradeOutcome};

/// Performance metrics for a backtest
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Total return (%)
    pub total_return_pct: f64,

    /// Annualized return (%)
    pub annualized_return_pct: f64,

    /// Sharpe ratio (risk-adjusted return)
    pub sharpe_ratio: f64,

    /// Sortino ratio (downside risk-adjusted return)
    pub sortino_ratio: f64,

    /// Maximum drawdown (%)
    pub max_drawdown_pct: f64,

    /// Calmar ratio (return / max drawdown)
    pub calmar_ratio: f64,

    /// Number of trading days
    pub trading_days: usize,

    /// Trade statistics
    pub trade_stats: TradeStatistics,
}

/// Statistics about trades
#[derive(Debug, Clone)]
pub struct TradeStatistics {
    /// Total number of trades
    pub total_trades: usize,

    /// Number of winning trades
    pub winning_trades: usize,

    /// Number of losing trades
    pub losing_trades: usize,

    /// Win rate (%)
    pub win_rate_pct: f64,

    /// Average winning trade ($)
    pub avg_win: f64,

    /// Average losing trade ($)
    pub avg_loss: f64,

    /// Largest winning trade ($)
    pub largest_win: f64,

    /// Largest losing trade ($)
    pub largest_loss: f64,

    /// Profit factor (gross profit / gross loss)
    pub profit_factor: f64,

    /// Expectancy (average $ per trade)
    pub expectancy: f64,

    /// Average R:R ratio
    pub avg_risk_reward: f64,

    /// Average hold time (hours)
    pub avg_hold_time_hours: f64,

    /// Maximum consecutive wins
    pub max_consecutive_wins: usize,

    /// Maximum consecutive losses
    pub max_consecutive_losses: usize,
}

impl PerformanceMetrics {
    /// Calculate metrics from equity curve and trades
    pub fn calculate(
        equity_curve: &[f64],
        trades: &[BacktestTrade],
        initial_capital: f64,
        days: usize,
    ) -> Self {
        let trade_stats = TradeStatistics::calculate(trades);

        // Total return
        let final_equity = equity_curve.last().copied().unwrap_or(initial_capital);
        let total_return_pct = ((final_equity - initial_capital) / initial_capital) * 100.0;

        // Annualized return (assuming 252 trading days per year)
        let years = days as f64 / 252.0;
        let annualized_return_pct = if years > 0.0 {
            (((final_equity / initial_capital).powf(1.0 / years)) - 1.0) * 100.0
        } else {
            0.0
        };

        // Calculate returns for each period
        let returns = calculate_returns(equity_curve);

        // Sharpe ratio (assuming risk-free rate of 2%)
        let sharpe_ratio = calculate_sharpe_ratio(&returns, 0.02);

        // Sortino ratio (only downside volatility)
        let sortino_ratio = calculate_sortino_ratio(&returns, 0.02);

        // Maximum drawdown
        let max_drawdown_pct = calculate_max_drawdown(equity_curve);

        // Calmar ratio (return / max drawdown)
        let calmar_ratio = if max_drawdown_pct.abs() > 0.01 {
            annualized_return_pct / max_drawdown_pct.abs()
        } else {
            0.0
        };

        Self {
            total_return_pct,
            annualized_return_pct,
            sharpe_ratio,
            sortino_ratio,
            max_drawdown_pct,
            calmar_ratio,
            trading_days: days,
            trade_stats,
        }
    }

    /// Is this a good strategy? (basic heuristic)
    pub fn is_good_strategy(&self) -> bool {
        self.sharpe_ratio > 1.0
            && self.max_drawdown_pct > -20.0
            && self.trade_stats.win_rate_pct > 40.0
            && self.trade_stats.profit_factor > 1.5
    }

    /// Get a letter grade for this strategy
    pub fn grade(&self) -> &'static str {
        if self.sharpe_ratio > 3.0 && self.trade_stats.profit_factor > 3.0 {
            "A+"
        } else if self.sharpe_ratio > 2.0 && self.trade_stats.profit_factor > 2.5 {
            "A"
        } else if self.sharpe_ratio > 1.5 && self.trade_stats.profit_factor > 2.0 {
            "B"
        } else if self.sharpe_ratio > 1.0 && self.trade_stats.profit_factor > 1.5 {
            "C"
        } else if self.sharpe_ratio > 0.5 {
            "D"
        } else {
            "F"
        }
    }
}

impl TradeStatistics {
    /// Calculate trade statistics
    pub fn calculate(trades: &[BacktestTrade]) -> Self {
        let closed_trades: Vec<_> = trades.iter().filter(|t| !t.is_open()).collect();

        if closed_trades.is_empty() {
            return Self::default();
        }

        let total_trades = closed_trades.len();

        // Separate winners and losers
        let winners: Vec<_> = closed_trades
            .iter()
            .filter(|t| t.outcome() == Some(TradeOutcome::Winner))
            .collect();

        let losers: Vec<_> = closed_trades
            .iter()
            .filter(|t| t.outcome() == Some(TradeOutcome::Loser))
            .collect();

        let winning_trades = winners.len();
        let losing_trades = losers.len();

        let win_rate_pct = (winning_trades as f64 / total_trades as f64) * 100.0;

        // Calculate average wins/losses
        let total_wins: f64 = winners.iter().filter_map(|t| t.net_pnl).sum();
        let total_losses: f64 = losers.iter().filter_map(|t| t.net_pnl).sum();

        let avg_win = if winning_trades > 0 {
            total_wins / winning_trades as f64
        } else {
            0.0
        };

        let avg_loss = if losing_trades > 0 {
            total_losses / losing_trades as f64
        } else {
            0.0
        };

        // Find largest win/loss
        let largest_win = winners
            .iter()
            .filter_map(|t| t.net_pnl)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        let largest_loss = losers
            .iter()
            .filter_map(|t| t.net_pnl)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        // Profit factor
        let profit_factor = if total_losses.abs() > 0.01 {
            total_wins / total_losses.abs()
        } else {
            f64::INFINITY
        };

        // Expectancy
        let total_pnl: f64 = closed_trades.iter().filter_map(|t| t.net_pnl).sum();
        let expectancy = total_pnl / total_trades as f64;

        // Average R:R
        let avg_risk_reward = closed_trades
            .iter()
            .filter_map(|t| t.risk_reward_ratio())
            .sum::<f64>()
            / total_trades as f64;

        // Average hold time
        let avg_hold_time_hours = closed_trades
            .iter()
            .filter_map(|t| t.hold_time_seconds())
            .sum::<f64>()
            / total_trades as f64
            / 3600.0;

        // Consecutive wins/losses
        let (max_consecutive_wins, max_consecutive_losses) =
            calculate_consecutive_streaks(&closed_trades);

        Self {
            total_trades,
            winning_trades,
            losing_trades,
            win_rate_pct,
            avg_win,
            avg_loss,
            largest_win,
            largest_loss,
            profit_factor,
            expectancy,
            avg_risk_reward,
            avg_hold_time_hours,
            max_consecutive_wins,
            max_consecutive_losses,
        }
    }
}

impl Default for TradeStatistics {
    fn default() -> Self {
        Self {
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            win_rate_pct: 0.0,
            avg_win: 0.0,
            avg_loss: 0.0,
            largest_win: 0.0,
            largest_loss: 0.0,
            profit_factor: 0.0,
            expectancy: 0.0,
            avg_risk_reward: 0.0,
            avg_hold_time_hours: 0.0,
            max_consecutive_wins: 0,
            max_consecutive_losses: 0,
        }
    }
}

/// Calculate period-to-period returns
fn calculate_returns(equity_curve: &[f64]) -> Vec<f64> {
    equity_curve
        .windows(2)
        .map(|w| (w[1] - w[0]) / w[0])
        .collect()
}

/// Calculate Sharpe ratio
fn calculate_sharpe_ratio(returns: &[f64], risk_free_rate: f64) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }

    let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance = returns
        .iter()
        .map(|r| (r - mean_return).powi(2))
        .sum::<f64>()
        / returns.len() as f64;
    let std_dev = variance.sqrt();

    if std_dev.abs() < 1e-10 {
        return 0.0;
    }

    // Annualize (assuming daily returns)
    let annualized_return = mean_return * 252.0;
    let annualized_vol = std_dev * (252.0_f64).sqrt();
    let annualized_rf = risk_free_rate;

    (annualized_return - annualized_rf) / annualized_vol
}

/// Calculate Sortino ratio (only downside deviation)
fn calculate_sortino_ratio(returns: &[f64], risk_free_rate: f64) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }

    let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;

    // Only negative returns count as downside risk
    let downside_returns: Vec<f64> = returns.iter().filter(|&&r| r < 0.0).copied().collect();

    if downside_returns.is_empty() {
        return f64::INFINITY; // No downside = infinite Sortino
    }

    let downside_variance = downside_returns
        .iter()
        .map(|r| r.powi(2))
        .sum::<f64>()
        / downside_returns.len() as f64;
    let downside_dev = downside_variance.sqrt();

    if downside_dev.abs() < 1e-10 {
        return 0.0;
    }

    // Annualize
    let annualized_return = mean_return * 252.0;
    let annualized_downside_dev = downside_dev * (252.0_f64).sqrt();

    (annualized_return - risk_free_rate) / annualized_downside_dev
}

/// Calculate maximum drawdown
fn calculate_max_drawdown(equity_curve: &[f64]) -> f64 {
    if equity_curve.is_empty() {
        return 0.0;
    }

    let mut max_drawdown = 0.0;
    let mut peak = equity_curve[0];

    for &equity in equity_curve.iter() {
        if equity > peak {
            peak = equity;
        }

        let drawdown = ((equity - peak) / peak) * 100.0;
        if drawdown < max_drawdown {
            max_drawdown = drawdown;
        }
    }

    max_drawdown
}

/// Calculate consecutive win/loss streaks
fn calculate_consecutive_streaks(trades: &[&BacktestTrade]) -> (usize, usize) {
    let mut max_wins = 0;
    let mut max_losses = 0;
    let mut current_wins = 0;
    let mut current_losses = 0;

    for trade in trades {
        match trade.outcome() {
            Some(TradeOutcome::Winner) => {
                current_wins += 1;
                current_losses = 0;
                max_wins = max_wins.max(current_wins);
            }
            Some(TradeOutcome::Loser) => {
                current_losses += 1;
                current_wins = 0;
                max_losses = max_losses.max(current_losses);
            }
            _ => {}
        }
    }

    (max_wins, max_losses)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use std::time::SystemTime;

    fn create_test_trade(net_pnl: f64) -> BacktestTrade {
        let mut trade = BacktestTrade::new(
            Symbol::new("TEST").unwrap(),
            SignalAction::Buy,
            Price::new(100.0).unwrap(),
            Quantity::buy(1),
            SystemTime::now(),
            0.0,
            0.0,
            0.5,
        );

        let exit_price = Price::new(100.0 + net_pnl).unwrap();
        trade.close(
            exit_price,
            SystemTime::now(),
            crate::backtest::trade::ExitReason::TakeProfit,
            0.0,
            0.0,
        );

        trade
    }

    #[test]
    fn test_trade_statistics() {
        let trades = vec![
            create_test_trade(10.0),  // win
            create_test_trade(-5.0),  // loss
            create_test_trade(15.0),  // win
            create_test_trade(-3.0),  // loss
            create_test_trade(20.0),  // win
        ];

        let stats = TradeStatistics::calculate(&trades);

        assert_eq!(stats.total_trades, 5);
        assert_eq!(stats.winning_trades, 3);
        assert_eq!(stats.losing_trades, 2);
        assert!((stats.win_rate_pct - 60.0).abs() < 0.1);

        // Average win: (10 + 15 + 20) / 3 = 15
        assert!((stats.avg_win - 15.0).abs() < 0.1);

        // Average loss: (-5 + -3) / 2 = -4
        assert!((stats.avg_loss - (-4.0)).abs() < 0.1);

        // Profit factor: 45 / 8 = 5.625
        assert!((stats.profit_factor - 5.625).abs() < 0.1);
    }

    #[test]
    fn test_max_drawdown() {
        let equity = vec![10000.0, 11000.0, 10500.0, 9000.0, 9500.0, 12000.0];

        let max_dd = calculate_max_drawdown(&equity);

        // Peak: 11000, trough: 9000
        // Drawdown: (9000 - 11000) / 11000 = -18.18%
        assert!((max_dd - (-18.18)).abs() < 0.1);
    }

    #[test]
    fn test_sharpe_ratio() {
        // Returns: 5%, -2%, 3%, 6%, -1%
        let returns = vec![0.05, -0.02, 0.03, 0.06, -0.01];

        let sharpe = calculate_sharpe_ratio(&returns, 0.02);

        // Mean: 2.2% per period
        // Std: ~3%
        // Annualized Sharpe should be positive
        assert!(sharpe > 0.0);
        assert!(sharpe < 100.0); // Sanity check (relaxed bound)
    }
}
