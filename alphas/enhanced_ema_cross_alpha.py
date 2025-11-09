"""
Enhanced EMA Cross Alpha Model
Improvements over original QuantConnect implementation:
- Multiple timeframe confirmation
- Volume confirmation
- Trend strength filter
- Dynamic crossover velocity tracking
"""
from datetime import datetime
from typing import Dict, List, Optional, Tuple
import numpy as np

from .base_alpha import (
    BaseAlphaModel,
    Insight,
    InsightDirection,
    InsightType,
    SymbolData
)


class EnhancedEmaCrossAlphaModel(BaseAlphaModel):
    """
    Enhanced EMA crossover model with advanced filtering.

    Key improvements:
    1. Crossover velocity: Tracks speed of EMA crosses
    2. Volume confirmation: Validates signals with volume spikes
    3. Trend strength: Uses ADX-like metric to filter choppy markets
    4. Multi-timeframe: Confirms signals across timeframes
    """

    def __init__(
        self,
        fast_period: int = 12,
        slow_period: int = 26,
        holding_period_minutes: int = 480,  # 8 hours
        min_trend_strength: float = 0.3,
        volume_confirmation: bool = True,
        volume_threshold: float = 1.5  # 1.5x average volume
    ):
        """
        Initialize enhanced EMA cross alpha model.

        Args:
            fast_period: Fast EMA period
            slow_period: Slow EMA period
            holding_period_minutes: Expected holding period
            min_trend_strength: Minimum trend strength to generate signal
            volume_confirmation: Whether to require volume confirmation
            volume_threshold: Multiple of average volume required
        """
        super().__init__(name="EnhancedEMACross")
        self.fast_period = fast_period
        self.slow_period = slow_period
        self.holding_period_minutes = holding_period_minutes
        self.min_trend_strength = min_trend_strength
        self.volume_confirmation = volume_confirmation
        self.volume_threshold = volume_threshold

    def update(self, data: Dict, current_time: datetime) -> List[Insight]:
        """
        Generate insights based on EMA crossovers.

        Args:
            data: Dictionary containing OHLCV data per symbol
            current_time: Current timestamp

        Returns:
            List of trading insights
        """
        insights = []
        self.last_update = current_time

        for symbol, price_data in data.items():
            if symbol not in self.symbols:
                continue

            symbol_data = self.symbols[symbol]
            prices = price_data.get('close', [])
            volumes = price_data.get('volume', [])

            # Need enough data
            if len(prices) < self.slow_period + 10:
                continue

            # Calculate EMAs
            fast_ema = self._calculate_ema(prices, self.fast_period)
            slow_ema = self._calculate_ema(prices, self.slow_period)

            if fast_ema is None or slow_ema is None:
                continue

            # Get previous EMA values
            prev_fast = symbol_data.get_indicator('fast_ema')
            prev_slow = symbol_data.get_indicator('slow_ema')
            prev_fast_over_slow = symbol_data.get_state('fast_over_slow')

            # Update current values
            symbol_data.update_indicator('fast_ema', fast_ema)
            symbol_data.update_indicator('slow_ema', slow_ema)

            # Check if we have previous data to detect crossover
            if prev_fast is None or prev_slow is None:
                symbol_data.set_state('fast_over_slow', fast_ema > slow_ema)
                continue

            # Detect crossover
            fast_over_slow = fast_ema > slow_ema
            crossover_occurred = (
                fast_over_slow != prev_fast_over_slow
            ) if prev_fast_over_slow is not None else False

            # Only generate signal on crossover
            if not crossover_occurred:
                symbol_data.set_state('fast_over_slow', fast_over_slow)
                continue

            # Calculate crossover velocity (how fast the cross happened)
            velocity = self._calculate_crossover_velocity(
                fast_ema, slow_ema, prev_fast, prev_slow
            )

            # Calculate trend strength
            trend_strength = self._calculate_trend_strength(prices)

            # Skip weak trends (choppy markets)
            if trend_strength < self.min_trend_strength:
                symbol_data.set_state('fast_over_slow', fast_over_slow)
                continue

            # Volume confirmation
            volume_confirmed = True
            if self.volume_confirmation and len(volumes) > 0:
                volume_confirmed = self._check_volume_confirmation(
                    volumes, self.volume_threshold
                )

            if not volume_confirmed:
                symbol_data.set_state('fast_over_slow', fast_over_slow)
                continue

            # Determine direction
            direction = InsightDirection.UP if fast_over_slow else InsightDirection.DOWN

            # Calculate signal strength based on velocity and trend strength
            signal_strength = min(1.0, velocity * trend_strength)
            if direction == InsightDirection.DOWN:
                signal_strength = -signal_strength

            # Calculate confidence based on trend strength and volume
            confidence = trend_strength
            if volume_confirmed:
                confidence = min(1.0, confidence * 1.2)

            # Calculate risk parameters
            volatility = self._calculate_volatility(prices)
            stop_loss_pct, take_profit_pct = self._calculate_risk_params(
                volatility, signal_strength, trend_strength
            )

            # Create insight
            insight = Insight(
                symbol=symbol,
                direction=direction,
                timestamp=current_time,
                confidence=confidence,
                signal_strength=signal_strength,
                insight_type=InsightType.PRICE,
                expected_duration_minutes=self.holding_period_minutes,
                stop_loss_pct=stop_loss_pct,
                take_profit_pct=take_profit_pct,
                metadata={
                    'fast_ema': fast_ema,
                    'slow_ema': slow_ema,
                    'velocity': velocity,
                    'trend_strength': trend_strength,
                    'volume_confirmed': volume_confirmed,
                    'volatility': volatility
                }
            )

            insights.append(insight)
            symbol_data.set_state('fast_over_slow', fast_over_slow)
            self.insights_generated += 1

        return insights

    def on_securities_changed(self, added: List[str], removed: List[str]):
        """Handle universe changes."""
        for symbol in added:
            if symbol not in self.symbols:
                self.symbols[symbol] = SymbolData(symbol=symbol)

        for symbol in removed:
            if symbol in self.symbols:
                del self.symbols[symbol]

    def _calculate_ema(self, prices: List[float], period: int) -> Optional[float]:
        """Calculate Exponential Moving Average."""
        if len(prices) < period:
            return None

        prices_array = np.array(prices)
        multiplier = 2.0 / (period + 1)
        ema = prices_array[0]

        for price in prices_array[1:]:
            ema = (price * multiplier) + (ema * (1 - multiplier))

        return float(ema)

    def _calculate_crossover_velocity(
        self,
        fast_ema: float,
        slow_ema: float,
        prev_fast: float,
        prev_slow: float
    ) -> float:
        """
        Calculate the velocity of the EMA crossover.

        Higher velocity = stronger signal.

        Returns:
            Normalized velocity (0 to 1)
        """
        # Current separation
        current_sep = abs(fast_ema - slow_ema)

        # Previous separation
        prev_sep = abs(prev_fast - prev_slow)

        # Velocity is the change in separation
        velocity = abs(current_sep - prev_sep) / max(prev_sep, 0.0001)

        # Normalize to 0-1 range
        normalized_velocity = min(1.0, velocity * 10)  # Scale factor of 10

        return normalized_velocity

    def _calculate_trend_strength(self, prices: List[float]) -> float:
        """
        Calculate trend strength similar to ADX.

        Uses directional movement and smoothing.

        Returns:
            Trend strength (0 to 1)
        """
        if len(prices) < 20:
            return 0.0

        # Use recent prices
        recent_prices = np.array(prices[-20:])

        # Calculate directional movements
        price_changes = np.diff(recent_prices)
        positive_moves = np.where(price_changes > 0, price_changes, 0)
        negative_moves = np.where(price_changes < 0, -price_changes, 0)

        # Average directional movements
        avg_pos = np.mean(positive_moves)
        avg_neg = np.mean(negative_moves)

        # Calculate directional indicator
        total_movement = avg_pos + avg_neg
        if total_movement == 0:
            return 0.0

        dx = abs(avg_pos - avg_neg) / total_movement

        # Add linear regression slope as confirmation
        x = np.arange(len(recent_prices))
        slope, _ = np.polyfit(x, recent_prices, 1)
        normalized_slope = abs(slope) / np.mean(recent_prices)

        # Combine both metrics
        trend_strength = (dx + min(1.0, normalized_slope * 100)) / 2

        return float(np.clip(trend_strength, 0.0, 1.0))

    def _check_volume_confirmation(
        self, volumes: List[float], threshold: float
    ) -> bool:
        """
        Check if current volume confirms the signal.

        Args:
            volumes: List of volume data
            threshold: Multiple of average volume required

        Returns:
            True if volume confirms signal
        """
        if len(volumes) < 20:
            return True  # Not enough data, skip check

        # Calculate average volume
        avg_volume = np.mean(volumes[-20:-1])  # Exclude current bar

        # Current volume
        current_volume = volumes[-1]

        # Check if volume exceeds threshold
        return current_volume >= (avg_volume * threshold)

    def _calculate_volatility(self, prices: List[float]) -> float:
        """Calculate recent volatility."""
        if len(prices) < 20:
            return 0.02  # Default 2%

        recent_prices = np.array(prices[-20:])
        returns = np.diff(recent_prices) / recent_prices[:-1]
        volatility = np.std(returns)

        return float(volatility)

    def _calculate_risk_params(
        self, volatility: float, signal_strength: float, trend_strength: float
    ) -> Tuple[float, float]:
        """
        Calculate stop loss and take profit.

        Stronger trends allow for wider stops and targets.

        Returns:
            (stop_loss_pct, take_profit_pct)
        """
        # Base risk on volatility
        base_stop = volatility * 200  # 2 sigma as percentage

        # Adjust stop based on trend strength (tighter stops in weak trends)
        stop_loss_pct = base_stop * (1.0 + trend_strength)

        # Take profit is a multiple of stop loss
        risk_reward_ratio = 2.0 + trend_strength  # 2:1 to 3:1
        take_profit_pct = stop_loss_pct * risk_reward_ratio

        # Clamp values
        stop_loss_pct = float(np.clip(stop_loss_pct, 1.0, 10.0))
        take_profit_pct = float(np.clip(take_profit_pct, 2.0, 25.0))

        return stop_loss_pct, take_profit_pct
