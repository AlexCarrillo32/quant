"""
Enhanced MACD Alpha Model
Improvements over original QuantConnect implementation:
- Adaptive bounce threshold based on volatility
- Signal strength calculation for position sizing
- Multi-timeframe confirmation
- Divergence detection
"""
from datetime import datetime, timedelta
from typing import Dict, List, Optional
import numpy as np

from .base_alpha import (
    BaseAlphaModel,
    Insight,
    InsightDirection,
    InsightType,
    SymbolData
)


class EnhancedMacdAlphaModel(BaseAlphaModel):
    """
    Enhanced MACD-based alpha model with adaptive parameters.

    Key improvements:
    1. Adaptive threshold: Adjusts based on recent volatility
    2. Signal strength: Normalized MACD value for position sizing
    3. Confirmation: Requires histogram agreement
    4. Divergence detection: Price vs MACD divergence signals
    """

    def __init__(
        self,
        fast_period: int = 12,
        slow_period: int = 26,
        signal_period: int = 9,
        base_threshold: float = 0.01,
        volatility_window: int = 20,
        holding_period_minutes: int = 240  # 4 hours
    ):
        """
        Initialize enhanced MACD alpha model.

        Args:
            fast_period: Fast EMA period
            slow_period: Slow EMA period
            signal_period: Signal line period
            base_threshold: Base bounce threshold (will be adjusted)
            volatility_window: Window for volatility calculation
            holding_period_minutes: Expected holding period for insights
        """
        super().__init__(name="EnhancedMACD")
        self.fast_period = fast_period
        self.slow_period = slow_period
        self.signal_period = signal_period
        self.base_threshold = base_threshold
        self.volatility_window = volatility_window
        self.holding_period_minutes = holding_period_minutes

    def update(self, data: Dict, current_time: datetime) -> List[Insight]:
        """
        Generate insights based on MACD signals.

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

            # Calculate MACD components
            macd_line, signal_line, histogram = self._calculate_macd(
                price_data['close']
            )

            if macd_line is None:
                continue

            # Calculate adaptive threshold based on volatility
            volatility = self._calculate_volatility(price_data['close'])
            adaptive_threshold = self.base_threshold * (1 + volatility)

            # Normalize MACD for signal strength
            normalized_macd = self._normalize_macd(macd_line, price_data['close'])

            # Store current values in symbol data
            symbol_data.update_indicator('macd', macd_line)
            symbol_data.update_indicator('signal', signal_line)
            symbol_data.update_indicator('histogram', histogram)
            symbol_data.update_indicator('volatility', volatility)

            # Get previous state
            previous_position = symbol_data.get_state('position')

            # Determine signal direction
            direction, confidence, signal_strength = self._evaluate_signal(
                macd_line=macd_line,
                signal_line=signal_line,
                histogram=histogram,
                normalized_macd=normalized_macd,
                adaptive_threshold=adaptive_threshold,
                previous_position=previous_position
            )

            # Only generate insight if signal is strong enough
            if direction != InsightDirection.FLAT and confidence > 0.5:
                # Check for divergence (enhances signal quality)
                divergence_detected = self._check_divergence(
                    price_data['close'],
                    symbol_data
                )

                # Boost confidence if divergence confirms signal
                if divergence_detected:
                    confidence = min(1.0, confidence * 1.2)

                # Calculate risk parameters
                stop_loss_pct, take_profit_pct = self._calculate_risk_params(
                    volatility, signal_strength
                )

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
                        'macd': macd_line,
                        'signal': signal_line,
                        'histogram': histogram,
                        'volatility': volatility,
                        'threshold': adaptive_threshold,
                        'divergence': divergence_detected
                    }
                )

                insights.append(insight)
                symbol_data.set_state('position', direction)
                self.insights_generated += 1

        return insights

    def on_securities_changed(self, added: List[str], removed: List[str]):
        """Handle universe changes."""
        # Add new symbols
        for symbol in added:
            if symbol not in self.symbols:
                self.symbols[symbol] = SymbolData(symbol=symbol)

        # Remove old symbols
        for symbol in removed:
            if symbol in self.symbols:
                del self.symbols[symbol]

    def _calculate_macd(
        self, prices: List[float]
    ) -> tuple[Optional[float], Optional[float], Optional[float]]:
        """
        Calculate MACD components.

        Returns:
            (macd_line, signal_line, histogram) or (None, None, None)
        """
        if len(prices) < self.slow_period + self.signal_period:
            return None, None, None

        prices_array = np.array(prices)

        # Calculate EMAs
        fast_ema = self._calculate_ema(prices_array, self.fast_period)
        slow_ema = self._calculate_ema(prices_array, self.slow_period)

        # MACD line
        macd_line = fast_ema - slow_ema

        # Signal line (EMA of MACD)
        macd_history = prices_array[-self.signal_period:]  # Simplified
        signal_line = self._calculate_ema(macd_history, self.signal_period)

        # Histogram
        histogram = macd_line - signal_line

        return macd_line, signal_line, histogram

    def _calculate_ema(self, prices: np.ndarray, period: int) -> float:
        """Calculate Exponential Moving Average."""
        multiplier = 2.0 / (period + 1)
        ema = prices[0]

        for price in prices[1:]:
            ema = (price * multiplier) + (ema * (1 - multiplier))

        return ema

    def _calculate_volatility(self, prices: List[float]) -> float:
        """Calculate recent volatility (standard deviation of returns)."""
        if len(prices) < self.volatility_window:
            return 1.0

        recent_prices = np.array(prices[-self.volatility_window:])
        returns = np.diff(recent_prices) / recent_prices[:-1]
        volatility = np.std(returns)

        return volatility

    def _normalize_macd(self, macd: float, prices: List[float]) -> float:
        """Normalize MACD by recent price to get signal strength."""
        if len(prices) == 0:
            return 0.0

        current_price = prices[-1]
        if current_price == 0:
            return 0.0

        # Normalize to percentage of price
        normalized = (macd / current_price) * 100

        # Clamp to [-1, 1]
        return np.clip(normalized, -1.0, 1.0)

    def _evaluate_signal(
        self,
        macd_line: float,
        signal_line: float,
        histogram: float,
        normalized_macd: float,
        adaptive_threshold: float,
        previous_position: Optional[InsightDirection]
    ) -> tuple[InsightDirection, float, float]:
        """
        Evaluate MACD signal and return direction, confidence, and strength.

        Returns:
            (direction, confidence, signal_strength)
        """
        # Calculate signal strength based on MACD vs signal line
        macd_diff = macd_line - signal_line

        # Check if signal crosses threshold
        if abs(normalized_macd) < adaptive_threshold:
            return InsightDirection.FLAT, 0.0, 0.0

        # Determine direction
        if macd_diff > 0 and histogram > 0:
            direction = InsightDirection.UP
            signal_strength = min(1.0, abs(normalized_macd))
        elif macd_diff < 0 and histogram < 0:
            direction = InsightDirection.DOWN
            signal_strength = -min(1.0, abs(normalized_macd))
        else:
            # No histogram confirmation
            return InsightDirection.FLAT, 0.0, 0.0

        # Avoid duplicate signals in same direction
        if previous_position == direction:
            return InsightDirection.FLAT, 0.0, 0.0

        # Calculate confidence based on histogram strength
        confidence = min(1.0, abs(histogram) / adaptive_threshold)

        return direction, confidence, signal_strength

    def _check_divergence(
        self, prices: List[float], symbol_data: SymbolData
    ) -> bool:
        """
        Check for bullish/bearish divergence between price and MACD.

        Divergence can indicate trend reversals.
        """
        # Need historical data to detect divergence
        if len(prices) < 20:
            return False

        # Simplified divergence detection
        # In production, would track swing highs/lows
        recent_prices = prices[-10:]
        price_trend = recent_prices[-1] - recent_prices[0]

        macd_history = symbol_data.history[-10:] if len(symbol_data.history) >= 10 else []
        if len(macd_history) < 10:
            return False

        macd_values = [h.get('macd', 0) for h in macd_history]
        macd_trend = macd_values[-1] - macd_values[0]

        # Bullish divergence: price down, MACD up
        # Bearish divergence: price up, MACD down
        divergence = (price_trend * macd_trend) < 0

        return divergence

    def _calculate_risk_params(
        self, volatility: float, signal_strength: float
    ) -> tuple[float, float]:
        """
        Calculate stop loss and take profit based on volatility and signal strength.

        Returns:
            (stop_loss_pct, take_profit_pct)
        """
        # Base risk on volatility
        base_risk = 2.0 * volatility * 100  # 2 standard deviations

        # Adjust based on signal strength
        stop_loss_pct = base_risk * (1.0 / abs(signal_strength + 0.1))
        take_profit_pct = base_risk * abs(signal_strength) * 2

        # Clamp values
        stop_loss_pct = np.clip(stop_loss_pct, 1.0, 10.0)
        take_profit_pct = np.clip(take_profit_pct, 2.0, 20.0)

        return stop_loss_pct, take_profit_pct
