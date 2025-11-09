//! Full Trading Engine Demo
//!
//! Demonstrates the complete engine running with:
//! - Yahoo Finance real data
//! - Panic Detector alpha model
//! - Signal aggregation
//! - Continuous monitoring loop

use quant_engine::{
    alphas::{AlphaModel, PanicDetectorAlpha},
    core::{AggregationStrategy, TradingEngine},
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

    print_banner();

    println!("🔧 Initializing The Human Edge Engine...\n");

    // Create data provider
    let data_provider = Arc::new(YahooFinanceProvider::new());

    // Define symbols to monitor
    let symbols = vec![
        Symbol::new("SPY")?, // S&P 500 ETF
        Symbol::new("QQQ")?, // Nasdaq 100 ETF
        Symbol::new("IWM")?, // Russell 2000 ETF
        Symbol::new("DIA")?, // Dow Jones ETF
    ];

    println!(
        "📊 Monitoring symbols: {}",
        symbols
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Build the engine
    let mut engine = TradingEngine::builder()
        .with_data_provider(data_provider)
        .with_symbols(symbols)
        .with_update_interval(Duration::from_secs(60)) // Check every 60 seconds
        .with_min_confidence(0.7) // 70% minimum confidence
        .with_aggregation_strategy(AggregationStrategy::WeightedAverage)
        .with_paper_trading(true) // Safe mode
        .build()?;

    // Add alpha models
    println!("\n🧠 Loading Alpha Models:");

    let panic_detector = Box::new(PanicDetectorAlpha::new());
    println!("   ✓ {}", panic_detector.name());
    println!("     └─ {}", panic_detector.human_insight());

    engine.add_alpha(panic_detector);

    // TODO: Add more alpha models
    println!("\n💡 More alpha models coming soon:");
    println!("   • Narrative Shift Detector");
    println!("   • Crowd Behavior Analyzer");
    println!("   • Structural Inefficiency Hunter");
    println!("   • Creative Synthesis Engine");

    println!("\n{}", "═".repeat(80));
    println!("🚀 Starting engine (Ctrl+C to stop)...");
    println!("{}", "═".repeat(80));
    println!();

    // Run the engine (infinite loop)
    engine.run().await?;

    Ok(())
}

fn print_banner() {
    println!(
        r#"
╔════════════════════════════════════════════════════════════════╗
║                                                                ║
║        🧠 THE HUMAN EDGE ENGINE - FULL SYSTEM DEMO 🚀         ║
║                                                                ║
║          Real-Time Trading Engine with Human Intuition        ║
║                                                                ║
╚════════════════════════════════════════════════════════════════╝
"#
    );
}
