//! Comprehensive Backtesting Demo
//!
//! Demonstrates:
//! - Historical data simulation
//! - Multiple alpha models
//! - Risk management
//! - Performance analysis
//! - Report generation

use chrono::Utc;
use quant_engine::{
    alphas::PanicDetectorAlpha,
    backtest::{BacktestConfig, BacktestReport, Backtester},
    core::risk_manager::RiskManagerConfig,
    core::signal_aggregator::AggregationStrategy,
    types::*,
};
use rand::Rng;
use std::collections::HashMap;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           BACKTESTING ENGINE DEMONSTRATION                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Generate synthetic market data for backtesting
    println!("📊 Generating synthetic market data...");
    let symbols = vec![
        Symbol::new("AAPL").unwrap(),
        Symbol::new("GOOGL").unwrap(),
        Symbol::new("MSFT").unwrap(),
    ];

    let historical_data = generate_synthetic_data(&symbols, 252); // 1 year of daily data
    println!(
        "   ✓ Generated {} days of data for {} symbols\n",
        252,
        symbols.len()
    );

    // Configure backtest
    println!("⚙️  Configuring backtester...");
    let config = BacktestConfig {
        initial_capital: 100_000.0,
        commission_per_trade: 1.0,
        slippage_pct: 0.05, // 5 basis points
        default_position_size_pct: 10.0,
        use_confidence_sizing: true,
        risk_config: RiskManagerConfig {
            max_risk_per_trade_pct: 1.0,
            max_daily_drawdown_pct: 5.0,
            max_correlation_exposure_pct: 50.0,
            max_consecutive_losses: 3,
            emergency_stop_value: 50_000.0,
        },
        aggregation_strategy: AggregationStrategy::WeightedAverage,
    };

    println!("   ✓ Initial Capital: ${:.2}", config.initial_capital);
    println!(
        "   ✓ Commission: ${:.2} per trade",
        config.commission_per_trade
    );
    println!("   ✓ Slippage: {:.3}%", config.slippage_pct);
    println!(
        "   ✓ Position Size: {:.1}% of capital\n",
        config.default_position_size_pct
    );

    // Create backtester and add alpha models
    println!("🧠 Adding alpha models...");
    let mut backtester = Backtester::new(config);

    let panic_alpha = PanicDetectorAlpha::default();
    backtester.add_alpha(Box::new(panic_alpha));
    println!("   ✓ Panic Detector Alpha added\n");

    // Run backtest
    println!("🚀 Running backtest...");
    println!("   This may take a moment...\n");

    match backtester.run(&historical_data, &symbols) {
        Ok(result) => {
            println!("✅ Backtest completed successfully!\n");

            // Display quick summary
            println!("╔══════════════════════════════════════════════════════════════╗");
            println!("║                    QUICK SUMMARY                             ║");
            println!("╚══════════════════════════════════════════════════════════════╝");
            println!("  Initial Capital:    ${:>12.2}", 100_000.0);
            println!("  Final Capital:      ${:>12.2}", result.final_capital);
            println!(
                "  P/L:                ${:>12.2}",
                result.final_capital - 100_000.0
            );
            println!(
                "  Total Return:       {:>12.2}%",
                result.metrics.total_return_pct
            );
            println!(
                "  Sharpe Ratio:       {:>12.2}",
                result.metrics.sharpe_ratio
            );
            println!(
                "  Max Drawdown:       {:>12.2}%",
                result.metrics.max_drawdown_pct
            );
            println!("  Total Trades:       {:>12}", result.trades.len());
            println!(
                "  Win Rate:           {:>12.2}%",
                result.metrics.trade_stats.win_rate_pct
            );
            println!(
                "  Profit Factor:      {:>12.2}",
                result.metrics.trade_stats.profit_factor
            );
            println!("  Strategy Grade:     {:>12}", result.metrics.grade());
            println!();

            // Generate and print full report
            let report = BacktestReport::new(&result);
            report.print();

            // Trading signal analysis
            println!("\n╔══════════════════════════════════════════════════════════════╗");
            println!("║                  SIGNAL ANALYSIS                             ║");
            println!("╚══════════════════════════════════════════════════════════════╝");
            println!("  Total Signals Generated:  {:>8}", result.total_signals);
            println!(
                "  Signals Accepted:         {:>8}",
                result.total_signals - result.rejected_signals
            );
            println!("  Signals Rejected:         {:>8}", result.rejected_signals);

            let acceptance_rate = if result.total_signals > 0 {
                ((result.total_signals - result.rejected_signals) as f64
                    / result.total_signals as f64)
                    * 100.0
            } else {
                0.0
            };
            println!("  Acceptance Rate:          {:>7.2}%", acceptance_rate);
            println!();

            // Strategy recommendation
            if result.metrics.is_good_strategy() {
                println!("✅ RECOMMENDATION: Strategy shows promise!");
                println!("   Next steps:");
                println!("   1. Forward test on recent data");
                println!("   2. Paper trade for 30 days");
                println!("   3. Consider parameter optimization");
                println!("   4. Add more alpha models for diversification");
            } else {
                println!("⚠️  RECOMMENDATION: Strategy needs improvement");
                println!("   Suggestions:");
                if result.metrics.trade_stats.win_rate_pct < 40.0 {
                    println!("   - Improve signal quality (low win rate)");
                }
                if result.metrics.trade_stats.profit_factor < 1.5 {
                    println!("   - Better risk management needed");
                }
                if result.metrics.max_drawdown_pct < -20.0 {
                    println!("   - Reduce position sizes");
                }
                if result.metrics.sharpe_ratio < 1.0 {
                    println!("   - Reduce volatility or improve returns");
                }
            }
            println!();

            // Equity curve visualization (simple ASCII)
            println!("╔══════════════════════════════════════════════════════════════╗");
            println!("║                   EQUITY CURVE                               ║");
            println!("╚══════════════════════════════════════════════════════════════╝");
            print_equity_curve(&result.equity_curve);
        }
        Err(e) => {
            eprintln!("❌ Backtest failed: {}", e);
            std::process::exit(1);
        }
    }

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                    DEMO COMPLETE                             ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}

/// Generate synthetic market data for testing
fn generate_synthetic_data(symbols: &[Symbol], days: usize) -> HashMap<Symbol, Vec<MarketData>> {
    let mut rng = rand::rng();
    let mut data = HashMap::new();

    for symbol in symbols {
        let mut time_series = Vec::new();
        let mut price: f64 = 100.0;

        for day in 0..days {
            // Random walk with drift
            let change_pct = rng.random_range(-2.0..3.0); // Slight upward bias
            price *= 1.0 + (change_pct / 100.0);
            price = price.max(10.0); // Don't go below $10

            // Create market data
            let bid = Price::new(price * 0.999).unwrap();
            let ask = Price::new(price * 1.001).unwrap();
            let bid_size = Quantity::buy(1_000_000);
            let ask_size = Quantity::sell(1_000_000);
            let volume = rng.random_range(1_000_000u64..10_000_000u64);

            // Generate timestamp
            let timestamp = Utc::now() - chrono::Duration::days((days - day - 1) as i64);

            let quote = Quote {
                bid,
                ask,
                bid_size,
                ask_size,
                timestamp,
            };

            let last_price = Price::new(price).unwrap();

            let market_data = MarketData {
                symbol: symbol.clone(),
                quote,
                last_price,
                volume,
                timestamp,
                open: Some(Price::new(price * 0.998).unwrap()),
                high: Some(Price::new(price * 1.005).unwrap()),
                low: Some(Price::new(price * 0.995).unwrap()),
                prev_close: Some(Price::new(price * 0.99).unwrap()),
                vix: Some(20.0 + rng.random_range(0.0..10.0)),
                put_call_ratio: Some(0.8 + rng.random_range(0.0..0.4)),
            };

            time_series.push(market_data);
        }

        data.insert(symbol.clone(), time_series);
    }

    data
}

/// Print a simple ASCII equity curve
fn print_equity_curve(equity: &[f64]) {
    if equity.is_empty() {
        return;
    }

    let min = equity.iter().copied().fold(f64::INFINITY, f64::min);
    let max = equity.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = max - min;

    // Sample 60 points for display
    let step = (equity.len() / 60).max(1);
    let samples: Vec<f64> = equity.iter().step_by(step).copied().collect();

    println!("  ${:.0}", max);
    println!("  │");

    // Print 10 rows
    for row in 0..10 {
        print!("  │");
        let threshold = max - (range * row as f64 / 10.0);

        for &value in &samples {
            if value >= threshold {
                print!("█");
            } else {
                print!(" ");
            }
        }
        println!();
    }

    println!("  └{}", "─".repeat(samples.len()));
    println!("  ${:.0}", min);
    println!("  (Time →)");
    println!();
}
