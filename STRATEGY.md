# Trading Strategy: High-Frequency Small Profits

## Core Philosophy

**Make small profits many times = Lots of money long term**

### The Math

```
1% profit × 100 trades = 2.7x returns (compounded)
0.5% profit × 200 trades = 2.7x returns (compounded)

Traditional investors: 10% yearly
Our strategy: 1% weekly × 52 weeks = 67% yearly (1.01^52)
```

### Why This Works

1. **High Win Rate**: Small profit targets are easier to hit (70-80% vs 50%)
2. **Lower Risk**: Small moves mean tighter stop losses (0.5% risk vs 5%)
3. **More Opportunities**: 100 small opportunities > 10 big ones
4. **Compound Interest**: Small gains compound fast

---

## Implementation Strategy

### Position Sizing
- **Target profit per trade**: 0.5% - 2.0%
- **Maximum risk per trade**: 0.3% - 0.5% of portfolio
- **Hold time**: Minutes to hours, not days
- **Win rate target**: 70%+

### Signal Requirements
- **Minimum confidence**: 70% (0.7)
- **Risk/Reward ratio**: Minimum 2:1 (0.5% risk → 1.0% profit)
- **Multiple confirmations**: At least 2 alphas agree

### Entry Criteria
```rust
// Good trade example:
Signal {
    confidence: 0.75,           // 75% confident
    target_profit: 1.5%,        // Small, achievable
    stop_loss: 0.5%,            // Tight risk control
    risk_reward: 3.0,           // 3:1 ratio
}
```

### Exit Strategy
1. **Take profit fast**: Hit 1-2% target → close immediately
2. **Stop loss tight**: 0.5% loss → exit immediately
3. **No hoping**: If signal invalidates → exit
4. **Time stop**: If no movement in 1 hour → exit

---

## Engine Configuration for This Strategy

### Update Frequency
```rust
EngineConfig {
    update_interval: Duration::from_secs(60),  // Check every 60 seconds
    // Fast enough to catch opportunities
    // Slow enough to avoid noise
}
```

### Confidence Thresholds
```rust
min_confidence: 0.7,  // 70% minimum (allows more trades)
```

### Aggregation Strategy
```rust
// Use WeightedAverage or MajorityVote
// Requires multiple alphas to agree
aggregation_strategy: AggregationStrategy::WeightedAverage,
```

### Risk Management
```rust
max_positions: 5-10,           // Diversify across multiple small trades
position_size: 2-5% per trade, // Never risk more than 5% on one trade
```

---

## Alpha Models Optimized for Small Profits

### 1. Panic Detector (IMPLEMENTED ✅)
- **Profit target**: 1-3% bounce from panic bottom
- **Hold time**: 1-4 hours
- **Human insight**: Humans overreact → prices overshoot → quick recovery

### 2. Narrative Shift (TODO)
- **Profit target**: 0.5-2% from early detection
- **Hold time**: Minutes to hours
- **Human insight**: Get in before the crowd realizes story changed

### 3. Crowd Behavior (TODO)
- **Profit target**: 1-2% fade against retail traders
- **Hold time**: 30 minutes - 2 hours
- **Human insight**: When WSB is all-in, fade them

### 4. Structural Inefficiency (TODO)
- **Profit target**: 0.3-1% from predictable institutional moves
- **Hold time**: Minutes
- **Human insight**: Index rebalancing creates predictable price moves

### 5. Creative Synthesis (TODO)
- **Profit target**: 2-5% from non-obvious correlations
- **Hold time**: Hours to days
- **Human insight**: Weather → e-commerce, oil → airlines

---

## Performance Metrics

### Success Criteria
- **Win rate**: >70%
- **Average profit**: 1.0% - 1.5%
- **Average loss**: 0.3% - 0.5%
- **Profit factor**: >2.0 (total profit / total loss)
- **Trades per day**: 5-20

### Example Month (Ideal)
```
100 trades:
- 75 wins × 1.2% avg = +90%
- 25 losses × -0.4% avg = -10%
- Net: +80% (monthly)
- Compounded: 2.2x in 1 month

Realistic (with slippage, fees):
- Net: +20-30% monthly
- Compounded: 1.5x - 2.0x in 3 months
```

---

## Risk Management Rules

### Position Limits (MUST)
1. **Never risk >0.5% per trade** (portfolio preservation)
2. **Max 10 positions** (don't spread too thin)
3. **Max 3 positions per symbol** (avoid concentration)
4. **Stop trading after 3 consecutive losses** (avoid tilt)

### Stop Loss Rules (MUST)
1. **Always set stop loss** before entry
2. **Never move stop loss further** (only tighten)
3. **Honor the stop** (no "just one more minute")

### Profit Target Rules (SHOULD)
1. **Take profits at 1-2%** (don't get greedy)
2. **Trail stop once +0.5%** (protect profits)
3. **Exit if signal invalidates** (even if profitable)

---

## Engine Features Needed

### Phase 2 (COMPLETED ✅)
- ✅ Trading engine event loop
- ✅ Signal aggregator
- ✅ Order manager with position tracking
- ✅ Automatic stop loss / take profit
- ✅ Position sizing calculator (risk-based)
- ✅ Risk manager (prevent over-exposure)
- ✅ Drawdown protection (stop trading if -5% day)

### Phase 3 (Current)
- ⏳ Performance tracker (win rate, profit factor)
- Full integration testing
- Backtesting framework

### Phase 4 (Future)
- Backtesting with realistic slippage
- Paper trading mode
- Live trading with real broker API

---

## Why This Beats Banks

**Banks can't do this because:**
1. **Size**: They trade billions, can't scalp 1%
2. **Compliance**: Can't trade fast enough (regulations)
3. **Psychology**: Institutional mindset = long-term only
4. **Human insight**: They model math, not human behavior

**We can do this because:**
1. **Small**: Can trade $1K-$100K positions
2. **Fast**: No compliance delays
3. **Behavioral**: Model humans, not just math
4. **Creative**: Banks won't build "weather → e-commerce" models

---

## Next Steps

1. ✅ **Add position manager** (track open positions, P&L) - DONE
2. ✅ **Add risk manager** (enforce 0.5% max risk per trade) - DONE
3. ⏳ **Add performance tracker** (calculate win rate, profit factor) - IN PROGRESS
4. **Implement remaining 4 alphas** (more opportunities)
5. **Backtest on 1 year of data** (validate 70%+ win rate)

---

**Remember**: A thousand small cuts can fell a giant. We're the thousand cuts.
