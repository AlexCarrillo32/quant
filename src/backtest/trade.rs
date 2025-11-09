//! Trade Tracking for Backtesting
//!
//! Records individual trades with full context for analysis

use crate::types::*;
use std::time::SystemTime;

/// A trade executed during backtesting
#[derive(Debug, Clone)]
pub struct BacktestTrade {
    /// Symbol traded
    pub symbol: Symbol,

    /// Trade direction
    pub action: SignalAction,

    /// Entry price (including slippage)
    pub entry_price: Price,

    /// Exit price (including slippage)
    pub exit_price: Option<Price>,

    /// Quantity traded
    pub quantity: Quantity,

    /// Entry timestamp
    pub entry_time: SystemTime,

    /// Exit timestamp
    pub exit_time: Option<SystemTime>,

    /// Commission paid
    pub commission: f64,

    /// Slippage experienced (actual - expected price)
    pub slippage: f64,

    /// Why we entered (alpha signal confidence)
    pub entry_confidence: f64,

    /// Why we exited (stop loss, take profit, signal)
    pub exit_reason: Option<ExitReason>,

    /// P&L for this trade (gross, before commission)
    pub gross_pnl: Option<f64>,

    /// P&L for this trade (net, after commission)
    pub net_pnl: Option<f64>,
}

/// Reason for exiting a trade
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitReason {
    /// Stop loss hit
    StopLoss,

    /// Take profit hit
    TakeProfit,

    /// Alpha signal reversed
    SignalReverse,

    /// Risk manager forced exit
    RiskManagement,

    /// End of backtest period
    EndOfData,

    /// Time-based exit
    TimeExit,
}

/// Outcome of a closed trade
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeOutcome {
    /// Trade was profitable
    Winner,

    /// Trade was unprofitable
    Loser,

    /// Trade broke even
    Breakeven,
}

impl BacktestTrade {
    /// Create a new trade at entry
    pub fn new(
        symbol: Symbol,
        action: SignalAction,
        entry_price: Price,
        quantity: Quantity,
        entry_time: SystemTime,
        commission: f64,
        slippage: f64,
        entry_confidence: f64,
    ) -> Self {
        Self {
            symbol,
            action,
            entry_price,
            exit_price: None,
            quantity,
            entry_time,
            exit_time: None,
            commission,
            slippage,
            entry_confidence,
            exit_reason: None,
            gross_pnl: None,
            net_pnl: None,
        }
    }

    /// Close the trade
    pub fn close(
        &mut self,
        exit_price: Price,
        exit_time: SystemTime,
        exit_reason: ExitReason,
        exit_commission: f64,
        exit_slippage: f64,
    ) {
        self.exit_price = Some(exit_price);
        self.exit_time = Some(exit_time);
        self.exit_reason = Some(exit_reason);

        // Calculate P&L
        let price_diff = match self.action {
            SignalAction::Buy => exit_price.value() - self.entry_price.value(),
            SignalAction::Sell => self.entry_price.value() - exit_price.value(),
            SignalAction::Hold | SignalAction::Close => 0.0,
        };

        let gross = price_diff * self.quantity.value() as f64;
        let total_commission = self.commission + exit_commission;
        let total_slippage = self.slippage + exit_slippage;

        self.gross_pnl = Some(gross);
        self.net_pnl = Some(gross - total_commission - total_slippage);
    }

    /// Is this trade still open?
    pub fn is_open(&self) -> bool {
        self.exit_price.is_none()
    }

    /// Get the outcome of this trade
    pub fn outcome(&self) -> Option<TradeOutcome> {
        self.net_pnl.map(|pnl| {
            if pnl > 0.01 {
                TradeOutcome::Winner
            } else if pnl < -0.01 {
                TradeOutcome::Loser
            } else {
                TradeOutcome::Breakeven
            }
        })
    }

    /// Get the return % for this trade
    pub fn return_pct(&self) -> Option<f64> {
        self.net_pnl.map(|pnl| {
            let entry_value = self.entry_price.value() * self.quantity.value() as f64;
            (pnl / entry_value) * 100.0
        })
    }

    /// Get the hold time in seconds
    pub fn hold_time_seconds(&self) -> Option<f64> {
        self.exit_time.map(|exit| {
            exit.duration_since(self.entry_time)
                .unwrap_or_default()
                .as_secs_f64()
        })
    }

    /// Get the risk/reward ratio (actual)
    pub fn risk_reward_ratio(&self) -> Option<f64> {
        self.net_pnl.map(|pnl| {
            let entry_value = self.entry_price.value() * self.quantity.value() as f64;
            let max_risk = entry_value * 0.01; // Assume 1% risk (should come from signal)
            if max_risk.abs() < 0.01 {
                0.0
            } else {
                pnl / max_risk
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_winning_trade() {
        let mut trade = BacktestTrade::new(
            Symbol::new("AAPL").unwrap(),
            SignalAction::Buy,
            Price::new(100.0).unwrap(),
            Quantity::buy(10),
            SystemTime::now(),
            1.0,  // commission
            0.1,  // slippage
            0.85, // confidence
        );

        assert!(trade.is_open());

        // Close at profit
        trade.close(
            Price::new(110.0).unwrap(),
            SystemTime::now(),
            ExitReason::TakeProfit,
            1.0,
            0.1,
        );

        assert!(!trade.is_open());
        assert_eq!(trade.outcome(), Some(TradeOutcome::Winner));

        // (110 - 100) * 10 = 100 gross
        // 100 - 2.0 (commission) - 0.2 (slippage) = 97.8 net
        let net_pnl = trade.net_pnl.unwrap();
        assert!((net_pnl - 97.8).abs() < 0.01);
    }

    #[test]
    fn test_losing_trade() {
        let mut trade = BacktestTrade::new(
            Symbol::new("TSLA").unwrap(),
            SignalAction::Sell,
            Price::new(200.0).unwrap(),
            Quantity::buy(5),
            SystemTime::now(),
            2.0,
            0.5,
            0.75,
        );

        // Close at loss (sold at 200, buy back at 210)
        trade.close(
            Price::new(210.0).unwrap(),
            SystemTime::now(),
            ExitReason::StopLoss,
            2.0,
            0.5,
        );

        assert_eq!(trade.outcome(), Some(TradeOutcome::Loser));

        // (200 - 210) * 5 = -50 gross
        // -50 - 4.0 - 1.0 = -55 net
        let net_pnl = trade.net_pnl.unwrap();
        assert!((net_pnl - (-55.0)).abs() < 0.01);
    }

    #[test]
    fn test_return_pct() {
        let mut trade = BacktestTrade::new(
            Symbol::new("AAPL").unwrap(),
            SignalAction::Buy,
            Price::new(100.0).unwrap(),
            Quantity::buy(10),
            SystemTime::now(),
            0.0,
            0.0,
            0.85,
        );

        trade.close(
            Price::new(105.0).unwrap(),
            SystemTime::now(),
            ExitReason::TakeProfit,
            0.0,
            0.0,
        );

        // Entry value: 100 * 10 = 1000
        // Net P&L: (105 - 100) * 10 = 50
        // Return %: (50 / 1000) * 100 = 5%
        let return_pct = trade.return_pct().unwrap();
        assert!((return_pct - 5.0).abs() < 0.01);
    }
}
