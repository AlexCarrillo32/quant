"""
Enhanced Alpha Models Package

Improved versions of QuantConnect's Lean alpha models with:
- Adaptive parameters based on market conditions
- Multi-signal confirmation
- Risk management integration
- Better signal quality filtering
"""

from .base_alpha import (
    BaseAlphaModel,
    Insight,
    InsightDirection,
    InsightType,
    SymbolData
)
from .enhanced_macd_alpha import EnhancedMacdAlphaModel
from .enhanced_ema_cross_alpha import EnhancedEmaCrossAlphaModel

__all__ = [
    'BaseAlphaModel',
    'Insight',
    'InsightDirection',
    'InsightType',
    'SymbolData',
    'EnhancedMacdAlphaModel',
    'EnhancedEmaCrossAlphaModel',
]

__version__ = '1.0.0'
