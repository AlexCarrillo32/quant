//! The Human Edge Engine - Library Exports
//!
//! Ultra-low latency trading engine with human intuition layer.

pub mod alphas;
pub mod backtest;
pub mod core;
pub mod data;
pub mod human_layer;
pub mod indicators;
pub mod network;
pub mod types;

// Re-export commonly used types
pub use alphas::{AlphaModel, AlphaSignal};
pub use backtest::{Backtester, BacktestConfig, BacktestReport, PerformanceMetrics};
pub use core::TradingEngine;
pub use types::{Confidence, MarketData, Price, Quantity, Signal, SignalAction};

/// Engine version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
