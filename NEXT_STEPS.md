# Next Steps - Prioritized Action Plan

**Current Status:** Phase 4 Complete (80% done)
**Grade:** A- (Excellent but incomplete)

---

## 🔥 High Priority (Do These First)

### 1. Complete Remaining Alpha Models (⏱️ 2-3 days)

**Why:** Unlock the full "Human Edge" advantage

**Current Status:**
- ✅ Panic Detector (COMPLETE)
- ⏳ Narrative Shift (STUB - 20% done)
- ⏳ Crowd Behavior (STUB - 20% done)
- ⏳ Structural Inefficiency (STUB - 20% done)
- ⏳ Creative Synthesis (STUB - 20% done)

#### 1.1 Narrative Shift Alpha (Priority #1) 🔥

**Human Insight:** Markets move on stories, not just numbers. Detect when narratives change.

**Implementation Plan:**

```rust
// src/alphas/narrative_shift.rs

pub struct NarrativeShiftAlpha {
    // Track dominant narratives
    current_narratives: HashMap<String, f64>,  // narrative -> confidence

    // Sentiment analysis
    fed_sentiment: SentimentTracker,
    news_sentiment: SentimentTracker,
    twitter_sentiment: SentimentTracker,

    // Thresholds
    shift_threshold: f64,  // How much change = shift
    confidence_threshold: f64,

    // State
    last_update: SystemTime,
}

impl NarrativeShiftAlpha {
    pub fn detect_shift(&self, current: &MarketSnapshot) -> Option<NarrativeShift> {
        // 1. Analyze Fed minutes sentiment
        // 2. Analyze news headlines
        // 3. Analyze social media
        // 4. Compare to previous narratives
        // 5. Detect shifts (e.g., "inflation transitory" → "persistent")
    }
}
```

**Data Sources Needed:**
- [ ] Fed minutes API
- [ ] News API (NewsAPI.org - free tier)
- [ ] Twitter/X sentiment (optional)

**Estimated Time:** 1 day

---

#### 1.2 Crowd Behavior Alpha (Priority #2) 🔥

**Human Insight:** Retail traders are predictably irrational. Exploit FOMO and panic.

**Implementation Plan:**

```rust
// src/alphas/crowd_behavior.rs

pub struct CrowdBehaviorAlpha {
    // Track retail activity
    wsb_mentions: HashMap<Symbol, usize>,  // WallStreetBets mentions
    robinhood_holdings: HashMap<Symbol, f64>,  // Top holdings %

    // Meme stock lifecycle
    meme_stage: HashMap<Symbol, MemeStage>,

    // Options flow (retail loves options)
    call_put_ratio: HashMap<Symbol, f64>,

    // FOMO detector
    fomo_threshold: f64,
}

enum MemeStage {
    Discovery,     // Early mentions
    Acceleration,  // Viral spread
    Peak,          // Maximum hype
    Decline,       // Reality sets in
    Forgotten,     // Back to normal
}

impl CrowdBehaviorAlpha {
    pub fn detect_meme_cycle(&self, symbol: &Symbol) -> Option<MemeStage> {
        // Track lifecycle and generate contrarian signals
    }
}
```

**Data Sources Needed:**
- [ ] Reddit API (r/WallStreetBets)
- [ ] Options flow data (optional)
- [ ] Google Trends API

**Estimated Time:** 0.5 day

---

#### 1.3 Structural Inefficiency Alpha (Priority #3)

**Human Insight:** Known market mechanics create predictable price movements.

**Implementation Plan:**

```rust
// src/alphas/structural.rs

pub struct StructuralInefficiencyAlpha {
    // Index rebalancing calendar
    index_rebalance_dates: Vec<RebalanceEvent>,

    // Option expiry tracking
    option_expiry_calendar: HashMap<Symbol, Vec<ExpiryDate>>,

    // Corporate actions
    dividend_calendar: HashMap<Symbol, Vec<DividendEvent>>,

    // Thresholds
    gamma_squeeze_threshold: f64,
}

impl StructuralInefficiencyAlpha {
    pub fn check_rebalancing(&self, date: DateTime<Utc>) -> Vec<Signal> {
        // Predict which stocks will be bought/sold during rebalancing
    }

    pub fn check_gamma_squeeze(&self, symbol: &Symbol) -> Option<Signal> {
        // Detect option expiry gamma squeeze potential
    }
}
```

**Data Sources Needed:**
- [ ] Index rebalancing schedule (S&P 500, etc.)
- [ ] Options expiry dates
- [ ] Dividend calendar

**Estimated Time:** 0.5 day

---

#### 1.4 Creative Synthesis Alpha (Priority #4)

**Human Insight:** Combine unrelated signals creatively (weather → retail, sports → sentiment).

**Implementation Plan:**

```rust
// src/alphas/creative.rs

pub struct CreativeSynthesisAlpha {
    // Weather data
    weather_api: WeatherProvider,

    // Sports results
    sports_results: HashMap<Team, Vec<GameResult>>,

    // Economic calendar
    earnings_surprises: HashMap<Symbol, f64>,

    // Correlation tracking
    discovered_correlations: Vec<CorrelationPattern>,
}

impl CreativeSynthesisAlpha {
    pub fn weather_retail_signal(&self, region: &str) -> Option<Signal> {
        // Bad weather → more online shopping
        // Good weather → less retail traffic
    }

    pub fn sports_sentiment_signal(&self, team: &Team) -> Option<Signal> {
        // Team wins → local economy boost
        // Team loses → sentiment drop
    }
}
```

**Data Sources Needed:**
- [ ] Weather API (OpenWeatherMap - free)
- [ ] Sports scores API
- [ ] Economic calendar

**Estimated Time:** 0.5 day

---

## 🚀 Phase 5: Python Bindings (⏱️ 2-3 days)

**Why:** Enable rapid strategy development in Python while keeping Rust's speed

### 5.1 Setup PyO3 Infrastructure

```bash
# Add to Cargo.toml
[lib]
name = "quant_engine"
crate-type = ["cdylib", "rlib"]

[dependencies]
pyo3 = { version = "0.20", features = ["extension-module"] }
```

### 5.2 Expose Core Types

```rust
// src/human_layer/python_bindings.rs

use pyo3::prelude::*;

#[pyclass]
pub struct PyPrice {
    inner: Price,
}

#[pymethods]
impl PyPrice {
    #[new]
    pub fn new(value: f64) -> PyResult<Self> {
        Ok(PyPrice {
            inner: Price::new(value).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?,
        })
    }

    pub fn value(&self) -> f64 {
        self.inner.value()
    }
}

#[pyclass]
pub struct PyBacktester {
    inner: Backtester,
}

#[pymethods]
impl PyBacktester {
    #[new]
    pub fn new(initial_capital: f64) -> Self {
        // Initialize backtester
    }

    pub fn add_alpha(&mut self, alpha: PyObject) {
        // Add Python-defined alpha
    }

    pub fn run(&mut self, data: PyObject) -> PyResult<PyBacktestResult> {
        // Run backtest
    }
}
```

### 5.3 Create Python Type Stubs

```python
# python/quant_engine.pyi

class Price:
    def __init__(self, value: float) -> None: ...
    def value(self) -> float: ...

class Signal:
    symbol: str
    action: SignalAction
    confidence: float

class AlphaModel:
    def update(self, data: MarketSnapshot) -> None: ...
    def generate_signals(self) -> list[Signal]: ...

class Backtester:
    def __init__(self, initial_capital: float) -> None: ...
    def add_alpha(self, alpha: AlphaModel) -> None: ...
    def run(self, historical_data: dict) -> BacktestResult: ...
```

### 5.4 Example Python Strategy

```python
# python/examples/my_custom_alpha.py

from quant_engine import AlphaModel, Signal, SignalAction

class MyCustomAlpha(AlphaModel):
    """
    Human Insight: Buy when everyone is fearful, sell when greedy.
    """

    def __init__(self):
        self.fear_threshold = 0.7
        self.greed_threshold = 0.3

    def update(self, data):
        self.latest_data = data

    def generate_signals(self):
        signals = []

        for symbol, market_data in self.latest_data.items():
            vix = market_data.vix

            if vix > self.fear_threshold:
                # Extreme fear → buy opportunity
                signals.append(Signal(
                    symbol=symbol,
                    action=SignalAction.BUY,
                    confidence=0.8
                ))
            elif vix < self.greed_threshold:
                # Extreme greed → sell signal
                signals.append(Signal(
                    symbol=symbol,
                    action=SignalAction.SELL,
                    confidence=0.7
                ))

        return signals

# Usage
from quant_engine import Backtester

backtester = Backtester(initial_capital=100_000)
backtester.add_alpha(MyCustomAlpha())
result = backtester.run(historical_data)

print(f"Sharpe Ratio: {result.sharpe_ratio}")
print(f"Total Return: {result.total_return}%")
```

**Estimated Time:** 2-3 days

---

## 🔧 Medium Priority (Nice to Have)

### 6. Kelly Criterion Position Sizing (⏱️ 0.5 day)

```rust
// src/core/position_sizer.rs

pub struct KellyCriterion {
    // Kelly formula: f* = (p * b - q) / b
    // f* = fraction of capital to bet
    // p = probability of winning
    // b = odds received on wager
    // q = probability of losing (1 - p)
}

impl KellyCriterion {
    pub fn calculate_position_size(
        &self,
        win_rate: f64,
        avg_win: f64,
        avg_loss: f64,
        capital: f64,
    ) -> f64 {
        let p = win_rate;
        let q = 1.0 - win_rate;
        let b = avg_win / avg_loss.abs();

        let kelly_fraction = (p * b - q) / b;

        // Use fractional Kelly (25% or 50%) for safety
        let fractional_kelly = kelly_fraction * 0.25; // Quarter Kelly

        capital * fractional_kelly.max(0.0).min(0.25) // Cap at 25%
    }
}
```

### 7. Data Caching Layer (⏱️ 1 day)

```rust
// src/data/cache.rs

pub struct DataCache {
    redis: Option<Redis>,
    memory: Arc<DashMap<String, CachedData>>,
    ttl: Duration,
}

impl DataCache {
    pub async fn get_or_fetch<F, Fut>(
        &self,
        key: &str,
        fetcher: F,
    ) -> Result<MarketData>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<MarketData>>,
    {
        // 1. Check memory cache
        // 2. Check Redis cache
        // 3. Fetch from source
        // 4. Update caches
    }
}
```

### 8. SIMD Indicators (⏱️ 1-2 days)

```cpp
// src/indicators/simd/ema.cpp

#include <immintrin.h>

// AVX2 SIMD for 8 floats at once
void calculate_ema_simd(
    const float* prices,
    float* output,
    size_t len,
    float multiplier
) {
    __m256 mult = _mm256_set1_ps(multiplier);
    __m256 one_minus_mult = _mm256_set1_ps(1.0f - multiplier);

    __m256 ema = _mm256_loadu_ps(prices); // Initialize with first 8 values

    for (size_t i = 8; i < len; i += 8) {
        __m256 price = _mm256_loadu_ps(&prices[i]);

        // EMA = price * mult + prev_ema * (1 - mult)
        ema = _mm256_add_ps(
            _mm256_mul_ps(price, mult),
            _mm256_mul_ps(ema, one_minus_mult)
        );

        _mm256_storeu_ps(&output[i], ema);
    }
}
```

---

## 📊 Roadmap Summary

| Task | Priority | Time | Benefit |
|------|----------|------|---------|
| **Narrative Shift Alpha** | 🔥 High | 1 day | Unlock narrative edge |
| **Crowd Behavior Alpha** | 🔥 High | 0.5 day | Exploit retail irrationality |
| **Structural Alpha** | 🔥 High | 0.5 day | Predictable opportunities |
| **Creative Synthesis** | 🔥 High | 0.5 day | Novel signals |
| **Python Bindings** | 🔥 High | 2-3 days | Rapid strategy dev |
| **Kelly Criterion** | 🔶 Medium | 0.5 day | Optimal position sizing |
| **Data Caching** | 🔶 Medium | 1 day | Reduce latency |
| **SIMD Indicators** | 🔶 Medium | 1-2 days | 10-100x speedup |

---

## 🎯 Recommended Sequence

### Week 1: Complete Alphas (3 days)
1. Day 1: Narrative Shift Alpha
2. Day 2: Crowd Behavior + Structural
3. Day 3: Creative Synthesis + Testing

### Week 2: Python Bindings (3 days)
1. Day 1: PyO3 setup + Core types
2. Day 2: Backtester bindings
3. Day 3: Examples + Documentation

### Week 3: Optimizations (3 days)
1. Day 1: Kelly Criterion
2. Day 2: Data Caching
3. Day 3: SIMD Indicators (start)

**Total:** ~9 days to complete everything

---

## 🚦 Quick Wins (Do These Today)

### Quick Win #1: Stub Out Alpha Interfaces (30 min)

```rust
// Give each alpha a proper structure

impl NarrativeShiftAlpha {
    pub fn new() -> Self {
        Self {
            current_narratives: HashMap::new(),
            shift_threshold: 0.3,
            confidence_threshold: 0.6,
            last_update: SystemTime::now(),
        }
    }
}
```

### Quick Win #2: Add API Key Management (15 min)

```rust
// src/config/secrets.rs

use std::env;

pub struct Secrets {
    pub news_api_key: Option<String>,
    pub reddit_client_id: Option<String>,
    pub twitter_bearer_token: Option<String>,
}

impl Secrets {
    pub fn from_env() -> Self {
        Self {
            news_api_key: env::var("NEWS_API_KEY").ok(),
            reddit_client_id: env::var("REDDIT_CLIENT_ID").ok(),
            twitter_bearer_token: env::var("TWITTER_BEARER_TOKEN").ok(),
        }
    }
}
```

### Quick Win #3: Update README with Status (10 min)

Add current status and what's next to README.md.

---

## 💡 Where to Start?

**Recommendation:** Start with **Narrative Shift Alpha**

**Why?**
1. Most impactful (narrative drives markets)
2. Good learning exercise
3. Sets pattern for other alphas
4. Can use free APIs (NewsAPI.org)

**Next Command:**
```bash
# Create the skeleton
touch src/alphas/narrative_shift_impl.rs

# Or let me help you implement it!
```

---

**Ready to start?** Let me know which task you want to tackle first! 🚀
