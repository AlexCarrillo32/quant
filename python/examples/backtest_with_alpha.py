"""
Advanced Backtest Example - Custom Alpha Model Integration

This example demonstrates:
1. Creating a custom Python alpha model
2. Generating signals from historical data (simulated)
3. Backtesting with risk management integration
4. Advanced performance analysis

**Human Insight**: Alpha models encode human psychology and market inefficiencies
"""

import quant_engine as qe
from typing import List
import random


class SimpleMomentumAlpha:
    """
    Simple momentum strategy that buys on strength, sells on weakness

    **Human Insight**: Markets exhibit momentum due to human psychology:
    - Herding behavior: "Everyone else is buying, I should too"
    - FOMO: Fear of missing out drives continuation
    - Confirmation bias: Winners attract more buyers

    This is a simple example. Real strategies would use actual indicators.
    """

    def __init__(self, lookback_period: int = 20, threshold: float = 0.02):
        self.lookback_period = lookback_period
        self.threshold = threshold  # 2% move to trigger signal
        self.name = "SimpleMomentumAlpha"

    def generate_signals(
        self,
        symbol: str,
        prices: List[float],
        start_idx: int = 0
    ) -> List[qe.Signal]:
        """
        Generate signals based on momentum

        Args:
            symbol: Stock symbol
            prices: Historical price data
            start_idx: Where to start generating signals

        Returns:
            List of trading signals
        """
        signals = []

        # Need enough data for lookback
        if len(prices) < self.lookback_period:
            return signals

        # Scan through data looking for momentum
        for i in range(start_idx + self.lookback_period, len(prices)):
            current_price = prices[i]
            past_price = prices[i - self.lookback_period]

            # Calculate momentum (% change over lookback)
            momentum = (current_price - past_price) / past_price

            # Generate signals based on momentum
            if momentum > self.threshold:
                # Strong upward momentum - BUY signal
                confidence = min(0.5 + (momentum - self.threshold) / 0.1, 0.95)

                signal = qe.Signal(
                    symbol=qe.Symbol(symbol),
                    action=qe.SignalAction.Buy,
                    confidence=confidence,
                    reason=f"Strong momentum: {momentum:.2%} over {self.lookback_period} days",
                    source=self.name
                )

                # Set entry price and risk parameters
                signal.target_price = qe.Price(current_price)
                signal.stop_loss = qe.Price(current_price * 0.98)  # 2% stop
                signal.take_profit = qe.Price(current_price * 1.04)  # 4% profit

                signals.append(signal)

            elif momentum < -self.threshold:
                # Strong downward momentum - close long positions
                # (In a real strategy, you might SELL/SHORT here)

                signal = qe.Signal(
                    symbol=qe.Symbol(symbol),
                    action=qe.SignalAction.Close,
                    confidence=0.8,
                    reason=f"Negative momentum: {momentum:.2%}",
                    source=self.name
                )
                signal.target_price = qe.Price(current_price)

                signals.append(signal)

        return signals


def simulate_price_data(
    initial_price: float,
    num_days: int,
    trend: float = 0.0005,
    volatility: float = 0.02
) -> List[float]:
    """
    Simulate random price data with trend and volatility

    Args:
        initial_price: Starting price
        num_days: Number of days to simulate
        trend: Daily drift (default: 0.05% per day)
        volatility: Daily volatility (default: 2%)

    Returns:
        List of simulated prices
    """
    prices = [initial_price]

    for _ in range(num_days - 1):
        # Random walk with drift
        daily_return = trend + (random.gauss(0, 1) * volatility)
        new_price = prices[-1] * (1 + daily_return)
        prices.append(new_price)

    return prices


def run_advanced_backtest():
    """
    Run backtest with custom alpha model
    """
    print("=" * 70)
    print("ADVANCED BACKTEST - CUSTOM ALPHA MODEL")
    print("=" * 70)
    print()

    # Set random seed for reproducibility
    random.seed(42)

    # 1. Simulate market data
    print("📊 Simulating market data...")
    spy_prices = simulate_price_data(450.0, 252, trend=0.0008, volatility=0.015)
    qqq_prices = simulate_price_data(380.0, 252, trend=0.001, volatility=0.02)
    print(f"   Generated {len(spy_prices)} days of SPY data")
    print(f"   Generated {len(qqq_prices)} days of QQQ data")
    print()

    # 2. Create alpha model
    print("🧠 Initializing alpha model...")
    alpha = SimpleMomentumAlpha(lookback_period=20, threshold=0.03)
    print(f"   Model: {alpha.name}")
    print(f"   Lookback: {alpha.lookback_period} days")
    print(f"   Threshold: {alpha.threshold:.1%}")
    print()

    # 3. Generate signals
    print("🎯 Generating trading signals...")
    spy_signals = alpha.generate_signals("SPY", spy_prices)
    qqq_signals = alpha.generate_signals("QQQ", qqq_prices)

    all_signals = spy_signals + qqq_signals
    all_signals.sort(key=lambda s: s.timestamp)  # Sort by timestamp

    print(f"   SPY signals: {len(spy_signals)}")
    print(f"   QQQ signals: {len(qqq_signals)}")
    print(f"   Total signals: {len(all_signals)}")
    print()

    if len(all_signals) == 0:
        print("⚠️  No signals generated. Try lowering the threshold.")
        return

    # 4. Create backtester with realistic costs
    print("🔧 Configuring backtester...")
    backtester = qe.Backtester(
        initial_capital=50_000.0,
        commission=0.5,   # $0.50 per trade (low-cost broker)
        slippage=0.03     # 3 basis points
    )
    print(f"   {backtester}")
    print()

    # 5. Run backtest
    print("🚀 Running backtest...")
    print("   This includes:")
    print("   - Risk management checks")
    print("   - Position sizing")
    print("   - Slippage simulation")
    print("   - Commission deduction")
    print()

    result = backtester.run(all_signals)

    # 6. Analyze results
    print("=" * 70)
    print("BACKTEST RESULTS")
    print("=" * 70)
    print()

    metrics = result.metrics()

    print("PORTFOLIO PERFORMANCE")
    print("-" * 70)
    print(f"💰 Initial Capital:       ${50_000:,.2f}")
    print(f"💰 Final Value:           ${result.final_portfolio_value():,.2f}")
    print(f"📈 Total Return:          {metrics.total_return():.2%}")
    print(f"📈 Annualized Return:     {metrics.annualized_return():.2%}")
    print()

    print("RISK METRICS")
    print("-" * 70)
    print(f"📊 Sharpe Ratio:          {metrics.sharpe_ratio():.3f}")
    print(f"📉 Sortino Ratio:         {metrics.sortino_ratio():.3f}")
    print(f"🎯 Max Drawdown:          {metrics.max_drawdown():.2%}")
    print(f"📊 Volatility:            {metrics.volatility():.2%}")
    print()

    print("TRADE STATISTICS")
    print("-" * 70)
    print(f"📊 Total Trades:          {result.total_trades()}")
    print(f"✅ Win Rate:              {metrics.win_rate():.2%}")
    print(f"💵 Average Win:           ${metrics.avg_win():,.2f}")
    print(f"💸 Average Loss:          ${metrics.avg_loss():,.2f}")
    print(f"⚖️  Profit Factor:         {metrics.profit_factor():.2f}")
    print()

    # 7. Performance evaluation with human insight
    print("=" * 70)
    print("STRATEGY EVALUATION")
    print("=" * 70)
    print()

    # Sharpe ratio evaluation
    sharpe = metrics.sharpe_ratio()
    print("📐 SHARPE RATIO ANALYSIS:")
    if sharpe > 2.0:
        print("   ✅ EXCELLENT (>2.0): Institutional-grade risk-adjusted returns")
        print("   💡 This strategy efficiently captures returns vs. risk")
    elif sharpe > 1.0:
        print("   ✅ GOOD (1.0-2.0): Solid risk-adjusted returns")
        print("   💡 Consider optimization to push above 2.0")
    elif sharpe > 0.5:
        print("   ⚠️  FAIR (0.5-1.0): Mediocre risk-adjusted returns")
        print("   💡 Needs improvement - too much risk for the return")
    else:
        print("   ❌ POOR (<0.5): Unacceptable risk-adjusted returns")
        print("   💡 Revisit strategy logic or risk management")
    print()

    # Win rate evaluation
    win_rate = metrics.win_rate()
    print("✅ WIN RATE ANALYSIS:")
    if win_rate > 0.60:
        print(f"   ✅ STRONG (>{60}%): High conviction signals")
        print("   💡 Good signal quality - focus on increasing position sizes")
    elif win_rate > 0.50:
        print(f"   ✅ POSITIVE (>{50}%): Edge exists but could improve")
        print("   💡 Consider tighter filters or better entry timing")
    else:
        print(f"   ❌ WEAK (<{50}%): No statistical edge")
        print("   💡 Strategy needs major revision")
    print()

    # Profit factor
    pf = metrics.profit_factor()
    print("⚖️  PROFIT FACTOR ANALYSIS:")
    if pf > 2.0:
        print("   ✅ EXCELLENT (>2.0): Wins significantly outweigh losses")
    elif pf > 1.5:
        print("   ✅ GOOD (1.5-2.0): Decent profit/loss ratio")
    elif pf > 1.0:
        print("   ⚠️  MARGINAL (1.0-1.5): Barely profitable")
    else:
        print("   ❌ LOSING (<1.0): Total losses exceed total wins")
    print()

    # Max drawdown
    mdd = metrics.max_drawdown()
    print("🎯 DRAWDOWN ANALYSIS:")
    if mdd > -0.10:
        print("   ✅ EXCELLENT (<10%): Low risk of ruin")
        print("   💡 Psychological stress is manageable")
    elif mdd > -0.20:
        print("   ⚠️  MODERATE (10-20%): Acceptable for most traders")
        print("   💡 Expect some uncomfortable periods")
    else:
        print("   ❌ HIGH (>20%): Significant drawdown risk")
        print("   💡 Most traders can't stomach this - reduce risk")
    print()

    # 8. Next steps
    print("=" * 70)
    print("NEXT STEPS FOR LIVE TRADING")
    print("=" * 70)
    print()
    print("1. ✅ Walk-forward testing (train on past, test on future)")
    print("2. ✅ Out-of-sample testing (test on different symbols)")
    print("3. ✅ Stress testing (2008 crash, COVID crash, etc.)")
    print("4. ✅ Monte Carlo simulation (test robustness)")
    print("5. ✅ Paper trading (forward test with live data)")
    print("6. 🚀 Small capital live trading (final validation)")
    print()

    # Human psychology reminder
    print("=" * 70)
    print("HUMAN PSYCHOLOGY REMINDER")
    print("=" * 70)
    print()
    print("📖 Backtest results are ALWAYS better than live trading because:")
    print()
    print("   • 🧠 No emotional decisions (fear, greed, FOMO)")
    print("   • ⏰ No execution delays or slippage surprises")
    print("   • 📊 No look-ahead bias from hindsight")
    print("   • 💰 No unexpected costs or fees")
    print()
    print("💡 Always trade smaller than backtests suggest!")
    print("💡 Expect 20-30% worse performance in live trading!")
    print()


if __name__ == "__main__":
    run_advanced_backtest()
