//! Structural Inefficiency Alpha
use super::{AlphaModel, AlphaStats};
use crate::types::*;
use async_trait::async_trait;

#[derive(Debug)]
pub struct StructuralInefficiencyAlpha {
    name: String,
    stats: AlphaStats,
}

impl Default for StructuralInefficiencyAlpha {
    fn default() -> Self {
        Self::new()
    }
}

impl StructuralInefficiencyAlpha {
    pub fn new() -> Self {
        StructuralInefficiencyAlpha {
            name: "Structural".to_string(),
            stats: AlphaStats::default(),
        }
    }
}

#[async_trait]
impl AlphaModel for StructuralInefficiencyAlpha {
    fn name(&self) -> &str {
        &self.name
    }

    fn human_insight(&self) -> &str {
        "Exploit structural market inefficiencies"
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
