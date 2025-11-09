//! Narrative Shift Alpha - Detect Story Changes
//!
//! **Human Insight:**
//! Markets move on narratives, not just fundamentals.
//! When the dominant story changes, prices adjust with a lag.

use super::{AlphaModel, AlphaStats};
use crate::types::*;
use async_trait::async_trait;

#[derive(Debug)]
pub struct NarrativeShiftAlpha {
    name: String,
    stats: AlphaStats,
}

impl Default for NarrativeShiftAlpha {
    fn default() -> Self {
        Self::new()
    }
}

impl NarrativeShiftAlpha {
    pub fn new() -> Self {
        NarrativeShiftAlpha {
            name: "NarrativeShift".to_string(),
            stats: AlphaStats::default(),
        }
    }
}

#[async_trait]
impl AlphaModel for NarrativeShiftAlpha {
    fn name(&self) -> &str {
        &self.name
    }

    fn human_insight(&self) -> &str {
        "Markets move on narratives. When the story changes \
         (e.g., 'inflation transitory' → 'inflation persistent'), \
         prices adjust with a lag. Detect narrative shifts early."
    }

    fn update(&mut self, _data: &MarketSnapshot) {}

    async fn generate_signals(&self) -> Vec<Signal> {
        Vec::new()
    }

    fn reset(&mut self) {
        self.stats = AlphaStats::default();
    }

    fn stats(&self) -> AlphaStats {
        self.stats.clone()
    }
}
