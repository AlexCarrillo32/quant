//! Integrated Position Sizing - Kelly Criterion + Risk Management
//!
//! **The Complete System**: Combines mathematical optimization (Kelly) with
//! risk management (stops, caps, drawdown limits) for safe, optimal sizing.
//!
//! ## Architecture
//!
//! ```text
//! Signal → Kelly Criterion → Risk Manager → Final Position Size
//!          (optimal growth)   (safety caps)   (safe & optimal)
//! ```
//!
//! ## Flow
//!
//! 1. **Kelly calculates optimal size** based on:
//!    - Historical win rate
//!    - Win/loss ratio
//!    - Signal confidence
//!
//! 2. **Risk Manager validates** against:
//!    - Max risk per trade (0.5%)
//!    - Daily drawdown limit (5%)
//!    - Consecutive losses (3 max)
//!    - Correlation exposure (50%)
//!    - Available capital
//!
//! 3. **Final position** = min(Kelly size, Risk caps)
//!
//! ## The Human Edge
//!
//! Banks have separate teams for sizing vs risk. We integrate both:
//! - **Kelly**: Maximizes long-term growth
//! - **Risk**: Prevents short-term ruin
//! - **Together**: Optimal AND safe

use crate::core::{
    KellyConfig, KellyCriterion, RiskCheckResult, RiskManager, RiskManagerConfig, RiskViolation,
    SizingMethod,
};
use crate::types::{Price, Signal, Symbol};
use std::collections::HashMap;

/// Integrated position sizer combining Kelly + Risk
pub struct IntegratedPositionSizer {
    kelly: KellyCriterion,
    risk_manager: RiskManager,
    portfolio_value: f64,
}

impl IntegratedPositionSizer {
    /// Create new position sizer
    pub fn new(
        kelly_config: KellyConfig,
        risk_config: RiskManagerConfig,
        initial_portfolio: f64,
    ) -> Self {
        IntegratedPositionSizer {
            kelly: KellyCriterion::with_config(kelly_config),
            risk_manager: RiskManager::new(risk_config, initial_portfolio),
            portfolio_value: initial_portfolio,
        }
    }

    /// Create with default configs
    pub fn with_defaults(initial_portfolio: f64) -> Self {
        Self::new(
            KellyConfig::default(),
            RiskManagerConfig::default(),
            initial_portfolio,
        )
    }

    /// Update portfolio value
    pub fn update_portfolio(&mut self, value: f64) {
        self.portfolio_value = value;
        self.risk_manager.update_portfolio_value(value);
    }

    /// Record a completed trade
    ///
    /// Updates both Kelly stats and Risk Manager
    pub fn record_trade(&mut self, pnl: f64) {
        self.kelly.record_trade(pnl);
        self.risk_manager.record_trade(pnl);
    }

    /// Calculate position size for a signal
    ///
    /// # Returns
    ///
    /// `Ok(PositionSizeDecision)` with approved size, or
    /// `Err(RiskViolation)` if trade is rejected
    pub fn calculate_position(
        &self,
        signal: &Signal,
        current_price: Price,
        open_positions: &HashMap<Symbol, f64>,
        max_positions: usize,
        cash: f64,
    ) -> Result<PositionSizeDecision, RiskViolation> {
        // Step 1: Get Kelly-optimal position size
        let kelly_summary = self.kelly.position_summary(
            self.portfolio_value,
            current_price,
            Some(signal.confidence),
        );

        let kelly_position_value = kelly_summary.position_value;
        let kelly_quantity = kelly_summary.quantity;

        // Step 2: Calculate risk amount (for stop loss)
        let risk_amount = if let Some(stop_loss) = signal.stop_loss {
            // Risk = (Entry - Stop) * Quantity
            let risk_per_share = (current_price.value() - stop_loss.value()).abs();
            risk_per_share * kelly_quantity as f64
        } else {
            // Default: assume 2% stop loss if not specified
            kelly_position_value * 0.02
        };

        // Step 3: Risk Manager validation
        let risk_check = self.risk_manager.check_trade(
            &signal.symbol,
            kelly_position_value,
            risk_amount,
            open_positions,
            max_positions,
            cash,
        );

        match risk_check {
            RiskCheckResult::Approved => {
                // Trade approved - use Kelly size
                Ok(PositionSizeDecision {
                    symbol: signal.symbol.clone(),
                    quantity: kelly_quantity,
                    position_value: kelly_position_value,
                    position_pct: (kelly_position_value / self.portfolio_value) * 100.0,
                    risk_amount,
                    risk_pct: (risk_amount / self.portfolio_value) * 100.0,
                    sizing_method: kelly_summary.sizing_method,
                    kelly_pct: kelly_summary.kelly_pct,
                    approval_status: ApprovalStatus::Approved,
                    rejection_reason: None,
                })
            }
            RiskCheckResult::Rejected(violation) => {
                // Check if we can reduce size to fit within risk limits
                match &violation {
                    RiskViolation::ExcessiveRisk { .. } => {
                        // Try reducing to max allowed risk
                        let max_risk_pct = self.risk_manager.config.max_risk_per_trade_pct;
                        let max_risk_amount = (max_risk_pct / 100.0) * self.portfolio_value;

                        // Recalculate position to fit max risk
                        let adjusted_quantity = if let Some(stop_loss) = signal.stop_loss {
                            let risk_per_share = (current_price.value() - stop_loss.value()).abs();
                            if risk_per_share > 0.0 {
                                (max_risk_amount / risk_per_share).floor() as u64
                            } else {
                                0
                            }
                        } else {
                            // Assume 2% stop, so position = risk / 0.02
                            let adjusted_value = max_risk_amount / 0.02;
                            (adjusted_value / current_price.value()).floor() as u64
                        };

                        if adjusted_quantity > 0 {
                            let adjusted_value = adjusted_quantity as f64 * current_price.value();
                            Ok(PositionSizeDecision {
                                symbol: signal.symbol.clone(),
                                quantity: adjusted_quantity,
                                position_value: adjusted_value,
                                position_pct: (adjusted_value / self.portfolio_value) * 100.0,
                                risk_amount: max_risk_amount,
                                risk_pct: max_risk_pct,
                                sizing_method: SizingMethod::Fixed, // Risk-limited, not pure Kelly
                                kelly_pct: kelly_summary.kelly_pct,
                                approval_status: ApprovalStatus::Reduced,
                                rejection_reason: Some(violation.to_string()),
                            })
                        } else {
                            Err(violation)
                        }
                    }
                    _ => {
                        // Other violations can't be fixed by reducing size
                        Err(violation)
                    }
                }
            }
        }
    }

    /// Get Kelly Criterion stats
    pub fn kelly_stats(&self) -> &crate::core::TradingStats {
        self.kelly.stats()
    }

    /// Get Risk Manager stats
    pub fn risk_stats(&self) -> crate::core::RiskStats {
        self.risk_manager.stats()
    }

    /// Reset daily counters (call at start of trading day)
    pub fn reset_daily(&mut self) {
        self.risk_manager.reset_daily();
    }

    /// Get reference to risk manager config (for inspection)
    pub fn risk_config(&self) -> &RiskManagerConfig {
        &self.risk_manager.config
    }
}

/// Position sizing decision
#[derive(Debug, Clone)]
pub struct PositionSizeDecision {
    /// Symbol to trade
    pub symbol: Symbol,

    /// Quantity to trade (shares)
    pub quantity: u64,

    /// Position value in dollars
    pub position_value: f64,

    /// Position as % of portfolio
    pub position_pct: f64,

    /// Risk amount (distance to stop loss)
    pub risk_amount: f64,

    /// Risk as % of portfolio
    pub risk_pct: f64,

    /// Sizing method used
    pub sizing_method: SizingMethod,

    /// Kelly percentage (if available)
    pub kelly_pct: Option<f64>,

    /// Whether position was approved, reduced, or rejected
    pub approval_status: ApprovalStatus,

    /// Reason if position was rejected or reduced
    pub rejection_reason: Option<String>,
}

/// Position approval status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalStatus {
    /// Position approved at full Kelly size
    Approved,

    /// Position reduced to fit risk limits
    Reduced,

    /// Position fully rejected (not used in Ok() result)
    Rejected,
}

impl std::fmt::Display for PositionSizeDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} shares @ ${:.2} ({:.2}% of portfolio, {:.2}% risk)",
            self.symbol, self.quantity, self.position_value, self.position_pct, self.risk_pct
        )?;

        match self.approval_status {
            ApprovalStatus::Approved => {
                if let Some(kelly) = self.kelly_pct {
                    write!(f, " [Kelly: {:.2}%]", kelly)?;
                } else {
                    write!(f, " [Fixed]")?;
                }
            }
            ApprovalStatus::Reduced => {
                write!(f, " [REDUCED]")?;
                if let Some(reason) = &self.rejection_reason {
                    write!(f, " - {}", reason)?;
                }
            }
            ApprovalStatus::Rejected => {
                write!(f, " [REJECTED]")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Confidence, SignalAction};

    fn create_test_signal(
        symbol: &str,
        price: f64,
        confidence: f64,
        stop_loss: Option<f64>,
    ) -> Signal {
        let mut signal = Signal::new(
            Symbol::new(symbol).unwrap(),
            SignalAction::Buy,
            Confidence::new(confidence).unwrap(),
            "Test signal",
            "TestAlpha",
        );

        if let Some(stop) = stop_loss {
            signal.stop_loss = Some(Price::new(stop).unwrap());
        }

        signal
    }

    #[test]
    fn test_integrated_sizer_basic() {
        let sizer = IntegratedPositionSizer::with_defaults(10_000.0);

        let signal = create_test_signal("AAPL", 150.0, 0.8, Some(145.0));
        let price = Price::new(150.0).unwrap();
        let open_positions = HashMap::new();

        let decision = sizer.calculate_position(&signal, price, &open_positions, 10, 10_000.0);

        assert!(decision.is_ok());
        let decision = decision.unwrap();

        assert_eq!(decision.approval_status, ApprovalStatus::Approved);
        assert!(decision.quantity > 0);
        assert!(decision.risk_pct < 1.0); // Should be well under 1%
    }

    #[test]
    fn test_position_reduction_for_excessive_risk() {
        let mut config = RiskManagerConfig::default();
        config.max_risk_per_trade_pct = 0.5; // Very tight 0.5% max risk

        let sizer = IntegratedPositionSizer::new(
            KellyConfig {
                default_position_pct: 10.0, // Want to take 10% position
                ..Default::default()
            },
            config,
            10_000.0,
        );

        // Signal with wide stop (high risk per share)
        let signal = create_test_signal("AAPL", 150.0, 0.9, Some(100.0)); // $50 stop!
        let price = Price::new(150.0).unwrap();
        let open_positions = HashMap::new();

        let decision = sizer.calculate_position(&signal, price, &open_positions, 10, 10_000.0);

        assert!(decision.is_ok());
        let decision = decision.unwrap();

        // Should be reduced to fit 0.5% risk
        assert_eq!(decision.approval_status, ApprovalStatus::Reduced);
        assert!(decision.risk_pct <= 0.51); // Allow small rounding
    }

    #[test]
    fn test_rejection_for_drawdown() {
        let config = RiskManagerConfig {
            max_daily_drawdown_pct: 5.0,
            ..Default::default()
        };

        let mut sizer = IntegratedPositionSizer::new(KellyConfig::default(), config, 10_000.0);

        // Simulate portfolio dropping 6% (exceeds 5% limit)
        sizer.update_portfolio(9_400.0);

        let signal = create_test_signal("AAPL", 150.0, 0.8, Some(145.0));
        let price = Price::new(150.0).unwrap();
        let open_positions = HashMap::new();

        let decision = sizer.calculate_position(&signal, price, &open_positions, 10, 9_400.0);

        // Should be rejected due to drawdown
        assert!(decision.is_err());
    }

    #[test]
    fn test_kelly_integration() {
        let mut sizer = IntegratedPositionSizer::with_defaults(10_000.0);

        // Build trading history (alternate wins/losses to avoid consecutive loss streak)
        for _ in 0..15 {
            sizer.record_trade(100.0); // Win
            sizer.record_trade(-80.0); // Loss
        }
        // End with wins to reset consecutive loss counter
        for _ in 0..5 {
            sizer.record_trade(100.0); // Win
        }

        // Total: 20 wins, 15 losses (good win rate)

        let signal = create_test_signal("AAPL", 100.0, 0.8, Some(98.0));
        let price = Price::new(100.0).unwrap();
        let open_positions = HashMap::new();

        let decision = sizer.calculate_position(&signal, price, &open_positions, 10, 10_000.0);

        assert!(decision.is_ok(), "Decision should be ok: {:?}", decision);
        let decision = decision.unwrap();

        // Should be using Kelly now (enough trades)
        assert!(decision.kelly_pct.is_some());
        assert_eq!(decision.sizing_method, SizingMethod::Kelly);
    }

    #[test]
    fn test_consecutive_losses_block() {
        let config = RiskManagerConfig {
            max_consecutive_losses: 3,
            ..Default::default()
        };

        let mut sizer = IntegratedPositionSizer::new(KellyConfig::default(), config, 10_000.0);

        // Record 3 consecutive losses
        sizer.record_trade(-50.0);
        sizer.record_trade(-30.0);
        sizer.record_trade(-20.0);

        let signal = create_test_signal("AAPL", 150.0, 0.8, Some(145.0));
        let price = Price::new(150.0).unwrap();
        let open_positions = HashMap::new();

        let decision = sizer.calculate_position(&signal, price, &open_positions, 10, 10_000.0);

        // Should be rejected
        assert!(decision.is_err());
        assert!(matches!(
            decision.unwrap_err(),
            RiskViolation::ConsecutiveLosses { .. }
        ));
    }

    #[test]
    fn test_position_summary_display() {
        let decision = PositionSizeDecision {
            symbol: Symbol::new("AAPL").unwrap(),
            quantity: 100,
            position_value: 15_000.0,
            position_pct: 15.0,
            risk_amount: 500.0,
            risk_pct: 0.5,
            sizing_method: SizingMethod::Kelly,
            kelly_pct: Some(20.0),
            approval_status: ApprovalStatus::Approved,
            rejection_reason: None,
        };

        let display = format!("{}", decision);
        assert!(display.contains("AAPL"));
        assert!(display.contains("100 shares"));
        assert!(display.contains("Kelly"));
    }
}
