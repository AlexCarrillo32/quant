#!/usr/bin/env python3
"""
Simple Python Strategy Example

Demonstrates how to use the quant_engine Python bindings to create
trading strategies in Python while leveraging Rust's execution speed.

This strategy implements a basic momentum + mean reversion hybrid.
"""

from typing import List, Optional
import quant_engine as qe


class MomentumMeanReversionStrategy:
    """
    Hybrid strategy combining momentum and mean reversion

    Human Insight:
    - Momentum works in trending markets (ride the trend)
    - Mean reversion works in ranging markets (fade extremes)
    - Combine both: follow strong trends, fade weak moves
    """

    def __init__(
        self,
        momentum_threshold: float = 2.0,  # % move to trigger momentum
        reversion_threshold: float = 5.0,  # % move to trigger reversion
        confidence_momentum: float = 0.75,
        confidence_reversion: float = 0.80,
    ):
        self.momentum_threshold = momentum_threshold
        self.reversion_threshold = reversion_threshold
        self.confidence_momentum = confidence_momentum
        self.confidence_reversion = confidence_reversion

    def analyze(
        self,
        symbol: str,
        current_price: float,
        prev_price: float,
        avg_price_20d: float,
    ) -> Optional[qe.Signal]:
        """
        Analyze a symbol and generate a signal

        Args:
            symbol: Stock symbol (e.g., "AAPL")
            current_price: Current price
            prev_price: Previous close price
            avg_price_20d: 20-day moving average

        Returns:
            Signal if conditions are met, None otherwise
        """
        # Calculate metrics
        pct_change = ((current_price - prev_price) / prev_price) * 100
        deviation_from_avg = ((current_price - avg_price_20d) / avg_price_20d) * 100

        # Create symbol and price objects
        sym = qe.Symbol(symbol)
        price = qe.Price(current_price)
        prev = qe.Price(prev_price)

        # Momentum strategy: Strong move up or down
        if abs(pct_change) > self.momentum_threshold and abs(
            pct_change
        ) < self.reversion_threshold:
            action = qe.SignalAction.Buy if pct_change > 0 else qe.SignalAction.Sell
            confidence = qe.Confidence(self.confidence_momentum)

            signal = qe.Signal.new(
                sym,
                action,
                confidence,
                f"Momentum: {pct_change:.2f}% move, following trend",
                "MomentumMeanReversion",
            )

            # Set risk management levels
            if action == qe.SignalAction.Buy:
                signal.with_target_price(qe.Price(current_price * 1.03))  # +3% target
                signal.with_stop_loss(qe.Price(current_price * 0.99))  # -1% stop
            else:
                signal.with_target_price(qe.Price(current_price * 0.97))  # -3% target
                signal.with_stop_loss(qe.Price(current_price * 1.01))  # +1% stop

            signal.with_quantity(qe.Quantity.buy(100))

            return signal

        # Mean reversion: Extreme deviation from average
        if abs(deviation_from_avg) > self.reversion_threshold:
            # Fade the extreme: buy if too low, sell if too high
            action = (
                qe.SignalAction.Buy
                if deviation_from_avg < 0
                else qe.SignalAction.Sell
            )
            confidence = qe.Confidence(self.confidence_reversion)

            signal = qe.Signal.new(
                sym,
                action,
                confidence,
                f"Mean reversion: {deviation_from_avg:.2f}% from 20d avg",
                "MomentumMeanReversion",
            )

            # Tighter stops for mean reversion (riskier)
            if action == qe.SignalAction.Buy:
                signal.with_target_price(qe.Price(avg_price_20d))  # Target = average
                signal.with_stop_loss(qe.Price(current_price * 0.98))  # -2% stop
            else:
                signal.with_target_price(qe.Price(avg_price_20d))
                signal.with_stop_loss(qe.Price(current_price * 1.02))

            signal.with_quantity(qe.Quantity.buy(100))

            return signal

        return None


def main():
    """Example usage"""
    strategy = MomentumMeanReversionStrategy()

    # Example 1: Momentum signal (strong upward move)
    print("Example 1: Momentum Signal")
    signal1 = strategy.analyze(
        symbol="AAPL",
        current_price=180.50,
        prev_price=176.00,  # +2.5% move
        avg_price_20d=175.00,
    )

    if signal1:
        print(f"  {signal1}")
        print(f"  Actionable: {signal1.is_actionable()}")
        if signal1.target_price and signal1.stop_loss:
            current = qe.Price(180.50)
            rr = signal1.risk_reward_ratio(current)
            print(f"  Risk/Reward: {rr:.2f}" if rr else "  R/R: N/A")
    else:
        print("  No signal")

    print()

    # Example 2: Mean reversion signal (extreme deviation)
    print("Example 2: Mean Reversion Signal")
    signal2 = strategy.analyze(
        symbol="GOOGL",
        current_price=140.00,
        prev_price=142.00,
        avg_price_20d=150.00,  # -6.7% below average (extreme)
    )

    if signal2:
        print(f"  {signal2}")
        print(f"  Actionable: {signal2.is_actionable()}")
    else:
        print("  No signal")

    print()

    # Example 3: No signal (normal market)
    print("Example 3: No Signal (Normal Market)")
    signal3 = strategy.analyze(
        symbol="MSFT",
        current_price=375.00,
        prev_price=374.00,  # +0.27% (too small)
        avg_price_20d=373.00,
    )

    if signal3:
        print(f"  {signal3}")
    else:
        print("  No signal - market within normal range")

    # Demonstrate type safety
    print("\nType Safety Demo:")
    try:
        bad_price = qe.Price(-100.0)  # This will raise ValueError
    except ValueError as e:
        print(f"  ✓ Caught invalid price: {e}")

    try:
        bad_symbol = qe.Symbol("")  # This will raise ValueError
    except ValueError as e:
        print(f"  ✓ Caught invalid symbol: {e}")

    try:
        bad_confidence = qe.Confidence(1.5)  # This will raise ValueError
    except ValueError as e:
        print(f"  ✓ Caught invalid confidence: {e}")


if __name__ == "__main__":
    main()
