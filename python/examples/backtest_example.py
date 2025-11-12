"""
Backtest Example - Testing a Simple Strategy with Python

This example demonstrates:
1. Creating trading signals manually
2. Running a backtest with the Rust engine
3. Analyzing performance metrics
4. Understanding risk-adjusted returns

**Human Insight**: This shows how to validate strategies before risking real capital
"""

import quant_engine as qe
from datetime import datetime, timedelta


def create_sample_signals():
    """
    Create a simple moving average crossover strategy signals

    In a real strategy, you'd generate these from historical data and indicators.
    For this example, we'll manually create signals as if we detected crossovers.
    """
    signals = []

    # Simulate detecting a bullish crossover on Jan 1
    signal1 = qe.Signal(
        symbol=qe.Symbol("SPY"),
        action=qe.SignalAction.Buy,
        confidence=0.75,
        reason="Bullish MA crossover - fast crossed above slow",
        source="SimpleMAStrategy"
    )
    # Set the target price (entry price for backtest)
    signal1.target_price = qe.Price(450.0)
    signal1.stop_loss = qe.Price(445.0)  # 1.1% stop loss
    signal1.take_profit = qe.Price(460.0)  # 2.2% take profit
    signals.append(signal1)

    # Hold for 10 days...

    # Simulate another bullish signal on QQQ while holding SPY
    signal2 = qe.Signal(
        symbol=qe.Symbol("QQQ"),
        action=qe.SignalAction.Buy,
        confidence=0.80,
        reason="Tech sector breakout",
        source="SimpleMAStrategy"
    )
    signal2.target_price = qe.Price(380.0)
    signal2.stop_loss = qe.Price(374.0)  # 1.6% stop
    signal2.take_profit = qe.Price(392.0)  # 3.2% profit
    signals.append(signal2)

    # Exit SPY after 15 days (take profit hit)
    signal3 = qe.Signal(
        symbol=qe.Symbol("SPY"),
        action=qe.SignalAction.Close,
        confidence=1.0,
        reason="Take profit target reached",
        source="SimpleMAStrategy"
    )
    signal3.target_price = qe.Price(460.0)  # Exit at take profit
    signals.append(signal3)

    # Exit QQQ after 20 days
    signal4 = qe.Signal(
        symbol=qe.Symbol("QQQ"),
        action=qe.SignalAction.Close,
        confidence=1.0,
        reason="Position hold time limit",
        source="SimpleMAStrategy"
    )
    signal4.target_price = qe.Price(388.0)  # Exit with profit
    signals.append(signal4)

    return signals


def run_backtest():
    """
    Run backtest and analyze results
    """
    print("=" * 60)
    print("SIMPLE BACKTEST EXAMPLE")
    print("=" * 60)
    print()

    # 1. Create signals
    print("📊 Generating signals...")
    signals = create_sample_signals()
    print(f"   Generated {len(signals)} signals")
    print()

    # 2. Create backtester
    print("🔧 Setting up backtester...")
    backtester = qe.Backtester(
        initial_capital=10_000.0,
        commission=1.0,  # $1 per trade
        slippage=0.05    # 5 basis points (0.05%)
    )
    print(f"   {backtester}")
    print()

    # 3. Run backtest
    print("🚀 Running backtest...")
    result = backtester.run(signals)
    print()

    # 4. Display results
    print("=" * 60)
    print("BACKTEST RESULTS")
    print("=" * 60)
    print()

    metrics = result.metrics()

    print(f"💰 Final Portfolio Value: ${result.final_portfolio_value():,.2f}")
    print(f"📈 Total Return:          {metrics.total_return():.2%}")
    print(f"📊 Total Trades:          {result.total_trades()}")
    print()

    print("RISK-ADJUSTED METRICS")
    print("-" * 60)
    print(f"📐 Sharpe Ratio:          {metrics.sharpe_ratio():.2f}")
    print(f"📉 Sortino Ratio:         {metrics.sortino_ratio():.2f}")
    print(f"🎯 Max Drawdown:          {metrics.max_drawdown():.2%}")
    print()

    print("WIN/LOSS STATISTICS")
    print("-" * 60)
    print(f"✅ Win Rate:              {metrics.win_rate():.2%}")
    print(f"💵 Avg Win:               ${metrics.avg_win():.2f}")
    print(f"💸 Avg Loss:              ${metrics.avg_loss():.2f}")
    print(f"⚖️  Profit Factor:         {metrics.profit_factor():.2f}")
    print()

    # 5. Strategy evaluation
    print("=" * 60)
    print("STRATEGY EVALUATION")
    print("=" * 60)
    print()

    # Evaluate performance
    if metrics.sharpe_ratio() > 2.0:
        print("✅ EXCELLENT: Sharpe > 2.0 indicates strong risk-adjusted returns")
    elif metrics.sharpe_ratio() > 1.0:
        print("✅ GOOD: Sharpe > 1.0 indicates acceptable risk-adjusted returns")
    else:
        print("⚠️  WARNING: Sharpe < 1.0 indicates poor risk-adjusted returns")
    print()

    if metrics.win_rate() > 0.60:
        print("✅ GOOD: Win rate > 60% shows consistent signal quality")
    elif metrics.win_rate() > 0.50:
        print("✅ OK: Win rate > 50% but could be improved")
    else:
        print("⚠️  WARNING: Win rate < 50% needs improvement")
    print()

    if metrics.max_drawdown() < -0.20:
        print("⚠️  WARNING: Max drawdown > 20% indicates high risk")
    elif metrics.max_drawdown() < -0.10:
        print("⚠️  CAUTION: Max drawdown > 10% is moderate risk")
    else:
        print("✅ GOOD: Drawdown well-controlled")
    print()

    # 6. Next steps
    print("=" * 60)
    print("NEXT STEPS")
    print("=" * 60)
    print()
    print("1. Generate signals from real historical data")
    print("2. Test with different market conditions (bull, bear, sideways)")
    print("3. Optimize position sizing and risk parameters")
    print("4. Add more alpha models and test combinations")
    print("5. Walk-forward testing to avoid overfitting")
    print()


if __name__ == "__main__":
    run_backtest()
