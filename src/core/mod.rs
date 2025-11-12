//! Core trading engine components

pub mod engine;
pub mod kelly_criterion;
pub mod order_manager;
pub mod performance;
pub mod position_sizer;
pub mod risk_manager;
pub mod signal_aggregator;

pub use engine::{EngineConfig, EngineStats, TradingEngine, TradingEngineBuilder};
pub use kelly_criterion::{
    KellyConfig, KellyCriterion, PositionSummary, SizingMethod, TradingStats,
};
pub use order_manager::{
    CloseReason, Order, OrderManager, OrderManagerConfig, OrderManagerStats, OrderSide,
    OrderStatus, OrderType, Position, PositionSizer, Trade,
};
pub use performance::{
    CpuPinning, OrderArena, PerformanceConfig, PerformanceStats, PrecisionTimer,
};
pub use position_sizer::{ApprovalStatus, IntegratedPositionSizer, PositionSizeDecision};
pub use risk_manager::{RiskCheckResult, RiskManager, RiskManagerConfig, RiskStats, RiskViolation};
pub use signal_aggregator::{AggregationStrategy, SignalAggregator};
