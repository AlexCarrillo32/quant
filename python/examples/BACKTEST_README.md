# Backtesting Examples

This directory contains examples demonstrating how to use the Rust-powered backtesting engine from Python.

## Overview

The backtesting engine allows you to:
- Test trading strategies on historical data
- Validate signal quality before live trading
- Analyze risk-adjusted returns
- Understand strategy behavior across market conditions

## Examples

### 1. `backtest_example.py` - Basic Backtesting

**Purpose**: Learn the fundamentals of backtesting with manually created signals.

**What you'll learn**:
- Creating trading signals with entry prices, stops, and targets
- Running a backtest with commission and slippage
- Interpreting performance metrics (Sharpe, win rate, drawdown)
- Evaluating strategy viability

**Run it**:
```bash
cd python/examples
python backtest_example.py
```

**Expected output**:
```
BACKTEST RESULTS
Final Portfolio Value: $10,200.00
Total Return:          2.00%
Sharpe Ratio:          1.45
Win Rate:              75.00%
```

### 2. `backtest_with_alpha.py` - Custom Alpha Model

**Purpose**: Build and test a complete alpha model with signal generation.

**What you'll learn**:
- Creating custom alpha models in Python
- Generating signals from price data (simulated)
- Advanced performance analysis
- Understanding risk-adjusted returns
- Human psychology insights on backtesting vs. live trading

**Run it**:
```bash
cd python/examples
python backtest_with_alpha.py
```

**Features**:
- SimpleMomentumAlpha model (demonstrates momentum trading)
- Monte Carlo price simulation
- Comprehensive performance evaluation
- Human psychology reminders

## Backtest Configuration

### Basic Parameters

```python
backtester = qe.Backtester(
    initial_capital=10_000.0,  # Starting capital
    commission=1.0,            # $ per trade
    slippage=0.05              # % slippage (5 bps)
)
```

### Signal Requirements

Signals must have:
- `symbol`: Stock symbol (e.g., "SPY")
- `action`: Buy, Sell, Close, or Hold
- `confidence`: 0.0 to 1.0
- `target_price`: Entry/exit price for the trade

Optional fields:
- `stop_loss`: Stop loss price
- `take_profit`: Take profit price
- `quantity`: Suggested position size

## Performance Metrics Explained

### Returns
- **Total Return**: Overall portfolio gain/loss
- **Annualized Return**: Geometric mean return per year

### Risk-Adjusted Returns
- **Sharpe Ratio**: Risk-adjusted return (return per unit of volatility)
  - `> 2.0`: Excellent
  - `1.0 - 2.0`: Good
  - `< 1.0`: Poor

- **Sortino Ratio**: Like Sharpe but only penalizes downside volatility
  - Higher is better
  - More relevant for asymmetric strategies

### Risk Metrics
- **Max Drawdown**: Largest peak-to-trough decline
  - `< 10%`: Low risk
  - `10% - 20%`: Moderate risk
  - `> 20%`: High risk

- **Volatility**: Standard deviation of returns
  - Lower is more stable

### Trading Metrics
- **Win Rate**: % of profitable trades
  - `> 60%`: Strong
  - `50% - 60%`: Good
  - `< 50%`: Needs work

- **Profit Factor**: Gross profits / Gross losses
  - `> 2.0`: Excellent
  - `1.5 - 2.0`: Good
  - `< 1.0`: Losing strategy

- **Average Win/Loss**: Mean P&L per winning/losing trade

## Best Practices

### 1. Signal Quality
```python
# ✅ GOOD - Include stop loss and target
signal.target_price = qe.Price(100.0)
signal.stop_loss = qe.Price(98.0)      # 2% risk
signal.take_profit = qe.Price(104.0)   # 4% reward (2:1 R:R)

# ❌ BAD - No risk management
signal.target_price = qe.Price(100.0)
# Missing stops!
```

### 2. Realistic Costs
```python
# ✅ GOOD - Real-world costs
backtester = qe.Backtester(
    commission=0.5,   # Interactive Brokers: ~$0.50/trade
    slippage=0.03     # 3 bps for liquid ETFs
)

# ❌ BAD - Unrealistic (ignoring costs)
backtester = qe.Backtester(
    commission=0.0,
    slippage=0.0
)
```

### 3. Walk-Forward Testing
```python
# ✅ GOOD - Train and test on different periods
train_signals = alpha.generate_signals(prices[:200])  # First 200 days
test_signals = alpha.generate_signals(prices[200:])   # Next period

# ❌ BAD - Testing on same data used for optimization
all_signals = alpha.generate_signals(prices)  # Overfitting risk!
```

### 4. Expectation Management
Backtest results are **always better** than live trading:
- Live trading has emotional stress (fear, greed, panic)
- Unexpected costs and slippage
- Execution delays
- Black swan events

**Rule of thumb**: Expect 20-30% worse performance in live trading.

## Integration with Rust Engine

These examples use the Rust backtesting engine via Python bindings (PyO3).

**Benefits**:
- 🚀 **Fast**: Rust speed for backtesting
- 🔒 **Safe**: Risk management enforced at Rust level
- 🧠 **Smart**: Python flexibility for strategy development

**How it works**:
1. You write alpha models in Python
2. Generate signals from historical data
3. Rust engine executes trades with risk checks
4. Results returned to Python for analysis

## Next Steps

After backtesting:

1. **Out-of-Sample Testing**: Test on different symbols, time periods
2. **Stress Testing**: 2008 crash, COVID crash, flash crashes
3. **Monte Carlo**: Randomize entry times to test robustness
4. **Paper Trading**: Forward test with live market data
5. **Small Capital Live**: Start with $1-5K to validate

## Human Psychology Insights

### Why Backtests Mislead

1. **Hindsight Bias**: You know what happened
   - Live: Uncertainty and fear dominate

2. **No Emotional Cost**: Backtests ignore psychology
   - Live: 20% drawdown feels like the end of the world

3. **Perfect Execution**: Backtests assume instant fills
   - Live: Slippage, delays, partial fills

4. **Survivorship Bias**: Backtests on existing symbols
   - Live: Companies go bankrupt, symbols delist

### The Human Edge

Banks have:
- Speed (colocation)
- Capital (billions)
- Data (Bloomberg terminals)

You have:
- **Psychology**: Model human irrationality
- **Creativity**: Build strategies banks won't
- **Agility**: Adapt faster than institutions

Focus on strategies that exploit human behavior, not just math.

## Questions?

See the main project README or check the docs in `/docs`.
