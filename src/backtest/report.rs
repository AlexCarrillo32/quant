//! Backtesting Report - Human-Readable Results
//!
//! **Purpose**: Generate comprehensive backtest reports

use crate::backtest::engine::BacktestResult;
use crate::backtest::trade::TradeOutcome;
use crate::types::Price;
use std::fmt;

/// Formatted backtest report
pub struct BacktestReport<'a> {
    result: &'a BacktestResult,
}

impl<'a> BacktestReport<'a> {
    /// Create a new report
    pub fn new(result: &'a BacktestResult) -> Self {
        Self { result }
    }

    /// Generate a text report
    pub fn to_text(&self) -> String {
        let mut report = String::new();

        report.push_str("═══════════════════════════════════════════════════════════════\n");
        report.push_str("                    BACKTEST REPORT                            \n");
        report.push_str("═══════════════════════════════════════════════════════════════\n\n");

        // Performance Overview
        report.push_str("PERFORMANCE OVERVIEW\n");
        report.push_str("───────────────────────────────────────────────────────────────\n");
        report.push_str(&format!(
            "  Total Return:         {:>10.2}%\n",
            self.result.metrics.total_return_pct
        ));
        report.push_str(&format!(
            "  Annualized Return:    {:>10.2}%\n",
            self.result.metrics.annualized_return_pct
        ));
        report.push_str(&format!(
            "  Sharpe Ratio:         {:>10.2}\n",
            self.result.metrics.sharpe_ratio
        ));
        report.push_str(&format!(
            "  Sortino Ratio:        {:>10.2}\n",
            self.result.metrics.sortino_ratio
        ));
        report.push_str(&format!(
            "  Max Drawdown:         {:>10.2}%\n",
            self.result.metrics.max_drawdown_pct
        ));
        report.push_str(&format!(
            "  Calmar Ratio:         {:>10.2}\n",
            self.result.metrics.calmar_ratio
        ));
        report.push_str(&format!(
            "  Strategy Grade:       {:>10}\n",
            self.result.metrics.grade()
        ));
        report.push_str("\n");

        // Trade Statistics
        let stats = &self.result.metrics.trade_stats;
        report.push_str("TRADE STATISTICS\n");
        report.push_str("───────────────────────────────────────────────────────────────\n");
        report.push_str(&format!(
            "  Total Trades:         {:>10}\n",
            stats.total_trades
        ));
        report.push_str(&format!(
            "  Winning Trades:       {:>10}\n",
            stats.winning_trades
        ));
        report.push_str(&format!(
            "  Losing Trades:        {:>10}\n",
            stats.losing_trades
        ));
        report.push_str(&format!(
            "  Win Rate:             {:>10.2}%\n",
            stats.win_rate_pct
        ));
        report.push_str(&format!(
            "  Profit Factor:        {:>10.2}\n",
            stats.profit_factor
        ));
        report.push_str(&format!(
            "  Expectancy:           {:>10.2}\n",
            stats.expectancy
        ));
        report.push_str("\n");

        // Win/Loss Analysis
        report.push_str("WIN/LOSS ANALYSIS\n");
        report.push_str("───────────────────────────────────────────────────────────────\n");
        report.push_str(&format!(
            "  Average Win:          ${:>9.2}\n",
            stats.avg_win
        ));
        report.push_str(&format!(
            "  Average Loss:         ${:>9.2}\n",
            stats.avg_loss
        ));
        report.push_str(&format!(
            "  Largest Win:          ${:>9.2}\n",
            stats.largest_win
        ));
        report.push_str(&format!(
            "  Largest Loss:         ${:>9.2}\n",
            stats.largest_loss
        ));
        report.push_str(&format!(
            "  Avg R:R Ratio:        {:>10.2}\n",
            stats.avg_risk_reward
        ));
        report.push_str(&format!(
            "  Avg Hold Time:        {:>10.2}h\n",
            stats.avg_hold_time_hours
        ));
        report.push_str("\n");

        // Streaks
        report.push_str("STREAKS\n");
        report.push_str("───────────────────────────────────────────────────────────────\n");
        report.push_str(&format!(
            "  Max Consecutive Wins:  {:>9}\n",
            stats.max_consecutive_wins
        ));
        report.push_str(&format!(
            "  Max Consecutive Losses:{:>9}\n",
            stats.max_consecutive_losses
        ));
        report.push_str("\n");

        // Signal Analysis
        report.push_str("SIGNAL ANALYSIS\n");
        report.push_str("───────────────────────────────────────────────────────────────\n");
        report.push_str(&format!(
            "  Total Signals:        {:>10}\n",
            self.result.total_signals
        ));
        report.push_str(&format!(
            "  Rejected Signals:     {:>10}\n",
            self.result.rejected_signals
        ));
        let acceptance_rate = if self.result.total_signals > 0 {
            ((self.result.total_signals - self.result.rejected_signals) as f64
                / self.result.total_signals as f64)
                * 100.0
        } else {
            0.0
        };
        report.push_str(&format!(
            "  Acceptance Rate:      {:>10.2}%\n",
            acceptance_rate
        ));
        report.push_str("\n");

        // Capital Evolution
        report.push_str("CAPITAL EVOLUTION\n");
        report.push_str("───────────────────────────────────────────────────────────────\n");
        let initial = self.result.equity_curve.first().copied().unwrap_or(0.0);
        report.push_str(&format!("  Initial Capital:      ${:>9.2}\n", initial));
        report.push_str(&format!(
            "  Final Capital:        ${:>9.2}\n",
            self.result.final_capital
        ));
        report.push_str(&format!(
            "  Profit/Loss:          ${:>9.2}\n",
            self.result.final_capital - initial
        ));
        report.push_str("\n");

        // Top Trades
        if !self.result.trades.is_empty() {
            report.push_str("TOP 5 WINNING TRADES\n");
            report.push_str("───────────────────────────────────────────────────────────────\n");

            let mut winning_trades: Vec<_> = self
                .result
                .trades
                .iter()
                .filter(|t| t.outcome() == Some(TradeOutcome::Winner))
                .collect();

            winning_trades.sort_by(|a, b| {
                b.net_pnl
                    .unwrap_or(0.0)
                    .partial_cmp(&a.net_pnl.unwrap_or(0.0))
                    .unwrap()
            });

            for (i, trade) in winning_trades.iter().take(5).enumerate() {
                let exit_price = trade.exit_price.unwrap_or_else(|| Price::new(0.0).unwrap());
                report.push_str(&format!(
                    "  {}. {} {} @ ${:.2} → ${:.2} = ${:.2}\n",
                    i + 1,
                    trade.symbol,
                    match trade.action {
                        crate::types::SignalAction::Buy => "BUY ",
                        crate::types::SignalAction::Sell => "SELL",
                        crate::types::SignalAction::Close => "CLOS",
                        crate::types::SignalAction::Hold => "HOLD",
                    },
                    trade.entry_price.value(),
                    exit_price.value(),
                    trade.net_pnl.unwrap_or(0.0)
                ));
            }
            report.push_str("\n");

            report.push_str("TOP 5 LOSING TRADES\n");
            report.push_str("───────────────────────────────────────────────────────────────\n");

            let mut losing_trades: Vec<_> = self
                .result
                .trades
                .iter()
                .filter(|t| t.outcome() == Some(TradeOutcome::Loser))
                .collect();

            losing_trades.sort_by(|a, b| {
                a.net_pnl
                    .unwrap_or(0.0)
                    .partial_cmp(&b.net_pnl.unwrap_or(0.0))
                    .unwrap()
            });

            for (i, trade) in losing_trades.iter().take(5).enumerate() {
                let exit_price = trade.exit_price.unwrap_or_else(|| Price::new(0.0).unwrap());
                report.push_str(&format!(
                    "  {}. {} {} @ ${:.2} → ${:.2} = ${:.2}\n",
                    i + 1,
                    trade.symbol,
                    match trade.action {
                        crate::types::SignalAction::Buy => "BUY ",
                        crate::types::SignalAction::Sell => "SELL",
                        crate::types::SignalAction::Close => "CLOS",
                        crate::types::SignalAction::Hold => "HOLD",
                    },
                    trade.entry_price.value(),
                    exit_price.value(),
                    trade.net_pnl.unwrap_or(0.0)
                ));
            }
            report.push_str("\n");
        }

        // Recommendation
        report.push_str("RECOMMENDATION\n");
        report.push_str("───────────────────────────────────────────────────────────────\n");
        if self.result.metrics.is_good_strategy() {
            report.push_str("  ✅ Strategy shows promising results!\n");
            report.push_str("  Consider:\n");
            report.push_str("    - Forward testing on recent data\n");
            report.push_str("    - Paper trading\n");
            report.push_str("    - Parameter optimization\n");
        } else {
            report.push_str("  ⚠️  Strategy needs improvement.\n");
            report.push_str("  Suggestions:\n");
            if stats.win_rate_pct < 40.0 {
                report.push_str("    - Improve signal quality (win rate is low)\n");
            }
            if stats.profit_factor < 1.5 {
                report.push_str("    - Reduce losing trade sizes\n");
            }
            if self.result.metrics.max_drawdown_pct < -20.0 {
                report.push_str("    - Add stricter risk management\n");
            }
            if self.result.metrics.sharpe_ratio < 1.0 {
                report.push_str("    - Reduce volatility or improve returns\n");
            }
        }

        report.push_str("\n");
        report.push_str("═══════════════════════════════════════════════════════════════\n");

        report
    }

    /// Print the report to stdout
    pub fn print(&self) {
        println!("{}", self.to_text());
    }
}

impl<'a> fmt::Display for BacktestReport<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_text())
    }
}
