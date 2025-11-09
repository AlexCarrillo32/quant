# Quick Start Guide

Get up and running with the enhanced alpha models in 5 minutes.

## Installation

### 1. Clone or download this repository

```bash
cd /Users/alex.carrillo/Desktop/Projects/quant
```

### 2. Install dependencies

```bash
pip install -r requirements.txt
```

**Note:** Some dependencies like `ta-lib` require system-level installation:

**macOS:**
```bash
brew install ta-lib
pip install ta-lib
```

**Linux:**
```bash
wget http://prdownloads.sourceforge.net/ta-lib/ta-lib-0.4.0-src.tar.gz
tar -xzf ta-lib-0.4.0-src.tar.gz
cd ta-lib/
./configure --prefix=/usr
make
sudo make install
pip install ta-lib
```

**Windows:**
Download from: https://www.lfd.uci.edu/~gohlke/pythonlibs/#ta-lib

## Running Your First Strategy

### Example 1: Simple MACD Strategy

```python
from alphas import EnhancedMacdAlphaModel
from datetime import datetime

# Initialize the model
macd = EnhancedMacdAlphaModel(
    fast_period=12,
    slow_period=26,
    signal_period=9
)

# Add your symbols
symbols = ['AAPL', 'GOOGL', 'MSFT']
macd.on_securities_changed(added=symbols, removed=[])

# Prepare market data (replace with real data)
market_data = {
    'AAPL': {
        'close': [150, 151, 149, 152, 153, ...],  # Price history
        'volume': [1000000, 1100000, ...]          # Volume history
    },
    # ... other symbols
}

# Generate insights
insights = macd.update(market_data, datetime.now())

# Process insights
for insight in insights:
    print(f"{insight.symbol}: {insight.direction.name}")
    print(f"  Confidence: {insight.confidence:.2%}")
    print(f"  Signal Strength: {insight.signal_strength:.3f}")
    print(f"  Stop Loss: {insight.stop_loss_pct:.2f}%")
    print(f"  Take Profit: {insight.take_profit_pct:.2f}%")
```

### Example 2: Run the Included Demo

```bash
cd examples
python basic_strategy_example.py
```

This will:
1. Generate sample market data
2. Run both MACD and EMA Cross strategies
3. Display generated insights with detailed metrics
4. Compare the two strategies

## Understanding the Output

### Insight Object

Each insight contains:

| Field | Description | Example |
|-------|-------------|---------|
| `symbol` | Ticker symbol | "AAPL" |
| `direction` | Trade direction | UP, DOWN, FLAT |
| `confidence` | Signal quality (0-1) | 0.75 (75%) |
| `signal_strength` | Normalized strength (-1 to 1) | 0.65 |
| `stop_loss_pct` | Suggested stop loss | 2.5% |
| `take_profit_pct` | Suggested take profit | 5.0% |
| `metadata` | Additional info | MACD values, volatility, etc. |

### Example Output

```
Date: 2025-11-06
  📈 AAPL
     Direction: UP
     Confidence: 78.50%
     Signal Strength: 0.654
     Stop Loss: 2.34%
     Take Profit: 5.87%
     MACD: 1.234
     Volatility: 0.0156
```

## Getting Real Market Data

### Option 1: Yahoo Finance (Free)

```python
import yfinance as yf

# Download data
ticker = yf.Ticker("AAPL")
hist = ticker.history(period="1y")

market_data = {
    'AAPL': {
        'close': hist['Close'].tolist(),
        'volume': hist['Volume'].tolist()
    }
}
```

### Option 2: Alpaca (Free for testing)

```python
from alpaca.data import StockHistoricalDataClient
from alpaca.data.requests import StockBarsRequest
from alpaca.data.timeframe import TimeFrame
from datetime import datetime

# Initialize client
client = StockHistoricalDataClient(api_key, secret_key)

# Get data
request_params = StockBarsRequest(
    symbol_or_symbols=["AAPL"],
    timeframe=TimeFrame.Day,
    start=datetime(2024, 1, 1),
    end=datetime(2025, 11, 6)
)

bars = client.get_stock_bars(request_params)

market_data = {
    'AAPL': {
        'close': [bar.close for bar in bars['AAPL']],
        'volume': [bar.volume for bar in bars['AAPL']]
    }
}
```

### Option 3: Interactive Brokers

```python
from ib_insync import IB, Stock

ib = IB()
ib.connect('127.0.0.1', 7497, clientId=1)

# Request data
contract = Stock('AAPL', 'SMART', 'USD')
bars = ib.reqHistoricalData(
    contract,
    endDateTime='',
    durationStr='1 Y',
    barSizeSetting='1 day',
    whatToShow='TRADES',
    useRTH=True
)

market_data = {
    'AAPL': {
        'close': [bar.close for bar in bars],
        'volume': [bar.volume for bar in bars]
    }
}
```

## Common Workflows

### 1. Backtesting a Strategy

```python
from alphas import EnhancedMacdAlphaModel
import yfinance as yf

# Get historical data
ticker = yf.Ticker("AAPL")
hist = ticker.history(period="2y")

# Initialize strategy
strategy = EnhancedMacdAlphaModel()
strategy.on_securities_changed(added=['AAPL'], removed=[])

# Simulate day by day
for i in range(100, len(hist)):
    data = {
        'AAPL': {
            'close': hist['Close'][:i].tolist(),
            'volume': hist['Volume'][:i].tolist()
        }
    }

    insights = strategy.update(data, hist.index[i])

    # Track insights for performance analysis
    if insights:
        print(f"{hist.index[i]}: {len(insights)} insights")
```

### 2. Live Trading (Paper Trading)

```python
import time
from alphas import EnhancedEmaCrossAlphaModel
from alpaca.trading.client import TradingClient

# Initialize
strategy = EnhancedEmaCrossAlphaModel()
trading_client = TradingClient(api_key, secret_key, paper=True)

symbols = ['AAPL', 'GOOGL', 'MSFT']
strategy.on_securities_changed(added=symbols, removed=[])

# Main loop
while True:
    # Fetch latest data
    market_data = fetch_current_data(symbols)

    # Generate insights
    insights = strategy.update(market_data, datetime.now())

    # Execute trades
    for insight in insights:
        if insight.confidence > 0.7:  # Only trade high confidence
            execute_trade(trading_client, insight)

    # Wait for next bar
    time.sleep(60)  # 1 minute
```

### 3. Parameter Optimization

```python
from itertools import product

# Define parameter ranges
fast_periods = [8, 10, 12, 14]
slow_periods = [24, 26, 28, 30]
thresholds = [0.005, 0.01, 0.015]

best_params = None
best_score = 0

# Grid search
for fast, slow, threshold in product(fast_periods, slow_periods, thresholds):
    model = EnhancedMacdAlphaModel(
        fast_period=fast,
        slow_period=slow,
        base_threshold=threshold
    )

    # Backtest with these parameters
    score = backtest(model, historical_data)

    if score > best_score:
        best_score = score
        best_params = (fast, slow, threshold)

print(f"Best parameters: {best_params}")
print(f"Best score: {best_score}")
```

## Next Steps

1. **Read the improvements document**: [IMPROVEMENTS.md](./IMPROVEMENTS.md)
2. **Understand the architecture**: Explore the `alphas/` directory
3. **Customize parameters**: Tune for your trading style
4. **Add risk management**: Implement position sizing
5. **Backtest thoroughly**: Test on historical data before live trading

## Troubleshooting

### "Module not found" errors

```bash
# Make sure you're in the correct directory
cd /Users/alex.carrillo/Desktop/Projects/quant

# Check Python path
export PYTHONPATH="${PYTHONPATH}:$(pwd)"
```

### TA-Lib installation issues

If you don't want to use TA-Lib, you can use pandas-ta instead:

```python
# Replace ta-lib with pandas-ta in requirements.txt
# pandas-ta>=0.3.14b

import pandas_ta as ta
df['MACD'] = ta.macd(df['close'])
```

### Performance issues with large datasets

```python
# Use vectorized operations
import numpy as np

# Instead of loops
prices_array = np.array(prices)
ema = calculate_ema_vectorized(prices_array)
```

## Support

- **Issues**: Open an issue on GitHub
- **Documentation**: See `docs/` directory
- **Examples**: See `examples/` directory

## Resources

- [QuantConnect Documentation](https://www.quantconnect.com/docs)
- [Algorithmic Trading Best Practices](https://www.quantstart.com/)
- [Python for Finance](https://www.oreilly.com/library/view/python-for-finance/9781492024323/)
