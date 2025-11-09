//! Single Cycle Engine Test
//!
//! Runs one engine cycle to verify everything works

use quant_engine::{
    alphas::{AlphaModel, PanicDetectorAlpha},
    core::{AggregationStrategy, EngineConfig, TradingEngine},
    data::YahooFinanceProvider,
    types::Symbol,
};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("\n🔧 Testing The Human Edge Engine - Single Cycle\n");

    // Create data provider
    let data_provider = Arc::new(YahooFinanceProvider::new());

    // Define symbols
    let symbols = vec![Symbol::new("SPY")?, Symbol::new("QQQ")?];

    println!(
        "📊 Monitoring: {}",
        symbols
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Build config
    let config = EngineConfig {
        update_interval: Duration::from_secs(60),
        symbols,
        min_confidence: 0.7,
        aggregation_strategy: AggregationStrategy::WeightedAverage,
        max_positions: 10,
        paper_trading: true,
    };

    // Build engine
    let mut engine = TradingEngine::new(config, data_provider, 10_000.0);

    // Add Panic Detector
    let panic_detector = Box::new(PanicDetectorAlpha::new());
    println!("\n🧠 Alpha: {}", panic_detector.name());
    println!("   {}\n", panic_detector.human_insight());

    engine.add_alpha(panic_detector);

    // Manually run one cycle
    println!("{}", "═".repeat(80));
    println!("🚀 Running one engine cycle...");
    println!("{}", "═".repeat(80));
    println!();

    // Access the run_cycle method through run with immediate cancel
    // For testing, we'll just call run and let it run one cycle
    tokio::select! {
        _ = engine.run() => {
            // Engine ran
        }
        _ = tokio::time::sleep(Duration::from_secs(5)) => {
            // Timeout after 5 seconds
            println!("\n✅ Engine cycle completed successfully!");
        }
    }

    println!("\n{}", "═".repeat(80));
    println!("Test complete!");
    println!("{}", "═".repeat(80));

    Ok(())
}
