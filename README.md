# Quant - Enhanced Algorithmic Trading Framework

An improved version of QuantConnect's Lean Algorithm Framework with enhanced alpha models, adaptive parameters, and comprehensive risk management.

## Project Structure

```
quant/
├── alphas/              # Enhanced alpha models
├── indicators/          # Technical indicators
├── risk/               # Risk management modules
├── backtesting/        # Backtesting framework
├── utils/              # Utility functions
├── tests/              # Test suite
├── examples/           # Usage examples
└── docs/               # Documentation
```

## Improvements Over Original Lean Framework

1. **Adaptive Parameters**: Dynamic threshold adjustments based on market volatility
2. **Multi-Signal Confirmation**: Combine multiple indicators to reduce false signals
3. **Advanced Risk Management**: Position sizing, stop-loss, and portfolio optimization
4. **Better Backtesting**: Enhanced performance metrics and visualization
5. **Modern Python**: Type hints, async support, and clean architecture

## Getting Started

```bash
# Install dependencies
pip install -r requirements.txt

# Run example strategy
python examples/macd_ema_strategy.py
```

## License

Apache License 2.0 (maintaining compatibility with QuantConnect Lean)
