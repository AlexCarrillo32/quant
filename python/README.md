# Python Interface for Quant Engine

This directory contains Python bindings and examples for the Rust-based quant trading engine.

## The Human Edge Philosophy

**Why Python + Rust?**

- **Python**: Rapid strategy development, research, prototyping
- **Rust**: Ultra-low latency execution, type safety, performance

**Best of both worlds**: Write strategies in Python (fast iteration), execute in Rust (fast performance).

## Installation

### Building from source

```bash
# Install maturin (Python package builder for Rust)
pip install maturin

# Build and install the Python package
cd /path/to/quant
maturin develop --release
```

This will compile the Rust code and make `quant_engine` available in Python.

### Using the package

```python
import quant_engine as qe

# Create trading primitives
symbol = qe.Symbol("AAPL")
price = qe.Price(150.25)
quantity = qe.Quantity.buy(100)

# Create signals
signal = qe.Signal.new(
    symbol,
    qe.SignalAction.Buy,
    qe.Confidence(0.85),
    "Strong momentum",
    "MyAlpha"
)

print(signal)  # [MyAlpha] BUY AAPL (confidence: 85.0%) - Strong momentum
```

## Type Stubs

Type stubs are provided in `quant_engine.pyi` for IDE support and static type checking with mypy.

```bash
# Type checking with mypy
pip install mypy
mypy examples/simple_strategy.py
```

## Examples

### 1. Simple Strategy (`examples/simple_strategy.py`)

Demonstrates basic usage:
- Creating prices, quantities, symbols
- Building signals with risk management
- Type safety and validation

**Run:**
```bash
python examples/simple_strategy.py
```

### 2. Behavioral Strategy (`examples/behavioral_strategy.py`)

Demonstrates "The Human Edge":
- **Panic detection**: Buy when humans irrationally fear
- **FOMO detection**: Sell when humans irrationally FOMO
- **Narrative shifts**: React to story changes before consensus

**Run:**
```bash
python examples/behavioral_strategy.py
```

## API Reference

### Core Types

#### `Price`
Represents a price (always positive, finite).

```python
price = qe.Price(150.25)
price.value()  # 150.25
price.percent_change(qe.Price(165.0))  # 9.77

# Arithmetic
p1 = qe.Price(100.0)
p2 = qe.Price(50.0)
p1 + p2  # Price(150.0)
p1 - p2  # Price(50.0)
p1 * 1.5  # Price(150.0)
```

#### `Quantity`
Represents quantity (positive = buy, negative = sell).

```python
buy = qe.Quantity.buy(100)  # +100
buy.is_buy()  # True
buy.value()  # 100

sell = qe.Quantity.sell(50)  # -50
sell.is_sell()  # True
sell.value()  # -50
```

#### `Symbol`
Stock symbol (validated, auto-uppercased).

```python
symbol = qe.Symbol("aapl")  # Auto-uppercased to "AAPL"
symbol.as_str()  # "AAPL"
```

#### `Confidence`
Confidence score (0.0 to 1.0).

```python
conf = qe.Confidence(0.85)
conf.value()  # 0.85
conf.as_percent()  # 85.0
conf.is_high()  # True (> 0.75)
conf.is_medium()  # False
conf.is_low()  # False
```

#### `SignalAction`
Trading action enum.

```python
qe.SignalAction.Buy
qe.SignalAction.Sell
qe.SignalAction.Close
qe.SignalAction.Hold
```

#### `Signal`
Complete trading signal with metadata.

```python
signal = qe.Signal.new(
    qe.Symbol("AAPL"),
    qe.SignalAction.Buy,
    qe.Confidence(0.85),
    "Strong momentum breakout",
    "MyAlpha"
)

# Add risk management
signal.with_target_price(qe.Price(160.0))
signal.with_stop_loss(qe.Price(145.0))
signal.with_quantity(qe.Quantity.buy(100))

# Check actionability
signal.is_actionable()  # True (high confidence + not Hold)

# Calculate risk/reward
current = qe.Price(150.0)
signal.risk_reward_ratio(current)  # 2.0 (R:R ratio)
```

## Type Safety

All types enforce validation:

```python
# These will raise ValueError
qe.Price(-100.0)  # Negative price
qe.Price(float('nan'))  # NaN price
qe.Symbol("")  # Empty symbol
qe.Symbol("VERYLONGSYMBOL")  # Too long (> 10 chars)
qe.Quantity.new(0)  # Zero quantity
qe.Confidence(1.5)  # Out of range [0, 1]
```

## Performance

The Python bindings use PyO3 with minimal overhead:

- **Zero-copy** where possible (read-only operations)
- **Type conversion** only at API boundaries
- **Execution** happens in Rust (hot path stays fast)

**Performance targets:**
- Signal creation: ~100ns (Rust side)
- Python → Rust conversion: ~1μs
- Full signal with metadata: ~5μs

This means you can generate **100,000+ signals/second** even from Python.

## Writing Custom Strategies

### Strategy Pattern

```python
from typing import Optional
import quant_engine as qe

class MyStrategy:
    """Your custom strategy"""

    def __init__(self):
        self.param1 = 0.5
        self.param2 = 100

    def analyze(self, symbol: str, price: float, **kwargs) -> Optional[qe.Signal]:
        """
        Analyze market data and generate signal

        Returns:
            Signal if conditions are met, None otherwise
        """
        # Your logic here
        if self._should_buy(price):
            return qe.Signal.new(
                qe.Symbol(symbol),
                qe.SignalAction.Buy,
                qe.Confidence(0.8),
                "Your reason",
                "MyStrategy"
            )
        return None

    def _should_buy(self, price: float) -> bool:
        # Your conditions
        return price > self.param1
```

### Integration with Rust Engine

```python
# Python strategy
strategy = MyStrategy()

# Generate signals in Python
signals = []
for symbol in ["AAPL", "GOOGL", "MSFT"]:
    signal = strategy.analyze(symbol, get_price(symbol))
    if signal:
        signals.append(signal)

# Pass signals to Rust engine for execution
# (This API will be added in next phase)
# engine.execute_signals(signals)
```

## Next Steps

- [ ] Backtester Python API (Phase 5.2)
- [ ] Real-time data feed integration
- [ ] Portfolio management API
- [ ] Performance analytics API
- [ ] Jupyter notebook examples

## Questions?

See main project documentation in `/docs` directory.

---

**The Human Edge**: Speed (Rust) + Intelligence (Human Psychology) + Creativity (Python)
