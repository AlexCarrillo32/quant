"""
Basic Strategy Example
Demonstrates how to use the enhanced alpha models.
"""
from datetime import datetime, timedelta
import numpy as np
from typing import Dict, List

# Import our enhanced alpha models
import sys
sys.path.append('..')

from alphas import (
    EnhancedMacdAlphaModel,
    EnhancedEmaCrossAlphaModel,
    Insight,
    InsightDirection
)


def generate_sample_data(symbol: str, days: int = 100) -> Dict:
    """
    Generate sample OHLCV data for testing.

    In production, replace this with real market data from:
    - yfinance
    - Alpaca
    - Interactive Brokers
    - etc.
    """
    np.random.seed(42)

    # Generate price data with trend
    base_price = 100.0
    trend = np.linspace(0, 20, days)
    noise = np.random.normal(0, 2, days)
    prices = base_price + trend + noise + np.cumsum(np.random.normal(0, 0.5, days))

    # Generate volume data
    base_volume = 1000000
    volumes = base_volume + np.random.normal(0, 200000, days)
    volumes = np.abs(volumes)  # Volume must be positive

    return {
        'symbol': symbol,
        'close': prices.tolist(),
        'volume': volumes.tolist(),
        'dates': [datetime.now() - timedelta(days=days-i) for i in range(days)]
    }


def run_macd_strategy():
    """
    Example: Running MACD strategy on sample data.
    """
    print("=" * 60)
    print("ENHANCED MACD ALPHA MODEL EXAMPLE")
    print("=" * 60)

    # Initialize the alpha model
    macd_alpha = EnhancedMacdAlphaModel(
        fast_period=12,
        slow_period=26,
        signal_period=9,
        base_threshold=0.01,
        holding_period_minutes=240  # 4 hours
    )

    # Generate sample data for multiple symbols
    symbols = ['AAPL', 'GOOGL', 'MSFT']
    market_data = {}

    for symbol in symbols:
        market_data[symbol] = generate_sample_data(symbol)

    # Add securities to the alpha model
    macd_alpha.on_securities_changed(added=symbols, removed=[])

    # Simulate running the strategy over time
    print(f"\nRunning strategy on {len(symbols)} symbols...")
    print(f"Data points per symbol: {len(market_data['AAPL']['close'])}\n")

    insights_by_day = []
    days_to_simulate = 30  # Last 30 days

    for day_offset in range(days_to_simulate):
        # Get data up to current day
        current_data = {}
        for symbol in symbols:
            data = market_data[symbol]
            end_idx = len(data['close']) - days_to_simulate + day_offset + 1

            current_data[symbol] = {
                'close': data['close'][:end_idx],
                'volume': data['volume'][:end_idx]
            }

        current_time = market_data['AAPL']['dates'][-days_to_simulate + day_offset]

        # Generate insights
        insights = macd_alpha.update(current_data, current_time)

        if insights:
            insights_by_day.append((current_time, insights))

    # Display results
    print(f"Total insights generated: {macd_alpha.insights_generated}\n")

    if insights_by_day:
        print("Sample Insights (first 5):")
        print("-" * 60)

        for i, (timestamp, insights) in enumerate(insights_by_day[:5]):
            print(f"\nDate: {timestamp.strftime('%Y-%m-%d')}")
            for insight in insights:
                direction_symbol = "📈" if insight.direction == InsightDirection.UP else "📉"
                print(f"  {direction_symbol} {insight.symbol}")
                print(f"     Direction: {insight.direction.name}")
                print(f"     Confidence: {insight.confidence:.2%}")
                print(f"     Signal Strength: {insight.signal_strength:.3f}")
                print(f"     Stop Loss: {insight.stop_loss_pct:.2f}%")
                print(f"     Take Profit: {insight.take_profit_pct:.2f}%")
                print(f"     MACD: {insight.metadata['macd']:.3f}")
                print(f"     Volatility: {insight.metadata['volatility']:.4f}")

    # Display statistics
    print("\n" + "=" * 60)
    print("STRATEGY STATISTICS")
    print("=" * 60)
    stats = macd_alpha.get_statistics()
    for key, value in stats.items():
        print(f"{key}: {value}")


def run_ema_cross_strategy():
    """
    Example: Running EMA Cross strategy on sample data.
    """
    print("\n\n" + "=" * 60)
    print("ENHANCED EMA CROSS ALPHA MODEL EXAMPLE")
    print("=" * 60)

    # Initialize the alpha model
    ema_alpha = EnhancedEmaCrossAlphaModel(
        fast_period=12,
        slow_period=26,
        holding_period_minutes=480,  # 8 hours
        min_trend_strength=0.3,
        volume_confirmation=True
    )

    # Generate sample data
    symbols = ['TSLA', 'NVDA']
    market_data = {}

    for symbol in symbols:
        market_data[symbol] = generate_sample_data(symbol, days=150)

    # Add securities
    ema_alpha.on_securities_changed(added=symbols, removed=[])

    # Simulate strategy
    print(f"\nRunning strategy on {len(symbols)} symbols...")
    print(f"Data points per symbol: {len(market_data['TSLA']['close'])}\n")

    insights_generated = 0
    days_to_simulate = 50

    for day_offset in range(days_to_simulate):
        current_data = {}
        for symbol in symbols:
            data = market_data[symbol]
            end_idx = len(data['close']) - days_to_simulate + day_offset + 1

            current_data[symbol] = {
                'close': data['close'][:end_idx],
                'volume': data['volume'][:end_idx]
            }

        current_time = market_data['TSLA']['dates'][-days_to_simulate + day_offset]

        insights = ema_alpha.update(current_data, current_time)
        insights_generated += len(insights)

        if insights:
            print(f"\n{current_time.strftime('%Y-%m-%d')}:")
            for insight in insights:
                direction_symbol = "🚀" if insight.direction == InsightDirection.UP else "🔻"
                print(f"  {direction_symbol} {insight.symbol} - "
                      f"Confidence: {insight.confidence:.2%}, "
                      f"Trend Strength: {insight.metadata['trend_strength']:.2f}, "
                      f"Volume Confirmed: {insight.metadata['volume_confirmed']}")

    print(f"\n\nTotal insights: {insights_generated}")
    print(f"Statistics: {ema_alpha.get_statistics()}")


def compare_strategies():
    """
    Example: Compare MACD vs EMA Cross on the same data.
    """
    print("\n\n" + "=" * 60)
    print("STRATEGY COMPARISON: MACD vs EMA CROSS")
    print("=" * 60)

    # Initialize both models
    macd_alpha = EnhancedMacdAlphaModel()
    ema_alpha = EnhancedEmaCrossAlphaModel()

    # Test on same symbol
    symbol = 'SPY'
    market_data = generate_sample_data(symbol, days=200)

    # Add to both models
    macd_alpha.on_securities_changed(added=[symbol], removed=[])
    ema_alpha.on_securities_changed(added=[symbol], removed=[])

    # Run both strategies
    macd_insights_count = 0
    ema_insights_count = 0

    for day_offset in range(100):
        end_idx = 100 + day_offset + 1
        current_data = {
            symbol: {
                'close': market_data['close'][:end_idx],
                'volume': market_data['volume'][:end_idx]
            }
        }
        current_time = market_data['dates'][end_idx - 1]

        macd_insights = macd_alpha.update(current_data, current_time)
        ema_insights = ema_alpha.update(current_data, current_time)

        macd_insights_count += len(macd_insights)
        ema_insights_count += len(ema_insights)

    print(f"\nResults for {symbol} over 100 trading days:")
    print(f"  MACD Insights: {macd_insights_count}")
    print(f"  EMA Cross Insights: {ema_insights_count}")
    print(f"\nMACD tends to generate {'more' if macd_insights_count > ema_insights_count else 'fewer'} signals")
    print("This is expected as they use different confirmation logic.")


if __name__ == "__main__":
    # Run all examples
    run_macd_strategy()
    run_ema_cross_strategy()
    compare_strategies()

    print("\n\n" + "=" * 60)
    print("NEXT STEPS")
    print("=" * 60)
    print("1. Replace sample data with real market data (yfinance, Alpaca, etc.)")
    print("2. Add portfolio construction module to manage position sizing")
    print("3. Implement execution module for order placement")
    print("4. Add comprehensive backtesting with performance metrics")
    print("5. Optimize parameters using walk-forward analysis")
    print("6. Implement risk management (stop loss, position limits, etc.)")
