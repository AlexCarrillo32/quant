//! Crowd Behavior Alpha
use super::{AlphaModel, AlphaStats};
use crate::types::*;
use async_trait::async_trait;

#[derive(Debug)]
pub struct CrowdBehaviorAlpha {
    name: String,
    stats: AlphaStats,
}

impl Default for CrowdBehaviorAlpha {
    fn default() -> Self {
        Self::new()
    }
}

impl CrowdBehaviorAlpha {
    pub fn new() -> Self {
        CrowdBehaviorAlpha {
            name: "CrowdBehavior".to_string(),
            stats: AlphaStats::default(),
        }
    }
}

#[async_trait]
impl AlphaModel for CrowdBehaviorAlpha {
    fn name(&self) -> &str {
        &self.name
    }

    fn human_insight(&self) -> &str {
        "Exploit retail trader irrationality"
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
