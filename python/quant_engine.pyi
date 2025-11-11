"""
Type stubs for quant_engine Python bindings

This file provides type hints for IDE support and static type checking.
"""

from typing import Optional

__version__: str

class Price:
    """Price in dollars (always positive)"""

    def __init__(self, value: float) -> None:
        """
        Create a new price

        Args:
            value: Price value (must be positive)

        Raises:
            ValueError: If price is negative, zero, NaN, or infinite
        """
        ...

    def value(self) -> float:
        """Get the raw price value"""
        ...

    def percent_change(self, other: Price) -> float:
        """
        Calculate percentage change to another price

        Args:
            other: Target price

        Returns:
            Percentage change
        """
        ...

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __lt__(self, other: Price) -> bool: ...
    def __le__(self, other: Price) -> bool: ...
    def __eq__(self, other: object) -> bool: ...
    def __ne__(self, other: object) -> bool: ...
    def __gt__(self, other: Price) -> bool: ...
    def __ge__(self, other: Price) -> bool: ...
    def __add__(self, other: Price) -> Price: ...
    def __sub__(self, other: Price) -> Price: ...
    def __mul__(self, scalar: float) -> Price: ...
    def __truediv__(self, scalar: float) -> Price: ...


class Quantity:
    """Quantity (positive = buy, negative = sell)"""

    def __init__(self, value: int) -> None:
        """
        Create a quantity from a signed integer

        Args:
            value: Quantity (positive = buy, negative = sell)

        Raises:
            ValueError: If quantity is zero
        """
        ...

    @staticmethod
    def buy(value: int) -> Quantity:
        """
        Create a buy quantity

        Args:
            value: Number of shares to buy
        """
        ...

    @staticmethod
    def sell(value: int) -> Quantity:
        """
        Create a sell quantity

        Args:
            value: Number of shares to sell
        """
        ...

    def value(self) -> int:
        """Get the signed value"""
        ...

    def abs(self) -> int:
        """Get the absolute value"""
        ...

    def is_buy(self) -> bool:
        """Check if this is a buy order"""
        ...

    def is_sell(self) -> bool:
        """Check if this is a sell order"""
        ...

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...


class Symbol:
    """Stock symbol (e.g., "AAPL", "GOOGL")"""

    def __init__(self, value: str) -> None:
        """
        Create a new symbol

        Args:
            value: Symbol string (e.g., "AAPL")

        Raises:
            ValueError: If symbol is invalid
        """
        ...

    def as_str(self) -> str:
        """Get the symbol as a string"""
        ...

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __hash__(self) -> int: ...
    def __eq__(self, other: object) -> bool: ...


class SignalAction:
    """Trading action"""

    Buy: SignalAction
    Sell: SignalAction
    Close: SignalAction
    Hold: SignalAction

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...


class Confidence:
    """Confidence score (0.0 to 1.0)"""

    def __init__(self, value: float) -> None:
        """
        Create a new confidence score

        Args:
            value: Confidence value between 0.0 and 1.0

        Raises:
            ValueError: If value is not in [0.0, 1.0] range
        """
        ...

    def value(self) -> float:
        """Get the raw confidence value"""
        ...

    def as_percent(self) -> float:
        """Get confidence as percentage (0-100)"""
        ...

    def is_low(self) -> bool:
        """Check if confidence is low (< 0.5)"""
        ...

    def is_medium(self) -> bool:
        """Check if confidence is medium (0.5 - 0.75)"""
        ...

    def is_high(self) -> bool:
        """Check if confidence is high (> 0.75)"""
        ...

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...


class Signal:
    """Trading signal with metadata"""

    @staticmethod
    def new(
        symbol: Symbol,
        action: SignalAction,
        confidence: Confidence,
        reason: str,
        source: str,
    ) -> Signal:
        """
        Create a new signal

        Args:
            symbol: Symbol to trade
            action: Action to take
            confidence: Signal confidence
            reason: Human-readable reason
            source: Alpha model name

        Returns:
            New signal instance
        """
        ...

    @property
    def symbol(self) -> Symbol:
        """Get the symbol"""
        ...

    @property
    def action(self) -> SignalAction:
        """Get the action"""
        ...

    @property
    def confidence(self) -> Confidence:
        """Get the confidence"""
        ...

    @property
    def reason(self) -> str:
        """Get the reason"""
        ...

    @property
    def source(self) -> str:
        """Get the source"""
        ...

    @property
    def target_price(self) -> Optional[Price]:
        """Get target price (if set)"""
        ...

    @property
    def stop_loss(self) -> Optional[Price]:
        """Get stop loss (if set)"""
        ...

    @property
    def take_profit(self) -> Optional[Price]:
        """Get take profit (if set)"""
        ...

    @property
    def quantity(self) -> Optional[Quantity]:
        """Get quantity (if set)"""
        ...

    def with_target_price(self, price: Price) -> None:
        """Set target price"""
        ...

    def with_stop_loss(self, price: Price) -> None:
        """Set stop loss"""
        ...

    def with_take_profit(self, price: Price) -> None:
        """Set take profit"""
        ...

    def with_quantity(self, quantity: Quantity) -> None:
        """Set quantity"""
        ...

    def is_actionable(self) -> bool:
        """Check if signal is actionable (high confidence, not hold)"""
        ...

    def risk_reward_ratio(self, current_price: Price) -> Optional[float]:
        """
        Calculate risk/reward ratio

        Args:
            current_price: Current market price

        Returns:
            Risk/reward ratio if prices are set, None otherwise
        """
        ...

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
