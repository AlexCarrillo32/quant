//! Alpha models - The Human Edge
//!
//! These alpha models incorporate human psychology and behavioral finance
//! insights that banks' purely mathematical models miss.

use crate::types::{MarketSnapshot, Signal};
use async_trait::async_trait;
use std::fmt;

mod creative;
mod crowd_behavior;
mod narrative_shift;
mod panic_detector;
mod structural;

pub use creative::CreativeSynthesisAlpha;
pub use crowd_behavior::CrowdBehaviorAlpha;
pub use narrative_shift::NarrativeShiftAlpha;
pub use panic_detector::PanicDetectorAlpha;
pub use structural::StructuralInefficiencyAlpha;

/// Base trait for all alpha models
///
/// Alpha models analyze market data and generate trading signals.
/// The key insight: model HUMAN behavior, not just price movements.
#[async_trait]
pub trait AlphaModel: Send + Sync + fmt::Debug {
    /// Name of this alpha model
    fn name(&self) -> &str;

    /// Human insight this model is based on
    ///
    /// This documents WHY this model works from a behavioral perspective.
    /// Banks' models don't have this - it's your edge.
    fn human_insight(&self) -> &str;

    /// Update internal state with new market data
    ///
    /// This should be fast (<1μs for hot path models)
    fn update(&mut self, data: &MarketSnapshot);

    /// Generate trading signals
    ///
    /// Returns signals if opportunities are detected, None otherwise.
    async fn generate_signals(&self) -> Vec<Signal>;

    /// Reset internal state
    fn reset(&mut self);

    /// Get performance statistics
    fn stats(&self) -> AlphaStats {
        AlphaStats::default()
    }
}

/// Alpha model performance statistics
#[derive(Debug, Clone, Default)]
pub struct AlphaStats {
    pub signals_generated: usize,
    pub signals_actionable: usize,
    pub avg_confidence: f64,
    pub last_signal_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

/// Convenience type for boxed alpha models
pub type BoxedAlpha = Box<dyn AlphaModel>;

/// Alpha signal with source attribution
#[derive(Debug, Clone)]
pub struct AlphaSignal {
    pub signal: Signal,
    pub alpha_name: String,
}
