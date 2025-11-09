"""
Base Alpha Model
Enhanced foundation for all alpha models with common functionality.
"""
from abc import ABC, abstractmethod
from dataclasses import dataclass
from datetime import datetime
from enum import Enum
from typing import Dict, List, Optional


class InsightDirection(Enum):
    """Direction of the trading insight."""
    UP = 1
    FLAT = 0
    DOWN = -1


class InsightType(Enum):
    """Type of insight signal."""
    PRICE = "price"
    VOLATILITY = "volatility"
    VOLUME = "volume"


@dataclass
class Insight:
    """
    Trading insight with enhanced metadata.

    Improvements over original Lean:
    - Confidence score (0-1)
    - Signal strength for position sizing
    - Expected holding period
    - Risk metrics
    """
    symbol: str
    direction: InsightDirection
    timestamp: datetime
    confidence: float  # 0.0 to 1.0
    signal_strength: float  # -1.0 to 1.0 (normalized)
    insight_type: InsightType = InsightType.PRICE
    expected_duration_minutes: Optional[int] = None
    stop_loss_pct: Optional[float] = None
    take_profit_pct: Optional[float] = None
    metadata: Optional[Dict] = None

    def __post_init__(self):
        """Validate insight parameters."""
        if not 0.0 <= self.confidence <= 1.0:
            raise ValueError("Confidence must be between 0 and 1")
        if not -1.0 <= self.signal_strength <= 1.0:
            raise ValueError("Signal strength must be between -1 and 1")


class BaseAlphaModel(ABC):
    """
    Enhanced base class for all alpha models.

    Improvements:
    - Abstract interface for consistency
    - Built-in risk management
    - Performance tracking
    - Adaptive parameter support
    """

    def __init__(self, name: str, lookback_period: int = 252):
        """
        Initialize base alpha model.

        Args:
            name: Unique identifier for this alpha model
            lookback_period: Historical data period for calculations
        """
        self.name = name
        self.lookback_period = lookback_period
        self.symbols: Dict[str, 'SymbolData'] = {}
        self.insights_generated = 0
        self.last_update: Optional[datetime] = None

    @abstractmethod
    def update(self, data: Dict, current_time: datetime) -> List[Insight]:
        """
        Generate trading insights based on current market data.

        Args:
            data: Market data dictionary {symbol: price_data}
            current_time: Current timestamp

        Returns:
            List of trading insights
        """
        pass

    @abstractmethod
    def on_securities_changed(self, added: List[str], removed: List[str]):
        """
        Handle changes to the universe of traded securities.

        Args:
            added: List of newly added symbols
            removed: List of removed symbols
        """
        pass

    def get_statistics(self) -> Dict:
        """
        Get performance statistics for this alpha model.

        Returns:
            Dictionary of performance metrics
        """
        return {
            "name": self.name,
            "insights_generated": self.insights_generated,
            "symbols_tracked": len(self.symbols),
            "last_update": self.last_update
        }

    def reset(self):
        """Reset the alpha model state."""
        self.symbols.clear()
        self.insights_generated = 0
        self.last_update = None


@dataclass
class SymbolData:
    """
    Enhanced symbol-specific data container.

    Improvements:
    - Consolidated indicator storage
    - State management
    - Historical tracking
    """
    symbol: str
    indicators: Dict = None
    state: Dict = None
    history: List = None

    def __post_init__(self):
        if self.indicators is None:
            self.indicators = {}
        if self.state is None:
            self.state = {}
        if self.history is None:
            self.history = []

    def update_indicator(self, name: str, value: float):
        """Update an indicator value."""
        self.indicators[name] = value

    def get_indicator(self, name: str) -> Optional[float]:
        """Get an indicator value."""
        return self.indicators.get(name)

    def set_state(self, key: str, value):
        """Set state variable."""
        self.state[key] = value

    def get_state(self, key: str):
        """Get state variable."""
        return self.state.get(key)
