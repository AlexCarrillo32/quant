//! Live Panic Detector Demo
//!
//! Fetches REAL market data and runs the Panic Detector alpha model
//! to find buying opportunities during market panic.

use quant_engine::{
    alphas::{AlphaModel, PanicDetectorAlpha},
    data::{DataProvider, YahooFinanceProvider},
    types::{SignalAction, Symbol},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    print_banner();

    println!("🔍 Scanning market for panic signals...\n");

    // Create data provider
    let data_provider = YahooFinanceProvider::new();

    // Symbols to monitor (major ETFs and indices)
    let symbols = vec![
        Symbol::new("SPY")?, // S&P 500 ETF
        Symbol::new("QQQ")?, // Nasdaq 100 ETF
        Symbol::new("IWM")?, // Russell 2000 ETF
        Symbol::new("DIA")?, // Dow Jones ETF
    ];

    println!(
        "📊 Fetching real-time data for: {}",
        symbols
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!();

    // Fetch market data
    let snapshot = data_provider.get_quotes(&symbols).await?;

    println!(
        "✅ Data received! Analyzing {} symbols...\n",
        snapshot.len()
    );
    println!("{}", "═".repeat(80));

    // Initialize Panic Detector
    let mut panic_detector = PanicDetectorAlpha::new();

    // Analyze each symbol
    for symbol in symbols {
        if let Some(market_data) = snapshot.get(&symbol) {
            // Display market data
            println!("\n📈 {}", symbol);
            println!("   Price:     ${:.2}", market_data.last_price.value());
            println!("   Volume:    {:>12}", format_number(market_data.volume));

            if let Some(change) = market_data.intraday_change_pct() {
                let arrow = if change >= 0.0 { "🟢" } else { "🔴" };
                println!("   Change:    {} {:+.2}%", arrow, change);
            }

            if let Some(vix) = market_data.vix {
                let vix_emoji = if vix > 30.0 {
                    "⚠️ "
                } else if vix > 20.0 {
                    "⚡"
                } else {
                    "✨"
                };
                println!("   VIX:       {} {:.2} (volatility index)", vix_emoji, vix);
            }

            if let Some(pc_ratio) = market_data.put_call_ratio {
                let pc_emoji = if pc_ratio > 1.5 { "⚠️ " } else { "📊" };
                println!("   P/C Ratio: {} {:.2} (put/call)", pc_emoji, pc_ratio);
            }

            // Run Panic Detector
            if let Some(signal) = panic_detector.analyze_symbol(&market_data) {
                println!("\n   🚨 SIGNAL DETECTED! 🚨");
                println!("   ────────────────────────────────────────");
                println!("   Action:     {}", format_action(&signal.action));
                println!(
                    "   Confidence: {} ({:.1}%)",
                    confidence_emoji(signal.confidence.value()),
                    signal.confidence.as_percent()
                );
                if let Some(stop_loss) = signal.stop_loss {
                    let sl_pct = ((market_data.last_price.value() - stop_loss.value())
                        / market_data.last_price.value())
                        * 100.0;
                    println!("   Stop Loss:  ${:.2} ({:.1}%)", stop_loss.value(), sl_pct);
                }

                if let Some(take_profit) = signal.take_profit {
                    let tp_pct = ((take_profit.value() - market_data.last_price.value())
                        / market_data.last_price.value())
                        * 100.0;
                    println!(
                        "   Take Profit: ${:.2} ({:.1}%)",
                        take_profit.value(),
                        tp_pct
                    );
                }

                println!("\n   💡 Reason: {}", signal.reason);
                println!("   ────────────────────────────────────────");

                // Show metadata
                if let Some(vix) = signal.metadata.get("vix") {
                    println!("\n   📊 Signal Details:");
                    println!("      VIX Level: {:.2}", vix.as_f64().unwrap_or(0.0));
                    if let Some(pc) = signal.metadata.get("put_call_ratio") {
                        println!("      P/C Ratio: {:.2}", pc.as_f64().unwrap_or(0.0));
                    }
                }
            } else {
                println!("   ✓ No panic detected - market conditions normal");
            }
        }
    }

    println!("\n{}", "═".repeat(80));
    println!("\n📊 Analysis Complete!");

    // Show statistics
    let stats = panic_detector.stats();
    if stats.signals_generated > 0 {
        println!("\n✅ Generated {} signal(s)", stats.signals_generated);
        println!(
            "   {} actionable (high confidence)",
            stats.signals_actionable
        );
        println!("\n💡 Tip: High confidence signals (>75%) indicate strong panic conditions");
        println!("   These may represent good buying opportunities as fear subsides.");
    } else {
        println!("\n✅ No panic signals detected");
        println!("   Market conditions appear stable.");
    }

    println!("\n{}", "─".repeat(80));
    println!("Note: This is for educational purposes. Always do your own research.");
    println!("{}", "─".repeat(80));

    Ok(())
}

fn print_banner() {
    println!(
        r#"
╔════════════════════════════════════════════════════════════════╗
║                                                                ║
║           🧠 THE HUMAN EDGE ENGINE - LIVE DEMO 🚀              ║
║                                                                ║
║                    Panic Detector Alpha                        ║
║          Exploiting Human Fear in Financial Markets           ║
║                                                                ║
╚════════════════════════════════════════════════════════════════╝
"#
    );
}

fn format_action(action: &SignalAction) -> String {
    match action {
        SignalAction::Buy => "🟢 BUY".to_string(),
        SignalAction::Sell => "🔴 SELL".to_string(),
        SignalAction::Close => "⚪ CLOSE".to_string(),
        SignalAction::Hold => "⏸️  HOLD".to_string(),
    }
}

fn confidence_emoji(confidence: f64) -> &'static str {
    if confidence > 0.9 {
        "🔥🔥🔥"
    } else if confidence > 0.75 {
        "🔥🔥"
    } else if confidence > 0.5 {
        "🔥"
    } else {
        "💤"
    }
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let mut count = 0;

    for c in s.chars().rev() {
        if count == 3 {
            result.push(',');
            count = 0;
        }
        result.push(c);
        count += 1;
    }

    result.chars().rev().collect()
}
