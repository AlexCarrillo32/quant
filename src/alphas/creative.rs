//! Creative Synthesis Alpha
use super::{AlphaModel, AlphaStats};
use crate::types::*;
use async_trait::async_trait;

#[derive(Debug)]
pub struct CreativeSynthesisAlpha {
    name: String,
    stats: AlphaStats,
}

impl Default for CreativeSynthesisAlpha {
    fn default() -> Self {
        Self::new()
    }
}

impl CreativeSynthesisAlpha {
    pub fn new() -> Self {
        CreativeSynthesisAlpha {
            name: "Creative".to_string(),
            stats: AlphaStats::default(),
        }
    }
}

#[async_trait]
impl AlphaModel for CreativeSynthesisAlpha {
    fn name(&self) -> &str {
        &self.name
    }

    fn human_insight(&self) -> &str {
        "Creative signal synthesis"
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
