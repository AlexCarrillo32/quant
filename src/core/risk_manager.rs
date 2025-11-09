//! Risk Manager - Prevent Over-Exposure and Protect Capital
//!
//! **Critical Component**: This is what keeps you in the game
//!
//! Strategy: Small profits only work if you survive to trade another day
//! - Max risk per trade: 0.5%
//! - Max portfolio drawdown: 5% daily
//! - Max correlation exposure: 50%
//! - Stop trading on losing streaks

use crate::types::*;
use std::collections::HashMap;

/// Risk management configuration
#[derive(Debug, Clone)]
pub struct RiskManagerConfig {
    /// Maximum risk per trade (% of portfolio)
    pub max_risk_per_trade_pct: f64,

    /// Maximum daily drawdown before stopping (% of portfolio)
    pub max_daily_drawdown_pct: f64,

    /// Maximum correlation exposure (% of portfolio in correlated assets)
    pub max_correlation_exposure_pct: f64,

    /// Stop trading after N consecutive losses
    pub max_consecutive_losses: usize,

    /// Minimum portfolio value before emergency stop
    pub emergency_stop_value: f64,
}

impl Default for RiskManagerConfig {
    fn default() -> Self {
        RiskManagerConfig {
            max_risk_per_trade_pct: 0.5,        // 0.5% max risk per trade
            max_daily_drawdown_pct: 5.0,        // Stop at -5% daily loss
            max_correlation_exposure_pct: 50.0, // Max 50% in correlated assets
            max_consecutive_losses: 3,          // Stop after 3 losses in a row
            emergency_stop_value: 5_000.0,      // Emergency stop at 50% loss
        }
    }
}

/// Risk check result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskCheckResult {
    /// Trade is approved
    Approved,

    /// Trade rejected with reason
    Rejected(RiskViolation),
}

/// Types of risk violations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskViolation {
    /// Exceeds max risk per trade
    ExcessiveRisk {
        risk_pct: String,
        max_allowed: String,
    },

    /// Daily drawdown limit reached
    MaxDrawdownReached {
        current_dd: String,
        max_allowed: String,
    },

    /// Too many consecutive losses
    ConsecutiveLosses { count: usize, max_allowed: usize },

    /// Too much correlation exposure
    CorrelationExposure {
        exposure_pct: String,
        max_allowed: String,
    },

    /// Portfolio value too low (emergency stop)
    EmergencyStop { value: String, threshold: String },

    /// Too many open positions
    MaxPositionsReached { current: usize, max_allowed: usize },

    /// Insufficient capital
    InsufficientCapital { required: String, available: String },
}

impl std::fmt::Display for RiskViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskViolation::ExcessiveRisk {
                risk_pct,
                max_allowed,
            } => {
                write!(f, "Risk {}% exceeds max {}%", risk_pct, max_allowed)
            }
            RiskViolation::MaxDrawdownReached {
                current_dd,
                max_allowed,
            } => {
                write!(f, "Drawdown {}% exceeds max {}%", current_dd, max_allowed)
            }
            RiskViolation::ConsecutiveLosses { count, max_allowed } => {
                write!(f, "{} consecutive losses (max {})", count, max_allowed)
            }
            RiskViolation::CorrelationExposure {
                exposure_pct,
                max_allowed,
            } => {
                write!(
                    f,
                    "Correlation exposure {}% exceeds max {}%",
                    exposure_pct, max_allowed
                )
            }
            RiskViolation::EmergencyStop { value, threshold } => {
                write!(
                    f,
                    "Emergency stop: portfolio ${} below threshold ${}",
                    value, threshold
                )
            }
            RiskViolation::MaxPositionsReached {
                current,
                max_allowed,
            } => {
                write!(f, "Max positions reached: {}/{}", current, max_allowed)
            }
            RiskViolation::InsufficientCapital {
                required,
                available,
            } => {
                write!(
                    f,
                    "Insufficient capital: need ${}, have ${}",
                    required, available
                )
            }
        }
    }
}

/// Risk manager
///
/// Responsibilities:
/// - Pre-trade risk checks
/// - Drawdown monitoring
/// - Correlation exposure tracking
/// - Loss streak detection
pub struct RiskManager {
    config: RiskManagerConfig,

    /// Starting portfolio value for the day
    day_start_value: f64,

    /// Current portfolio value
    current_value: f64,

    /// Consecutive losses counter
    consecutive_losses: usize,

    /// Symbol correlation groups (for correlation exposure)
    correlation_groups: HashMap<String, Vec<Symbol>>,
}

impl RiskManager {
    /// Create new risk manager
    pub fn new(config: RiskManagerConfig, initial_value: f64) -> Self {
        // Define correlation groups (e.g., tech stocks move together)
        let mut correlation_groups = HashMap::new();
        correlation_groups.insert(
            "tech_etf".to_string(),
            vec![
                Symbol::new("QQQ").unwrap(), // Nasdaq
                Symbol::new("XLK").unwrap(), // Tech sector
            ],
        );
        correlation_groups.insert(
            "broad_market".to_string(),
            vec![
                Symbol::new("SPY").unwrap(), // S&P 500
                Symbol::new("IWM").unwrap(), // Russell 2000
                Symbol::new("DIA").unwrap(), // Dow Jones
            ],
        );

        RiskManager {
            config,
            day_start_value: initial_value,
            current_value: initial_value,
            consecutive_losses: 0,
            correlation_groups,
        }
    }

    /// Update portfolio value
    pub fn update_portfolio_value(&mut self, value: f64) {
        self.current_value = value;
    }

    /// Reset daily counters (call at start of each trading day)
    pub fn reset_daily(&mut self) {
        self.day_start_value = self.current_value;
        self.consecutive_losses = 0;
        tracing::info!(
            "🔄 Risk manager reset for new day. Starting value: ${:.2}",
            self.day_start_value
        );
    }

    /// Record a trade result
    pub fn record_trade(&mut self, pnl: f64) {
        if pnl < 0.0 {
            self.consecutive_losses += 1;
            tracing::warn!(
                "📉 Loss recorded. Consecutive losses: {}",
                self.consecutive_losses
            );
        } else {
            self.consecutive_losses = 0;
        }
    }

    /// Check if new trade is allowed
    ///
    /// Returns Approved or Rejected with specific violation
    pub fn check_trade(
        &self,
        symbol: &Symbol,
        position_value: f64,
        risk_amount: f64,
        open_positions: &HashMap<Symbol, f64>,
        max_positions: usize,
        cash: f64,
    ) -> RiskCheckResult {
        // Check 1: Emergency stop
        if self.current_value <= self.config.emergency_stop_value {
            return RiskCheckResult::Rejected(RiskViolation::EmergencyStop {
                value: format!("{:.2}", self.current_value),
                threshold: format!("{:.2}", self.config.emergency_stop_value),
            });
        }

        // Check 2: Daily drawdown
        let daily_dd_pct =
            ((self.day_start_value - self.current_value) / self.day_start_value) * 100.0;
        if daily_dd_pct >= self.config.max_daily_drawdown_pct {
            return RiskCheckResult::Rejected(RiskViolation::MaxDrawdownReached {
                current_dd: format!("{:.2}", daily_dd_pct),
                max_allowed: format!("{:.2}", self.config.max_daily_drawdown_pct),
            });
        }

        // Check 3: Consecutive losses
        if self.consecutive_losses >= self.config.max_consecutive_losses {
            return RiskCheckResult::Rejected(RiskViolation::ConsecutiveLosses {
                count: self.consecutive_losses,
                max_allowed: self.config.max_consecutive_losses,
            });
        }

        // Check 4: Risk per trade
        let risk_pct = (risk_amount / self.current_value) * 100.0;
        if risk_pct > self.config.max_risk_per_trade_pct {
            return RiskCheckResult::Rejected(RiskViolation::ExcessiveRisk {
                risk_pct: format!("{:.2}", risk_pct),
                max_allowed: format!("{:.2}", self.config.max_risk_per_trade_pct),
            });
        }

        // Check 5: Max positions
        if open_positions.len() >= max_positions {
            return RiskCheckResult::Rejected(RiskViolation::MaxPositionsReached {
                current: open_positions.len(),
                max_allowed: max_positions,
            });
        }

        // Check 6: Sufficient capital
        if cash < position_value {
            return RiskCheckResult::Rejected(RiskViolation::InsufficientCapital {
                required: format!("{:.2}", position_value),
                available: format!("{:.2}", cash),
            });
        }

        // Check 7: Correlation exposure
        if let Some(violation) =
            self.check_correlation_exposure(symbol, position_value, open_positions)
        {
            return RiskCheckResult::Rejected(violation);
        }

        RiskCheckResult::Approved
    }

    /// Check correlation exposure
    fn check_correlation_exposure(
        &self,
        new_symbol: &Symbol,
        new_position_value: f64,
        open_positions: &HashMap<Symbol, f64>,
    ) -> Option<RiskViolation> {
        // Find which correlation group this symbol belongs to
        for (_group_name, symbols) in &self.correlation_groups {
            if symbols.contains(new_symbol) {
                // Calculate total exposure in this correlation group
                let mut group_exposure = new_position_value;

                for symbol in symbols {
                    if let Some(value) = open_positions.get(symbol) {
                        group_exposure += value;
                    }
                }

                let exposure_pct = (group_exposure / self.current_value) * 100.0;

                if exposure_pct > self.config.max_correlation_exposure_pct {
                    return Some(RiskViolation::CorrelationExposure {
                        exposure_pct: format!("{:.2}", exposure_pct),
                        max_allowed: format!("{:.2}", self.config.max_correlation_exposure_pct),
                    });
                }
            }
        }

        None
    }

    /// Get current drawdown percentage
    pub fn current_drawdown_pct(&self) -> f64 {
        ((self.day_start_value - self.current_value) / self.day_start_value) * 100.0
    }

    /// Get consecutive losses count
    pub fn consecutive_losses(&self) -> usize {
        self.consecutive_losses
    }

    /// Get risk statistics
    pub fn stats(&self) -> RiskStats {
        RiskStats {
            current_value: self.current_value,
            day_start_value: self.day_start_value,
            daily_drawdown_pct: self.current_drawdown_pct(),
            consecutive_losses: self.consecutive_losses,
            max_daily_drawdown_pct: self.config.max_daily_drawdown_pct,
            max_consecutive_losses: self.config.max_consecutive_losses,
        }
    }
}

/// Risk statistics
#[derive(Debug, Clone)]
pub struct RiskStats {
    pub current_value: f64,
    pub day_start_value: f64,
    pub daily_drawdown_pct: f64,
    pub consecutive_losses: usize,
    pub max_daily_drawdown_pct: f64,
    pub max_consecutive_losses: usize,
}

impl RiskStats {
    /// Check if risk limits are healthy
    pub fn is_healthy(&self) -> bool {
        self.daily_drawdown_pct < self.max_daily_drawdown_pct
            && self.consecutive_losses < self.max_consecutive_losses
    }

    /// Format risk status for logging
    pub fn status_message(&self) -> String {
        format!(
            "Drawdown: {:.2}% ({:.2}% max) | Losses: {}/{} | Value: ${:.2}",
            self.daily_drawdown_pct,
            self.max_daily_drawdown_pct,
            self.consecutive_losses,
            self.max_consecutive_losses,
            self.current_value
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_manager_creation() {
        let config = RiskManagerConfig::default();
        let manager = RiskManager::new(config, 10_000.0);

        assert_eq!(manager.current_value, 10_000.0);
        assert_eq!(manager.consecutive_losses, 0);
    }

    #[test]
    fn test_excessive_risk_rejection() {
        let config = RiskManagerConfig {
            max_risk_per_trade_pct: 0.5, // 0.5% max
            ..Default::default()
        };
        let manager = RiskManager::new(config, 10_000.0);

        let symbol = Symbol::new("SPY").unwrap();
        let position_value = 1_000.0;
        let risk_amount = 100.0; // 1% risk (exceeds 0.5% max)
        let open_positions = HashMap::new();

        let result = manager.check_trade(
            &symbol,
            position_value,
            risk_amount,
            &open_positions,
            10,
            10_000.0,
        );

        assert!(matches!(
            result,
            RiskCheckResult::Rejected(RiskViolation::ExcessiveRisk { .. })
        ));
    }

    #[test]
    fn test_consecutive_losses_rejection() {
        let config = RiskManagerConfig {
            max_consecutive_losses: 3,
            ..Default::default()
        };
        let mut manager = RiskManager::new(config, 10_000.0);

        // Record 3 losses
        manager.record_trade(-50.0);
        manager.record_trade(-30.0);
        manager.record_trade(-20.0);

        assert_eq!(manager.consecutive_losses, 3);

        // Next trade should be rejected
        let symbol = Symbol::new("SPY").unwrap();
        let result = manager.check_trade(&symbol, 1_000.0, 10.0, &HashMap::new(), 10, 10_000.0);

        assert!(matches!(
            result,
            RiskCheckResult::Rejected(RiskViolation::ConsecutiveLosses { .. })
        ));
    }

    #[test]
    fn test_win_resets_loss_streak() {
        let config = RiskManagerConfig::default();
        let mut manager = RiskManager::new(config, 10_000.0);

        // Record losses
        manager.record_trade(-50.0);
        manager.record_trade(-30.0);
        assert_eq!(manager.consecutive_losses, 2);

        // Record win
        manager.record_trade(100.0);
        assert_eq!(manager.consecutive_losses, 0);
    }

    #[test]
    fn test_daily_drawdown_rejection() {
        let config = RiskManagerConfig {
            max_daily_drawdown_pct: 5.0, // 5% max daily loss
            ..Default::default()
        };
        let mut manager = RiskManager::new(config, 10_000.0);

        // Portfolio drops to $9,400 (-6% drawdown)
        manager.update_portfolio_value(9_400.0);

        let symbol = Symbol::new("SPY").unwrap();
        let result = manager.check_trade(&symbol, 1_000.0, 10.0, &HashMap::new(), 10, 9_400.0);

        assert!(matches!(
            result,
            RiskCheckResult::Rejected(RiskViolation::MaxDrawdownReached { .. })
        ));
    }

    #[test]
    fn test_approved_trade() {
        let config = RiskManagerConfig::default();
        let manager = RiskManager::new(config, 10_000.0);

        let symbol = Symbol::new("SPY").unwrap();
        let position_value = 1_000.0;
        let risk_amount = 30.0; // 0.3% risk (within 0.5% max)
        let open_positions = HashMap::new();

        let result = manager.check_trade(
            &symbol,
            position_value,
            risk_amount,
            &open_positions,
            10,
            10_000.0,
        );

        assert_eq!(result, RiskCheckResult::Approved);
    }

    #[test]
    fn test_correlation_exposure() {
        let config = RiskManagerConfig {
            max_correlation_exposure_pct: 50.0, // Max 50% in correlated assets
            ..Default::default()
        };
        let manager = RiskManager::new(config, 10_000.0);

        // Already have $4k in SPY (broad market group)
        let mut open_positions = HashMap::new();
        open_positions.insert(Symbol::new("SPY").unwrap(), 4_000.0);

        // Try to add $2k in IWM (also broad market group)
        // Total would be $6k = 60% (exceeds 50% max)
        let symbol = Symbol::new("IWM").unwrap();
        let result = manager.check_trade(&symbol, 2_000.0, 20.0, &open_positions, 10, 10_000.0);

        assert!(matches!(
            result,
            RiskCheckResult::Rejected(RiskViolation::CorrelationExposure { .. })
        ));
    }
}
