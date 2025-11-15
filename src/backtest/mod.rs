//! Backtesting Engine - Test Strategies on Historical Data
//!
//! **Purpose**: Validate strategies before risking real capital
//!
//! Features:
//! - Historical data simulation
//! - Realistic trade execution (slippage, commissions)
//! - Performance metrics (Sharpe, Sortino, Max DD)
//! - Walk-forward analysis
//! - Monte Carlo simulation

pub mod engine;
pub mod metrics;
pub mod report;
pub mod trade;

pub use engine::{BacktestConfig, BacktestResult, Backtester};
pub use metrics::{PerformanceMetrics, TradeStatistics};
pub use report::BacktestReport;
pub use trade::{BacktestTrade, TradeOutcome};
